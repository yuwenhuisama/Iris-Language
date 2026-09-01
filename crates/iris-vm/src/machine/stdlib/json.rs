use iris_runtime::{ArrayRef, HashRef, KernelError, Value};

use super::super::{Machine, MachineError, selector_id};

impl Machine {
    pub(crate) fn json_call(
        &mut self,
        selector: &str,
        arguments: &[Value],
        program: &crate::compile::Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<Value, MachineError> {
        let [value, rest @ ..] = arguments else {
            return Err(MachineError::Kernel(KernelError::Arity));
        };
        // `C013` makes a decode limit a REFUSAL before the offending container
        // is allocated, not a truncation afterwards. It arrives as a KEYWORD
        // argument, so accepting only one argument refused the call outright.
        let depth_limit = rest.iter().find_map(|option| match option {
            Value::KeywordArgument(name, limit) if name == "depth" => match &**limit {
                Value::Integer(limit) => limit.to_usize(),
                _ => None,
            },
            _ => None,
        });
        match selector {
            "decode" => {
                // `C012` raises EncodingError for invalid UTF-8 BEFORE any
                // JSON token is interpreted, so a byte input is refused at the
                // BOUNDARY rather than part-way through a parse.
                let text =
                    match value {
                        Value::Text(text) => text.clone(),
                        Value::Bytes(bytes) => String::from_utf8(bytes.clone())
                            .map_err(|_| MachineError::EncodingError)?,
                        Value::ByteArray(bytes) => String::from_utf8(bytes.bytes())
                            .map_err(|_| MachineError::EncodingError)?,
                        _ => return Err(MachineError::Kernel(KernelError::Type)),
                    };
                let mut cursor = text.chars().peekable();
                decode_json(&mut cursor, depth_limit, 0)
            }
            "encode" => {
                // `C002` leaves ordering to the caller, so a document is
                // rendered in INSERTION order unless canonical ordering is
                // selected by name.
                let canonical = rest.iter().any(|option| {
                    matches!(option, Value::KeywordArgument(name, flag)
                        if name == "canonical" && **flag == Value::Bool(true))
                });
                let mut rendered = String::new();
                self.encode_json(value, canonical, &mut rendered, program, classes)?;
                Ok(Value::Text(rendered))
            }
            _ => Err(MachineError::MessageNotFound {
                receiver_class: "JSON".to_owned(),
                selector: selector.to_owned(),
            }),
        }
    }
}

impl Machine {
    /// Renders a `C011` JSON-compatible value.
    ///
    /// The encoder REJECTS an unsupported value rather than falling back to
    /// `to_string`, `inspect`, object identity or raw ivar scanning.
    fn encode_json(
        &mut self,
        value: &Value,
        canonical: bool,
        output: &mut String,
        program: &crate::compile::Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<(), MachineError> {
        match value {
            Value::Nil => output.push_str("null"),
            Value::Bool(flag) => output.push_str(if *flag { "true" } else { "false" }),
            Value::Integer(number) => output.push_str(&number.decimal_text()),
            Value::Text(text) => render_json_text(text, output),
            Value::Array(values) => {
                output.push('[');
                for (index, element) in values.elements().iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    self.encode_json(element, canonical, output, program, classes)?;
                }
                output.push(']');
            }
            Value::Hash(entries) => {
                let mut pairs = Vec::new();
                for (key, held) in entries.entries() {
                    let Value::Text(key) = key else {
                        return Err(MachineError::SerializationError);
                    };
                    pairs.push((key, held));
                }
                // Canonical ordering sorts by KEY, which is what makes two
                // encodings of one document comparable byte for byte.
                if canonical {
                    pairs.sort_by(|left, right| left.0.cmp(&right.0));
                }
                output.push('{');
                for (index, (key, held)) in pairs.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    render_json_text(key, output);
                    output.push(':');
                    self.encode_json(held, canonical, output, program, classes)?;
                }
                output.push('}');
            }
            // `C004` makes participation an OPT-IN `for Serializable` promise that
            // duck typing, reflection visibility or a merely matching method must
            // not imply, and `C005` makes the representation ordinary Iris data
            // the Class chooses - so it is obtained by ASKING the class rather
            // than by inspecting the object.
            Value::Object(object) => {
                let class = self
                    .runtime
                    .class_of(*object)
                    .map_err(MachineError::Construction)?;
                let declares = classes
                    .iter()
                    .position(|known| *known == class)
                    .and_then(|index| program.classes.get(index))
                    .is_some_and(|declaration| {
                        declaration.contracts.iter().any(|contract| {
                            program
                                .contracts
                                .get(*contract)
                                .is_some_and(|contract| contract.name == "Serializable")
                        })
                    });
                if !declares {
                    return Err(MachineError::SerializationError);
                }
                // `C005` makes the representation ordinary Iris data the
                // Class CHOOSES, so it is obtained by running the class's own
                // `serialize` rather than by inspecting the object.
                let selector =
                    selector_id(program, "serialize").ok_or(MachineError::SerializationError)?;
                let method = self
                    .runtime
                    .dispatch_instance(*object, selector)
                    .map_err(|_| MachineError::SerializationError)?;
                let function = usize::try_from(method.body().raw())
                    .map_err(|_| MachineError::SerializationError)?;
                let callee = program
                    .functions
                    .get(function)
                    .cloned()
                    .ok_or(MachineError::SerializationError)?;
                let returned = self.run_body(
                    &callee.instructions,
                    callee.registers,
                    vec![Value::Object(*object)],
                    program,
                    classes,
                )?;
                let represented = returned.into_iter().next().unwrap_or(Value::Nil);
                self.encode_json(&represented, canonical, output, program, classes)?;
            }
            _ => return Err(MachineError::SerializationError),
        }
        Ok(())
    }
}

fn render_json_text(value: &str, output: &mut String) {
    output.push('"');
    for scalar in value.chars() {
        match scalar {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            scalar => output.push(scalar),
        }
    }
    output.push('"');
}

fn decode_json(
    cursor: &mut std::iter::Peekable<std::str::Chars<'_>>,
    depth_limit: Option<usize>,
    depth: usize,
) -> Result<Value, MachineError> {
    while cursor.peek().is_some_and(|scalar| scalar.is_whitespace()) {
        cursor.next();
    }
    let Some(&scalar) = cursor.peek() else {
        return Err(MachineError::JsonSyntaxError);
    };
    match scalar {
        '[' | '{' => {
            // The refusal happens BEFORE the container is allocated, so a
            // document at the limit is turned down rather than truncated.
            if depth_limit.is_some_and(|limit| depth >= limit) {
                return Err(MachineError::JsonLimitError);
            }
            decode_container(cursor, scalar, depth_limit, depth + 1)
        }
        '"' => decode_text(cursor),
        _ => decode_scalar(cursor),
    }
}

fn decode_container(
    cursor: &mut std::iter::Peekable<std::str::Chars<'_>>,
    opening: char,
    depth_limit: Option<usize>,
    depth: usize,
) -> Result<Value, MachineError> {
    let closing = if opening == '[' { ']' } else { '}' };
    cursor.next();
    let mut values = Vec::new();
    let mut entries = Vec::new();
    loop {
        while cursor.peek().is_some_and(|scalar| scalar.is_whitespace()) {
            cursor.next();
        }
        if cursor.peek() == Some(&closing) {
            cursor.next();
            break;
        }
        if closing == ']' {
            values.push(decode_json(cursor, depth_limit, depth)?);
        } else {
            let key = decode_json(cursor, depth_limit, depth)?;
            while cursor.peek().is_some_and(|scalar| scalar.is_whitespace()) {
                cursor.next();
            }
            if cursor.next() != Some(':') {
                return Err(MachineError::JsonSyntaxError);
            }
            let held = decode_json(cursor, depth_limit, depth)?;
            // The text PARSED - it is the object it describes that is
            // refused, so this is a duplicate-name failure rather than a
            // syntax one.
            if entries.iter().any(|(existing, _)| *existing == key) {
                return Err(MachineError::JsonDuplicateNameError);
            }
            entries.push((key, held));
        }
        while cursor.peek().is_some_and(|scalar| scalar.is_whitespace()) {
            cursor.next();
        }
        if cursor.peek() == Some(&',') {
            cursor.next();
        }
    }
    if closing == ']' {
        Ok(Value::Array(ArrayRef::new(values)))
    } else {
        Ok(Value::Hash(HashRef::new(entries)))
    }
}

fn decode_text(
    cursor: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<Value, MachineError> {
    cursor.next();
    let mut text = String::new();
    loop {
        let Some(scalar) = cursor.next() else {
            return Err(MachineError::JsonSyntaxError);
        };
        match scalar {
            '"' => break,
            '\\' => match cursor.next() {
                Some('n') => text.push('\n'),
                Some('t') => text.push('\t'),
                Some('r') => text.push('\r'),
                Some(other) => text.push(other),
                None => return Err(MachineError::JsonSyntaxError),
            },
            other => text.push(other),
        }
    }
    Ok(Value::Text(text))
}

fn decode_scalar(
    cursor: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<Value, MachineError> {
    let mut token = String::new();
    while let Some(&scalar) = cursor.peek() {
        if scalar.is_whitespace() || matches!(scalar, ',' | ']' | '}' | ':') {
            break;
        }
        token.push(scalar);
        cursor.next();
    }
    match token.as_str() {
        "null" => Ok(Value::Nil),
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        _ => token
            .parse()
            .map(Value::Integer)
            .map_err(|_| MachineError::JsonSyntaxError),
    }
}
