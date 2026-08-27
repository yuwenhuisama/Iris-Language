//! Literal and call lowering.

use iris_runtime::NativeSelector;
use iris_syntax::{BinaryOperator, Expression};

use super::expressions;
use super::lowering::Lowering;
use super::{CompileError, FloatWidth, Instruction, Register};

impl<'a, 'b> Lowering<'a, 'b> {
    /// Lowers a literal by RE-LEXING its text.
    ///
    /// The parser keeps a literal as source text, and the lexer is what turns
    /// it into a value under the chapter 02 rules - suffixes, radix prefixes,
    /// separators and precision warnings. Parsing the text here would be a
    /// second literal implementation, and a differential row would then compare
    /// this backend's literal rules against the lexer's.
    pub(super) fn literal(&mut self, text: &str) -> Result<Register, CompileError> {
        let destination = self.allocate()?;
        // The parser keeps these keyword values as literal TEXT and the lexer
        // does not convert them, so they are recognised here.
        match text {
            "nil" => {
                self.instructions.push(Instruction::LoadNil { destination });
                return Ok(destination);
            }
            "true" | "false" => {
                self.instructions.push(Instruction::LoadBool {
                    destination,
                    value: text == "true",
                });
                return Ok(destination);
            }
            _ => {}
        }
        if let Some((bytes, mutable)) = byte_literal(text)? {
            self.instructions.push(if mutable {
                Instruction::LoadByteArray { destination, bytes }
            } else {
                Instruction::LoadBytes { destination, bytes }
            });
            return Ok(destination);
        }
        if let Some(string_source) = mutable_string_literal(text) {
            let source = self.literal(&string_source)?;
            self.instructions.push(Instruction::MakeMutableString {
                destination,
                source,
            });
            return Ok(destination);
        }
        if let Some((pattern, flags)) = regex_literal(text) {
            // `IRIS-V1-COLLECTIONS-C081` canonicalizes flags into `imsx` order
            // with absent flags omitted, so `/a+/im` and `/a+/mi` are the SAME
            // value and hash alike. A repeated or unknown flag is a lexical
            // refusal rather than a silently accepted duplicate.
            let mut canonical = String::new();
            for flag in "imsx".chars() {
                if flags.contains(flag) {
                    canonical.push(flag);
                }
            }
            if flags.chars().count() != canonical.chars().count()
                || !flags.chars().all(|flag| "imsx".contains(flag))
            {
                return Err(CompileError::new("regex flags"));
            }
            // The pattern is compiled here only to REJECT it: an unsupported
            // construct names itself, and the reference reports which one, so
            // the refusal is derived from the engine rather than guessed.
            if let Err(error) = regex::RegexBuilder::new(&pattern)
                .case_insensitive(canonical.contains('i'))
                .multi_line(canonical.contains('m'))
                .dot_matches_new_line(canonical.contains('s'))
                .ignore_whitespace(canonical.contains('x'))
                .unicode(true)
                .build()
            {
                let reported = error.to_string();
                return Err(CompileError::new(if reported.contains("backreference") {
                    "regex backreference"
                } else if reported.contains("look-around")
                    || reported.contains("look-behind")
                    || reported.contains("look-ahead")
                {
                    "regex lookaround"
                } else {
                    "regex syntax"
                }));
            }
            self.instructions.push(Instruction::LoadRegex {
                destination,
                pattern,
                flags: canonical,
            });
            return Ok(destination);
        }
        let conversion = iris_lexer::convert_literals(text);
        if !conversion.diagnostics().is_empty() {
            return Err(CompileError::new("rejected literal"));
        }
        let [literal] = conversion.values() else {
            return Err(CompileError::new("literal"));
        };
        self.instructions.push(match literal {
            iris_lexer::Literal::Integer(digits) => Instruction::LoadInteger {
                destination,
                digits: digits.clone(),
            },
            iris_lexer::Literal::Float64(number) => Instruction::LoadFloat64 {
                destination,
                bits: number.to_bits(),
            },
            iris_lexer::Literal::Float32(number) => Instruction::LoadFloat32 {
                destination,
                bits: number.to_bits(),
            },
            iris_lexer::Literal::String(text) => {
                if text.contains("${") {
                    return self.interpolated_text(text);
                }
                Instruction::LoadText {
                    destination,
                    text: text.clone(),
                }
            }
        });
        Ok(destination)
    }

    /// `Float32.from_bits(bits)`, `value.to_bits()` and `value.hash()` are the
    /// call shapes this subset covers. A general call needs frames and user
    /// Methods it does not have.
    pub(super) fn call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) -> Result<Register, CompileError> {
        if matches!(callee, Expression::Name(name) if name == "super") {
            let Some((owner, selector)) = self.current_method.clone() else {
                return Err(CompileError::new("super outside method"));
            };
            let receiver = self
                .lookup("self")
                .ok_or_else(|| CompileError::new("super outside method"))?;
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::SendSuper {
                destination,
                receiver,
                owner,
                selector,
                first,
                count,
            });
            return Ok(destination);
        }
        if matches!(callee, Expression::Name(name) if name == "using") {
            let [resource, block] = arguments else {
                return Err(CompileError::new("using arity"));
            };
            let resource = self.expression(resource)?;
            let block = self.expression(block)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Using {
                destination,
                resource,
                block,
            });
            return Ok(destination);
        }
        if let Expression::Name(name) = callee {
            let callee = self.lookup(name);
            // A bare call inside a method with a receiver is a send to SELF:
            // `property tag: Symbol = arm()` calls the object's own `arm`.
            // Only a name no binding claims is treated this way, so a local
            // holding a closure still wins.
            if callee.is_none()
                && let Some(receiver) = self.lookup("self")
                && self
                    .signatures
                    .iter()
                    .any(|signature| signature.selector == name && signature.receiver)
            {
                let (first, count) = self.argument_window(arguments)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Send {
                    destination,
                    receiver,
                    selector: name.clone(),
                    first,
                    count,
                });
                return Ok(destination);
            }
            // A bare call in a MODULE body names that module's own function,
            // which has no receiver, so it resolves by index like `M.f()`.
            if callee.is_none()
                && let Some(module) = self.enclosing_module.clone()
                && let Some(function) = self.resolve(&module, name)
            {
                let (first, count) = self.argument_window(arguments)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Call {
                    destination,
                    function,
                    first,
                    count,
                });
                return Ok(destination);
            }
            if callee.is_none() && !matches!(name.as_str(), "Integer" | "Float64") {
                return Err(CompileError::new("call bare name"));
            }
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::BareCall {
                destination,
                callee,
                name: name.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if let Expression::ContractView { receiver, selector } = callee {
            let receiver = self.expression(receiver)?;
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::SendContract {
                destination,
                receiver,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        let Expression::Member { receiver, selector } = callee else {
            return Err(CompileError::new(match callee {
                Expression::Closure { .. } => "call closure",
                _ => "call callee",
            }));
        };
        if let Expression::Name(name) = receiver.as_ref()
            && self
                .lookup(name)
                .is_some_and(|register| self.method_values.contains(&register))
            && selector == "call"
        {
            return Err(CompileError::new("Method.call"));
        }
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "Iteration")
            && selector == "yield"
        {
            let [value] = arguments else {
                return Err(CompileError::new("yield arity"));
            };
            let value = self.expression(value)?;
            let destination = self.allocate()?;
            self.instructions
                .push(Instruction::BuildIterationYield { destination, value });
            return Ok(destination);
        }
        if selector == "define_method" {
            let [
                name,
                Expression::Closure {
                    parameters, body, ..
                },
            ] = arguments
            else {
                return Err(CompileError::new("define_method arity"));
            };
            let receiver = self.expression(receiver)?;
            let name = self.expression(name)?;
            let function = self.dynamic_method(parameters, body)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::DefineMethod {
                destination,
                receiver,
                name,
                function,
            });
            return Ok(destination);
        }
        if let Expression::Name(namespace) = receiver.as_ref()
            && matches!(
                namespace.as_str(),
                "Reflection::Class" | "Reflection::Module"
            )
            && selector == "invoke"
        {
            return Err(CompileError::new(format!("{namespace}.invoke")));
        }
        if let Expression::Name(namespace) = receiver.as_ref()
            && self.lookup(namespace).is_none()
        {
            // `C022` names each Encoding explicitly, so a decode is routed by
            // the namespace the source wrote rather than by a runtime lookup.
            let encoding = match namespace.as_str() {
                "Encoding::UTF_8" => Some("Encoding::UTF_8"),
                "Encoding::UTF_16LE" => Some("Encoding::UTF_16LE"),
                "Encoding::UTF_16BE" => Some("Encoding::UTF_16BE"),
                "Encoding::Latin_1" => Some("Encoding::Latin_1"),
                _ => None,
            };
            if let Some(encoding) = encoding
                && selector == "decode"
                && !arguments.is_empty()
            {
                let value = self.expression(&arguments[0])?;
                let (first, count) = self.argument_window(&arguments[1..])?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::EncodingDecode {
                    destination,
                    encoding,
                    value,
                    first,
                    count,
                });
                return Ok(destination);
            }
            // `C025` refuses an IMPLICIT selection: a host default names no
            // Encoding, and so does omitting the argument entirely.
            if namespace == "Encoding" && selector == "default" {
                let destination = self.allocate()?;
                self.instructions.push(Instruction::RaiseEncodingSelection {
                    destination,
                    code: "EncodingSelectionError",
                });
                return Ok(destination);
            }
            if namespace == "File" && selector == "read_text" {
                let names_encoding = arguments.iter().skip(1).any(|argument| {
                    matches!(argument, Expression::KeywordArgument { name, value }
                        if name == "encoding"
                            && !matches!(value.as_ref(),
                                Expression::Symbol(named) if named == "host_default"))
                });
                if !names_encoding {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::RaiseEncodingSelection {
                        destination,
                        code: "ENCODING_EXPLICIT_REQUIRED",
                    });
                    return Ok(destination);
                }
            }
        }
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "FFI")
            && self.lookup("FFI").is_none()
            && selector == "open"
            && !arguments.is_empty()
        {
            let path = self.expression(&arguments[0])?;
            let (first, count) = self.argument_window(&arguments[1..])?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::FfiOpen {
                destination,
                path,
                first,
                count,
            });
            return Ok(destination);
        }
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "IrisValue")
            && self.lookup("IrisValue").is_none()
        {
            match (selector.as_str(), arguments) {
                ("encode", [value, ..]) => {
                    let value = self.expression(value)?;
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::IrisValueEncode { destination, value });
                    return Ok(destination);
                }
                // The options carry `element_limit:`, which `C017` consults
                // BEFORE any declared length is trusted.
                ("decode", [stream, options @ ..]) => {
                    let stream = self.expression(stream)?;
                    let (first, count) = self.argument_window(options)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::IrisValueDecode {
                        destination,
                        stream,
                        first,
                        count,
                    });
                    return Ok(destination);
                }
                _ => return Err(CompileError::new("call unbound receiver")),
            }
        }
        // `IRIS-V1-COLLECTIONS-C042` pins the Unicode data version, so the
        // version string is read from the same tables the operations use
        // rather than being written down twice.
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "Unicode")
            && self.lookup("Unicode").is_none()
            && selector == "version"
            && arguments.is_empty()
        {
            let destination = self.allocate()?;
            self.instructions
                .push(Instruction::UnicodeVersion { destination });
            return Ok(destination);
        }
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "Gate")
            && self.lookup("Gate").is_none()
        {
            match (selector.as_str(), arguments) {
                ("new", []) => {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::GateNew { destination });
                    return Ok(destination);
                }
                // A completion may post a value or none, and `C014` gives an
                // absent one the same meaning as `nil`.
                ("complete", [gate] | [gate, _]) => {
                    let gate = self.expression(gate)?;
                    let value = match arguments.get(1) {
                        Some(value) => self.expression(value)?,
                        None => {
                            let destination = self.allocate()?;
                            self.instructions.push(Instruction::LoadNil { destination });
                            destination
                        }
                    };
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::GateComplete {
                        destination,
                        gate,
                        value,
                    });
                    return Ok(destination);
                }
                _ => return Err(CompileError::new("call unbound receiver")),
            }
        }
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "Host")
            && selector == "run"
        {
            let [argument] = arguments else {
                return Err(CompileError::new("Host.run arity"));
            };
            let task = self.expression(argument)?;
            let destination = self.allocate()?;
            self.instructions
                .push(Instruction::HostRun { destination, task });
            return Ok(destination);
        }
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "Diagnostics")
            && selector == "unobserved_failures"
            && arguments.is_empty()
        {
            let destination = self.allocate()?;
            self.instructions
                .push(Instruction::UnobservedFailures { destination });
            return Ok(destination);
        }
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "JSON")
            && matches!(selector.as_str(), "decode" | "encode")
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Json {
                destination,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if let Expression::Name(namespace) = receiver.as_ref()
            && matches!(namespace.as_str(), "Revision" | "RevisionHistory")
            && matches!(
                (namespace.as_str(), selector.as_str()),
                ("Revision", "subscribe" | "flush" | "event_errors")
                    | ("RevisionHistory", "events" | "prune")
            )
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Revision {
                destination,
                namespace: namespace.clone(),
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if let Expression::Name(class) = receiver.as_ref()
            && selector == "open"
            && let Some(class) = self.class_index(class)
        {
            let [callback] = arguments else {
                return Err(CompileError::new("Class.open arity"));
            };
            let callback = self.expression(callback)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::OpenClass {
                destination,
                class,
                callback,
            });
            return Ok(destination);
        }
        if let Expression::ClosedGeneric { name, .. } = receiver.as_ref()
            && selector == "open"
            && let Some(class) = self.class_index(name)
        {
            let [callback] = arguments else {
                return Err(CompileError::new("Class.open arity"));
            };
            let callback = self.expression(callback)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::OpenClass {
                destination,
                class,
                callback,
            });
            return Ok(destination);
        }
        if let Expression::ClosedGeneric { name, .. } = receiver.as_ref()
            && selector == "new"
            && let Some(class) = self.class_index(name)
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::New {
                destination,
                class,
                first,
                count,
            });
            return Ok(destination);
        }
        if let Expression::Name(namespace) = receiver.as_ref()
            && matches!(
                namespace.as_str(),
                "Reflection::Class"
                    | "Reflection::Module"
                    | "Reflection::Contract"
                    | "Reflection::Object"
            )
            && matches!(
                (namespace.as_str(), selector.as_str()),
                ("Reflection::Class" | "Reflection::Module", "method")
                    | ("Reflection::Class", "properties" | "revision")
                    | ("Reflection::Contract", "requirement")
                    | ("Reflection::Object", "get_ivar" | "set_ivar")
            )
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Reflection {
                destination,
                namespace: namespace.clone(),
                selector: selector.clone(),
                first,
                count,
            });
            if matches!(
                namespace.as_str(),
                "Reflection::Class" | "Reflection::Module"
            ) && selector == "method"
            {
                self.method_values.push(destination);
            }
            return Ok(destination);
        }
        if matches!(receiver.as_ref(), Expression::Member { receiver, selector }
        if selector == "type"
            && matches!(
                receiver.as_ref(),
                Expression::ReifiedType(iris_syntax::TypeExpression::Name(_)
                    | iris_syntax::TypeExpression::Generic { .. })
            ))
            && selector == "kind"
            && arguments.is_empty()
        {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::LoadSymbol {
                destination,
                name: "nominal".to_owned(),
            });
            return Ok(destination);
        }
        if let Expression::Name(name) = receiver.as_ref()
            && selector == "from_bits"
        {
            let width = match name.as_str() {
                "Float32" => FloatWidth::Bits32,
                "Float64" => FloatWidth::Bits64,
                _ => return Err(CompileError::new("call")),
            };
            let [bits] = arguments else {
                return Err(CompileError::new("from_bits arity"));
            };
            let bits = self.expression(bits)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::FromBits {
                destination,
                width,
                bits,
            });
            return Ok(destination);
        }
        if let Expression::Name(class) = receiver.as_ref()
            && selector == "new"
            && let Some(class) = self.class_index(class)
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::New {
                destination,
                class,
                first,
                count,
            });
            return Ok(destination);
        }
        if let Expression::ClosedGeneric { name, .. } = receiver.as_ref()
            && let Some(class) = self.class_index(name)
            && self.signatures.iter().any(|signature| {
                signature.module == self.classes[class].name
                    && signature.selector == selector
                    && signature.class_method
            })
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::SendClass {
                destination,
                class,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if selector == "same?" {
            let [right] = arguments else {
                return Err(CompileError::new("same? arity"));
            };
            let left = self.expression(receiver)?;
            let right = self.expression(right)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Identity {
                destination,
                left,
                right,
            });
            return Ok(destination);
        }
        if selector == "call" {
            let receiver = self.expression(receiver)?;
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Send {
                destination,
                receiver,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if let Expression::Name(class) = receiver.as_ref()
            && let Some(class) = self.class_index(class)
            && self.signatures.iter().any(|signature| {
                signature.module == self.classes[class].name
                    && signature.selector == selector
                    && signature.class_method
            })
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::SendClass {
                destination,
                class,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        // A resolved module function is called by INDEX. Resolution happened
        // before lowering, so no name is looked up at run time.
        if let Expression::Name(module) = receiver.as_ref()
            && let Some(function) = self.resolve(module, selector)
        {
            // A resolved function binds its arguments POSITIONALLY, and the
            // backend has no keyword parameters, so a keyword argument here
            // would silently fill a positional slot with the wrapper - a
            // wrong answer where the reference raises ArgumentError.
            if arguments
                .iter()
                .any(|argument| matches!(argument, Expression::KeywordArgument { .. }))
            {
                return Err(CompileError::new("expression keyword argument"));
            }
            let parameters = &self.signatures[function].parameters;
            let required = parameters
                .iter()
                .take_while(|parameter| parameter.default.is_none())
                .count();
            if arguments.len() < required || arguments.len() > parameters.len() {
                return Err(CompileError::new("call arity"));
            }
            let count =
                u16::try_from(parameters.len()).map_err(|_| CompileError::new("call too wide"))?;
            let mut lowered = Vec::with_capacity(parameters.len());
            for argument in arguments {
                lowered.push(self.expression(argument)?);
            }
            for parameter in &parameters[arguments.len()..] {
                let default = parameter
                    .default
                    .as_ref()
                    .ok_or_else(|| CompileError::new("call arity"))?;
                lowered.push(self.expression(default)?);
            }
            // Arguments are copied into a CONTIGUOUS window, so the call names
            // a range and the callee sees them as its leading registers.
            let first = self.next_register;
            for source in lowered {
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source,
                });
            }
            // A zero-argument call still needs a window start inside the file.
            if count == 0 {
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
            }
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Call {
                destination,
                function,
                first,
                count,
            });
            return Ok(destination);
        }
        if arguments.is_empty()
            && let Some(native) = match selector.as_str() {
                "to_bits" => Some("to_bits"),
                // C146 fixes the public hash of each numeric and singleton
                // value, which V073 compares across backends.
                "hash" => Some("hash"),
                _ => None,
            }
        {
            let operand = self.expression(receiver)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Unary {
                destination,
                selector: native,
                operand,
            });
            return Ok(destination);
        }
        if NativeSelector::from_source(selector).is_some() {
            let receiver = self.expression(receiver)?;
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Send {
                destination,
                receiver,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if let Some(construct) = expressions::ordinary_receiver_decline(receiver, |name| {
            self.lookup(name).is_some()
                || self.class_index(name).is_some()
                // A top-level binding is a legitimate RECEIVER inside a method
                // body, not only a readable name: `mut log = []` followed by
                // `log.append(:x)` in a method is how most of the corpus
                // accumulates. The check consulted locals and classes only, so
                // the send declined while the plain read already worked.
                || self
                    .program_bindings
                    .iter()
                    .any(|binding| binding.name == *name)
                || matches!(name, "nil" | "true" | "false")
        }) {
            return Err(CompileError::new(construct));
        }
        let receiver = self.expression(receiver)?;
        let (first, count) = self.argument_window(arguments)?;
        let destination = self.allocate()?;
        self.instructions.push(Instruction::Send {
            destination,
            receiver,
            selector: selector.clone(),
            first,
            count,
        });
        Ok(destination)
    }

    pub(super) fn class_index(&self, name: &str) -> Option<usize> {
        self.classes.iter().position(|class| class.name == name)
    }

    pub(super) fn contract_index(&self, name: &str) -> Option<usize> {
        self.contracts
            .iter()
            .position(|contract| contract.name == name)
    }

    pub(super) fn argument_window(
        &mut self,
        arguments: &[Expression],
    ) -> Result<(Register, u16), CompileError> {
        let count =
            u16::try_from(arguments.len()).map_err(|_| CompileError::new("call too wide"))?;
        let mut lowered = Vec::with_capacity(arguments.len());
        for argument in arguments {
            lowered.push(self.expression(argument)?);
        }
        let first = self.next_register;
        for source in lowered {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Move {
                destination,
                source,
            });
        }
        if count == 0 {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::LoadNil { destination });
        }
        Ok((first, count))
    }
}

/// Decodes byte prefixes before the ordinary lexer can erase their value kind.
fn byte_literal(source: &str) -> Result<Option<(Vec<u8>, bool)>, CompileError> {
    let Some(prefix_end) = source.find(['"', '\'']) else {
        return Ok(None);
    };
    let (prefix, body) = source.split_at(prefix_end);
    let mutable = match prefix {
        "b" | "br" => false,
        "mb" | "mbr" => true,
        _ => return Ok(None),
    };
    let raw = prefix.ends_with('r');
    let quote = body
        .chars()
        .next()
        .ok_or_else(|| CompileError::new("literal byte body"))?;
    let body = body
        .strip_prefix(quote)
        .and_then(|body| body.strip_suffix(quote))
        .ok_or_else(|| CompileError::new("literal byte body"))?;
    let mut bytes = Vec::new();
    let mut characters = body.chars();
    while let Some(character) = characters.next() {
        if raw || character != '\\' {
            let mut buffer = [0_u8; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        let escape = characters
            .next()
            .ok_or_else(|| CompileError::new("literal byte escape"))?;
        match escape {
            'x' => {
                let high = characters
                    .next()
                    .ok_or_else(|| CompileError::new("literal byte hex escape"))?;
                let low = characters
                    .next()
                    .ok_or_else(|| CompileError::new("literal byte hex escape"))?;
                let pair = format!("{high}{low}");
                let value = u8::from_str_radix(&pair, 16)
                    .map_err(|_| CompileError::new("literal byte hex escape"))?;
                bytes.push(value);
            }
            'n' => bytes.push(b'\n'),
            'r' => bytes.push(b'\r'),
            't' => bytes.push(b'\t'),
            '0' => bytes.push(0),
            '\\' => bytes.push(b'\\'),
            '"' => bytes.push(b'"'),
            '\'' => bytes.push(b'\''),
            _ => return Err(CompileError::new("literal byte escape")),
        }
    }
    Ok(Some((bytes, mutable)))
}

/// Returns equivalent String source so all String-family body rules stay shared.
/// Splits a `/pattern/flags` literal, when the text is one.
///
/// An interpolating literal is NOT handled here: `${..}` has to be evaluated
/// and escaped at run time, which a compile-time literal cannot do.
fn regex_literal(source: &str) -> Option<(String, String)> {
    let body = source.strip_prefix('/')?;
    let end = body.rfind('/')?;
    let (pattern, flags) = body.split_at(end);
    if pattern.contains("${") {
        return None;
    }
    Some((pattern.to_owned(), flags.get(1..)?.to_owned()))
}

fn mutable_string_literal(source: &str) -> Option<String> {
    let prefix_end = source.find(['"', '\''])?;
    let (prefix, body) = source.split_at(prefix_end);
    match prefix {
        "m" => Some(body.to_owned()),
        "mr" => Some(format!("r{body}")),
        _ => None,
    }
}

/// The native selector for a binary operator, when this backend covers it.
///
/// Only operators whose whole meaning is a native send appear here. A range,
/// regex or type operator carries semantics beyond the kernel send, so it is
/// declined rather than approximated.
pub(super) fn binary_selector(operator: &BinaryOperator) -> Result<&'static str, CompileError> {
    Ok(match operator {
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Power => "**",
        BinaryOperator::ShiftLeft => "<<",
        BinaryOperator::ShiftRight => ">>",
        BinaryOperator::BitwiseAnd => "&",
        BinaryOperator::BitwiseXor => "^",
        BinaryOperator::BitwiseOr => "|",
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::Less => "<",
        BinaryOperator::LessEqual => "<=",
        BinaryOperator::Greater => ">",
        BinaryOperator::GreaterEqual => ">=",
        BinaryOperator::Compare => "<=>",
        other => return Err(CompileError::new(format!("operator {other:?}"))),
    })
}

pub(super) fn construct_name(expression: &Expression) -> String {
    let name = match expression {
        Expression::ReifiedType(_) => "expression reified type",
        Expression::ClosedGeneric { .. } => "expression closed generic",
        Expression::KeywordArgument { .. } => "expression keyword argument",
        Expression::ContractView { .. } => "expression contract view",
        Expression::ClassVar(_) => "expression class variable",
        Expression::Symbol(_) => "symbol",
        Expression::Hash(_) => "hash",
        Expression::Tuple(_) => "tuple",
        Expression::Member { .. } => "member",
        Expression::Index { .. } => "index",
        Expression::Closure { .. } => "closure",
        Expression::If { .. } => "if",
        Expression::While { .. } => "while",
        Expression::Try { .. } => "try",
        Expression::Await(_) => "await",
        Expression::Yield(_) => "yield",
        Expression::Assignment { .. } => "assignment",
        Expression::Name(_)
        | Expression::Literal(_)
        | Expression::Array(_)
        | Expression::Call { .. }
        | Expression::Unary { .. }
        | Expression::Binary { .. }
        | Expression::Grouped(_)
        | Expression::RawIvar(_)
        | Expression::GlobalVar(_) => "expression covered",
    };
    name.to_owned()
}

/// The ordinary operator a COMPOUND assignment sends, per `IRIS-V1-CONTROL-C036`.
///
/// `Assign` and the two logical forms answer None because they are not a send:
/// a plain assign writes directly, and a logical assignment short-circuits.
pub(super) const fn compound_selector(
    operator: iris_syntax::AssignmentOperator,
) -> Option<&'static str> {
    use iris_syntax::AssignmentOperator as Operator;
    match operator {
        Operator::Add => Some("+"),
        Operator::Subtract => Some("-"),
        Operator::Multiply => Some("*"),
        Operator::Divide => Some("/"),
        Operator::Power => Some("**"),
        Operator::BitwiseAnd => Some("&"),
        Operator::BitwiseOr => Some("|"),
        Operator::BitwiseXor => Some("^"),
        Operator::ShiftLeft => Some("<<"),
        Operator::ShiftRight => Some(">>"),
        Operator::Assign | Operator::LogicalAnd | Operator::LogicalOr => None,
    }
}
