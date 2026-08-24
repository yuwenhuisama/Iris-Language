//! Execution backends and the observations used to compare them.
//!
//! `IRIS-V1-TRACE-C011` and the chapter 01 scope table put the Rust runtime,
//! VM, GC and JIT OUT OF SCOPE for this specification wave, so nothing here is
//! mandated by a frozen clause. What the specification does constrain is the
//! observations a backend produces: `IRIS-V1-CONFORMANCE-C068` requires a
//! differential checker to compare ONLY semantic observations named by a row's
//! `expect`, and forbids comparing implementation logs, heap addresses, object
//! pointer values, timing, or private bytecode layout.
//!
//! That constraint is why comparison happens over an [`Observation`] rather
//! than over runtime values directly. A runtime value carries identity - an
//! `ObjectId`, an `Arc` address, a content version - which two backends have no
//! reason to agree on and which C068 forbids comparing. Rendering to a
//! semantic form first makes an agreement check meaningful instead of an
//! accidental assertion about representation.

use crate::EvaluationError;

/// What a backend observably produced for one program.
///
/// This is deliberately a STRING rendering rather than a runtime value: it is
/// the semantic content C068 permits comparing, with identity stripped out.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Observation {
    /// The program completed with this rendered value.
    Value(String),
    /// The program failed with this rendered error.
    Error(String),
}

impl Observation {
    /// Renders an evaluation outcome as a comparable observation.
    #[must_use]
    pub fn of(outcome: Result<iris_runtime::Value, EvaluationError>) -> Self {
        match outcome {
            Ok(value) => Self::Value(render_value(&value)),
            Err(error) => Self::Error(format!("{error:?}")),
        }
    }
}

/// One way of executing Iris source.
///
/// A differential row asserts that every backend agrees, so the trait exists to
/// make "every backend" enumerable rather than hard-coded. A backend that
/// cannot execute a given program answers [`Support::Unsupported`], which is
/// NOT agreement: an incomplete backend must not silently pass a row it never
/// ran.
pub trait Backend {
    /// The backend's name, for reporting a disagreement.
    fn name(&self) -> &'static str;

    /// Executes `source`, or declines when the backend cannot run it.
    fn execute(&self, source: &str) -> Support;
}

/// Whether a backend ran a program, and what it saw.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Support {
    /// The backend ran the program.
    Ran(Observation),
    /// The backend cannot run this program yet, with a reason.
    ///
    /// A partially built backend is expected to answer this for most programs.
    /// Reporting it separately keeps a differential row honest: a row is only
    /// satisfied when at least two backends actually RAN it and agreed.
    Unsupported(String),
}

/// The reference backend: the tree-walking evaluator.
#[derive(Clone, Copy, Debug, Default)]
pub struct Interpreter;

impl Backend for Interpreter {
    fn name(&self) -> &'static str {
        "interpreter"
    }

    fn execute(&self, source: &str) -> Support {
        Support::Ran(Observation::of(crate::evaluate(source)))
    }
}

/// The bytecode backend.
///
/// It is deliberately PARTIAL and DECLINES any construct it does not fully
/// cover, rather than approximating one. A backend that guessed would make a
/// differential row agree for the wrong reason, which is worse than leaving
/// the row held.
#[derive(Clone, Copy, Debug, Default)]
pub struct Bytecode;

impl Backend for Bytecode {
    fn name(&self) -> &'static str {
        "bytecode"
    }

    fn execute(&self, source: &str) -> Support {
        match iris_vm::compile(source) {
            Err(declined) => Support::Unsupported(declined.construct),
            Ok(program) => match iris_vm::run(&program) {
                Ok(value) => Support::Ran(Observation::Value(render_value(&value))),
                // A kernel failure is a real observation: a row may assert
                // that both backends REJECT a program the same way.
                Err(iris_vm::MachineError::Kernel(error)) => Support::Ran(Observation::Error(
                    format!("{:?}", EvaluationError::Runtime(error)),
                )),
                Err(iris_vm::MachineError::Construction(error)) => Support::Ran(
                    Observation::Error(format!("{:?}", EvaluationError::Construction(error))),
                ),
                Err(iris_vm::MachineError::NameError) => Support::Ran(Observation::Error(format!(
                    "{:?}",
                    EvaluationError::NameError
                ))),
                Err(iris_vm::MachineError::IndexError) => Support::Ran(Observation::Error(
                    format!("{:?}", EvaluationError::IndexError),
                )),
                Err(iris_vm::MachineError::ConcurrentModification) => Support::Ran(
                    Observation::Error(format!("{:?}", EvaluationError::ConcurrentModification)),
                ),
                Err(iris_vm::MachineError::IteratorState) => Support::Ran(Observation::Error(
                    format!("{:?}", EvaluationError::IteratorState),
                )),
                Err(iris_vm::MachineError::TypeContractError) => Support::Ran(Observation::Error(
                    format!("{:?}", EvaluationError::TypeContractError),
                )),
                Err(iris_vm::MachineError::ReflectionAccess) => Support::Ran(Observation::Error(
                    format!("{:?}", EvaluationError::ReflectionAccess),
                )),
                Err(iris_vm::MachineError::JsonSyntaxError) => Support::Ran(Observation::Error(
                    format!("{:?}", EvaluationError::JsonSyntaxError),
                )),
                Err(iris_vm::MachineError::SerializationError) => Support::Ran(Observation::Error(
                    format!("{:?}", EvaluationError::SerializationError),
                )),
                Err(iris_vm::MachineError::MessageNotFound {
                    receiver_class,
                    selector,
                }) => Support::Ran(Observation::Error(format!(
                    "{:?}",
                    EvaluationError::MessageNotFound {
                        receiver_class,
                        selector,
                    }
                ))),
                Err(iris_vm::MachineError::Raised(propagation)) => Support::Ran(
                    Observation::Error(format!("{:?}", EvaluationError::Raised(propagation.0))),
                ),
                // A machine defect is not a program observation.
                Err(defect) => Support::Unsupported(format!("machine defect: {defect:?}")),
            },
        }
    }
}

/// The outcome of running one program across every backend.
#[derive(Clone, Debug)]
pub enum Agreement {
    /// Two or more backends ran the program and produced the same observation.
    Agreed {
        /// The backends that ran it.
        backends: Vec<&'static str>,
        /// The observation they share.
        observation: Observation,
    },
    /// Backends ran the program and did NOT produce the same observation.
    Disagreed {
        /// Each backend's observation, in registration order.
        observations: Vec<(&'static str, Observation)>,
    },
    /// Fewer than two backends could run the program.
    ///
    /// This is a HELD row rather than a passing one. A single backend agreeing
    /// with itself observes nothing.
    Insufficient {
        /// The backends that ran it, if any.
        ran: Vec<&'static str>,
        /// Why each remaining backend declined.
        declined: Vec<(&'static str, String)>,
    },
}

/// Runs `source` on every backend and reports whether they agree.
#[must_use]
pub fn compare_backends(source: &str, backends: &[&dyn Backend]) -> Agreement {
    let mut observations = Vec::new();
    let mut declined = Vec::new();
    for backend in backends {
        match backend.execute(source) {
            Support::Ran(observation) => observations.push((backend.name(), observation)),
            Support::Unsupported(reason) => declined.push((backend.name(), reason)),
        }
    }
    if observations.len() < 2 {
        return Agreement::Insufficient {
            ran: observations.into_iter().map(|(name, _)| name).collect(),
            declined,
        };
    }
    let first = &observations[0].1;
    if observations
        .iter()
        .all(|(_, observation)| observation == first)
    {
        Agreement::Agreed {
            backends: observations.iter().map(|(name, _)| *name).collect(),
            observation: first.clone(),
        }
    } else {
        Agreement::Disagreed { observations }
    }
}

/// Renders a value as semantic content, with identity stripped.
///
/// C068 forbids comparing heap addresses and object pointer values, so an
/// identity-bearing value renders as its KIND. Two backends allocate
/// independently and would otherwise disagree for reasons the clause says are
/// not semantic.
fn render_value(value: &iris_runtime::Value) -> String {
    use iris_runtime::Value;
    match value {
        Value::Nil => "nil".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::Integer(number) => number.decimal_text(),
        // A float renders through its BITS, since `IRIS-V1-RUNTIME-V066`
        // compares exact IEEE-754 results and a decimal rendering would hide a
        // one-ulp disagreement between backends.
        Value::Float32(number) => format!("f32:{:#x}", number.to_bits()),
        Value::Float64(number) => format!("f64:{:#x}", number.to_bits()),
        Value::Text(text) => format!("{text:?}"),
        Value::MutableString(text) => format!("m{:?}", text.text()),
        Value::Symbol(name) => format!(":{name}"),
        Value::Array(values) => {
            let rendered: Vec<String> = values.elements().iter().map(render_value).collect();
            format!("[{}]", rendered.join(", "))
        }
        Value::ReadonlyArray(values) | Value::Tuple(values) => {
            let rendered: Vec<String> = values.iter().map(render_value).collect();
            format!("[{}]", rendered.join(", "))
        }
        Value::Hash(entries) => {
            let rendered: Vec<String> = entries
                .entries()
                .iter()
                .map(|(key, entry)| format!("{}: {}", render_value(key), render_value(entry)))
                .collect();
            format!("{{{}}}", rendered.join(", "))
        }
        Value::Bytes(bytes) => format!("bytes:{}", hex(bytes)),
        Value::ByteArray(bytes) => format!("byte_array:{}", hex(&bytes.bytes())),
        Value::IterationDone => "Iteration.done".to_owned(),
        Value::IterationYield(payload) => format!("Iteration.yield({})", render_value(payload)),
        Value::KeywordArgument(name, inner) => format!("{name}: {}", render_value(inner)),
        // Identity-bearing kinds render as their KIND alone.
        other => format!("<{}>", kind_of(other)),
    }
}

fn kind_of(value: &iris_runtime::Value) -> &'static str {
    use iris_runtime::Value;
    match value {
        Value::Object(_) => "object",
        Value::Class(_) => "class",
        Value::Closure(_) => "closure",
        Value::Method(_) | Value::BoundMethod(_) => "method",
        Value::Contract(_) | Value::ContractView(..) => "contract",
        Value::Type(..) | Value::ComposedType(_) => "type",
        Value::Range(_) => "range",
        Value::Regex(_) => "regex",
        Value::Match(_) => "match",
        Value::Task(_) => "task",
        Value::Generator(_) => "generator",
        Value::Gate(_) => "gate",
        Value::Library(_) => "library",
        Value::ArrayIterator(_) | Value::HashIterator(_) | Value::ByteIterator(_) => "iterator",
        Value::NativeResource(_) => "native-resource",
        Value::Transformation { .. } => "transformation",
        Value::ExceptionContext(..) => "exception-context",
        Value::StackFrame(..) | Value::RaiseSite(_) | Value::SourceLocation(..) => "trace",
        _ => "value",
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A backend that answers a fixed observation, for testing the harness.
    struct Fixed(&'static str, Observation);

    impl Backend for Fixed {
        fn name(&self) -> &'static str {
            self.0
        }
        fn execute(&self, _source: &str) -> Support {
            Support::Ran(self.1.clone())
        }
    }

    /// A backend that runs nothing, standing in for a partially built VM.
    struct Absent(&'static str);

    impl Backend for Absent {
        fn name(&self) -> &'static str {
            self.0
        }
        fn execute(&self, _source: &str) -> Support {
            Support::Unsupported("not implemented".to_owned())
        }
    }

    /// ONE backend cannot satisfy a differential row. A backend agreeing with
    /// itself observes nothing, so this must be reported as insufficient
    /// rather than as agreement - otherwise every differential row would pass
    /// the moment a single backend existed.
    #[test]
    fn a_lone_backend_is_insufficient_rather_than_agreement() {
        let interpreter = Interpreter;
        let backends: Vec<&dyn Backend> = vec![&interpreter];

        let agreement = compare_backends("1 + 1", &backends);

        let Agreement::Insufficient { ran, declined } = agreement else {
            unreachable!("one backend cannot agree with anything")
        };
        assert_eq!(ran, vec!["interpreter"]);
        assert!(declined.is_empty());
    }

    /// A backend that DECLINES is not agreement either, however many decline.
    #[test]
    fn declining_backends_never_count_as_agreement() {
        let interpreter = Interpreter;
        let absent = Absent("vm");
        let backends: Vec<&dyn Backend> = vec![&interpreter, &absent];

        let agreement = compare_backends("1 + 1", &backends);

        let Agreement::Insufficient { ran, declined } = agreement else {
            unreachable!("a declining backend ran nothing")
        };
        assert_eq!(ran, vec!["interpreter"]);
        assert_eq!(declined, vec![("vm", "not implemented".to_owned())]);
    }

    #[test]
    fn two_backends_producing_the_same_observation_agree() {
        let first = Fixed("a", Observation::Value("2".to_owned()));
        let second = Fixed("b", Observation::Value("2".to_owned()));
        let backends: Vec<&dyn Backend> = vec![&first, &second];

        let agreement = compare_backends("1 + 1", &backends);

        let Agreement::Agreed {
            backends,
            observation,
        } = agreement
        else {
            unreachable!("identical observations agree")
        };
        assert_eq!(backends, vec!["a", "b"]);
        assert_eq!(observation, Observation::Value("2".to_owned()));
    }

    /// The case the harness exists to catch.
    #[test]
    fn differing_observations_are_reported_as_disagreement() {
        let first = Fixed("a", Observation::Value("2".to_owned()));
        let second = Fixed("b", Observation::Value("3".to_owned()));
        let backends: Vec<&dyn Backend> = vec![&first, &second];

        let agreement = compare_backends("1 + 1", &backends);

        let Agreement::Disagreed { observations } = agreement else {
            unreachable!("2 and 3 are not the same observation")
        };
        assert_eq!(observations.len(), 2);
    }

    /// C068 forbids comparing object pointer values, so identity must not
    /// leak into an observation: two runs allocate independently and would
    /// disagree for a reason the clause says is not semantic.
    #[test]
    fn identity_does_not_leak_into_an_observation() {
        let interpreter = Interpreter;
        let source = "class A { } module M { public fun run() -> Object { A.new() } } M.run()";

        let (Support::Ran(first), Support::Ran(second)) =
            (interpreter.execute(source), interpreter.execute(source))
        else {
            unreachable!("the interpreter runs this")
        };

        assert_eq!(first, second);
        assert_eq!(first, Observation::Value("<object>".to_owned()));
    }

    /// A float compares by BITS: a decimal rendering would hide a one-ulp
    /// disagreement, which is exactly what RUNTIME-V066 is about.
    #[test]
    fn floats_are_observed_by_their_bits() {
        let interpreter = Interpreter;

        let Support::Ran(observation) = interpreter.execute("1.0") else {
            unreachable!("the interpreter runs a literal")
        };

        assert_eq!(
            observation,
            Observation::Value(format!("f64:{:#x}", 1.0_f64.to_bits()))
        );
    }
}

#[cfg(test)]
mod differential_tests {
    use super::*;

    fn both() -> (Interpreter, Bytecode) {
        (Interpreter, Bytecode)
    }

    fn execute_with_grants(
        source: &str,
        grants: Vec<(String, String)>,
    ) -> (Observation, Observation) {
        let packages = vec![(
            "app".to_owned(),
            vec![("main.iris".to_owned(), source.to_owned())],
        )];
        let interpreted =
            crate::load_package_tree_with_grants(&packages, grants.clone(), Some("M.r()"))
                .map(|(_, value)| value.unwrap_or(iris_runtime::Value::Nil));
        let interpreter = Observation::of(interpreted);
        let bytecode_source = format!("{source} M.r()");
        let bytecode = match iris_vm::compile(&bytecode_source) {
            Ok(program) => match iris_vm::Machine::new() {
                Ok(mut machine) => {
                    machine.enter_reflection_grants(grants);
                    match machine.execute(&program) {
                        Ok(value) => Observation::Value(render_value(&value)),
                        Err(iris_vm::MachineError::ReflectionAccess) => {
                            Observation::Error(format!("{:?}", EvaluationError::ReflectionAccess))
                        }
                        Err(error) => Observation::Error(format!("machine defect: {error:?}")),
                    }
                }
                Err(error) => Observation::of(Err(EvaluationError::Runtime(error))),
            },
            Err(_) => Observation::Error("bytecode declined reflection fixture".to_owned()),
        };
        (interpreter, bytecode)
    }

    #[test]
    fn backends_agree_on_reflecting_an_existing_class_method() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "class A { public fun value() -> Integer { 7 } } module M { public fun r() -> Object { Reflection::Class.method(A, :value) } } M.r()";

        let agreement = compare_backends(source, &backends);

        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends reflect a declared method")
        };
        assert_eq!(observation, Observation::Value("<method>".to_owned()));
    }

    #[test]
    fn backends_agree_that_reflecting_an_absent_class_method_answers_nil() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "class A { public fun value() -> Integer { 7 } } module M { public fun r() -> Object { Reflection::Class.method(A, :missing) } } M.r()";

        let agreement = compare_backends(source, &backends);

        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends distinguish an absent method")
        };
        assert_eq!(observation, Observation::Value("nil".to_owned()));
    }

    #[test]
    fn backends_agree_on_raw_ivar_round_trip_and_absent_read() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "class A { } module M { public fun r() -> Object { let a = A.new(); let stored = Reflection::Object.set_ivar(a, :@value, 9); [stored, Reflection::Object.get_ivar(a, :@value), Reflection::Object.get_ivar(a, :@absent)] } } M.r()";

        let agreement = compare_backends(source, &backends);

        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends preserve raw instance state")
        };
        assert_eq!(observation, Observation::Value("[9, 9, nil]".to_owned()));
    }

    #[test]
    fn reflection_object_values_remain_ordinary_downstream_values() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "class A { } module M { public fun r() -> Object { let a = A.new(); let stored = Reflection::Object.set_ivar(a, :@value, [1]); let read = Reflection::Object.get_ivar(a, :@value); stored.push(2); read.push(3); [read, Reflection::Object.get_ivar(a, :@missing)] } } M.r()";

        let agreement = compare_backends(source, &backends);

        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("reflected ivar values keep their ordinary value protocols: {agreement:?}")
        };
        assert_eq!(
            observation,
            Observation::Value("[[1, 2, 3], nil]".to_owned())
        );
    }

    #[test]
    fn backends_agree_on_granted_and_ungranted_reflection_access() {
        let source = "class A { } module M { public fun r() -> Object { let a = A.new(); try { Reflection::Object.set_ivar(a, :@value, 9); Reflection::Object.get_ivar(a, :@value) } catch e { e } } }";

        let granted = execute_with_grants(
            source,
            vec![
                ("reflection.inspect".to_owned(), "A".to_owned()),
                ("reflection.mutate".to_owned(), "A".to_owned()),
            ],
        );
        let denied = execute_with_grants(
            source,
            vec![("reflection.inspect".to_owned(), "Other".to_owned())],
        );

        assert_eq!(granted.0, granted.1);
        assert_eq!(granted.0, Observation::Value("9".to_owned()));
        assert_eq!(denied.0, denied.1);
        assert_eq!(
            denied.0,
            Observation::Value(":ReflectionAccessError".to_owned())
        );
    }

    #[test]
    fn requested_method_differential_evidence() {
        let (interpreter, bytecode) = both();
        let programs = [
            "class A { public fun m() -> Integer { 1 } } module M { public fun r() -> Object { Reflection::Class.method(A, :m).selector } } M.r()",
            "class A { public fun m() -> Integer { 1 } } module M { public fun r() -> Object { Reflection::Class.method(A, :m).owner } } M.r()",
            "class A { public fun m() -> Integer { 1 } } module M { public fun r() -> Object { Reflection::Class.method(A, :m).visibility } } M.r()",
            "class A { public fun m() -> Integer { 7 } } module M { public fun r() -> Object { Reflection::Class.method(A, :m).bind(A.new()).call() } } M.r()",
        ];

        for source in programs {
            let interpreted = interpreter.execute(source);
            let compiled = bytecode.execute(source);
            eprintln!("METHOD DIFF interpreter={interpreted:?} bytecode={compiled:?}");
            assert_eq!(interpreted, compiled);
        }
        let denial_source = "class A { } module M { public fun r() -> Object { try { Reflection::Object.get_ivar(A.new(), :@value) } catch e { e } } }";
        let denied = execute_with_grants(
            denial_source,
            vec![("reflection.inspect".to_owned(), "Other".to_owned())],
        );
        eprintln!(
            "DENIAL DIFF interpreter={:?} bytecode={:?}",
            denied.0, denied.1
        );
        assert_eq!(denied.0, denied.1);
    }

    #[test]
    fn backends_agree_on_method_metadata_and_bound_invocation() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "class A { public fun m() -> Integer { 7 } } module M { public fun r() -> Object { let method = Reflection::Class.method(A, :m); [method.selector, method.owner same? A, method.visibility, method.parameters, method.return_type, method.source[3], method.bind(A.new()).call()] } } M.r()";

        let agreement = compare_backends(source, &backends);

        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends expose and bind the reflected Method: {agreement:?}")
        };
        assert_eq!(
            observation,
            Observation::Value("[:m, true, :public, [], :Integer, :static, 7]".to_owned())
        );
    }

    #[test]
    fn bytecode_declines_unimplemented_method_view_members_before_execution() {
        let bytecode = Bytecode;
        let prefix = "class A { public fun m() -> Integer { 7 } } module M { public fun r() -> Object { let method = Reflection::Class.method(A, :m); ";

        for (expression, expected) in [
            ("method.signature", "Method.signature"),
            ("method.package", "Method.package"),
            ("method.call()", "Method.call"),
        ] {
            let source = format!("{prefix}{expression} }} }} M.r()");
            let Support::Unsupported(reason) = bytecode.execute(&source) else {
                unreachable!("an unimplemented Method member must be declined")
            };
            assert_eq!(reason, expected);
        }
    }

    /// `IRIS-V1-RUNTIME-V052`: an arbitrary-precision Integer survives a
    /// round trip with NO representation Type split, and both backends see it.
    #[test]
    fn backends_agree_on_arbitrary_precision_integer_arithmetic() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let agreement = compare_backends("((2 ** 200) + 1) - (2 ** 200)", &backends);

        let Agreement::Agreed {
            backends,
            observation,
        } = agreement
        else {
            unreachable!("both backends run integer arithmetic")
        };
        assert_eq!(backends, vec!["interpreter", "bytecode"]);
        assert_eq!(observation, Observation::Value("1".to_owned()));
    }

    /// The bytecode backend must DECLINE what it does not cover, rather than
    /// approximate it. Agreement reached by guessing would be worthless.
    #[test]
    fn the_bytecode_backend_declines_uncovered_constructs() {
        let bytecode = Bytecode;

        // `.map(...)` is a CALL whose receiver is an array; the call is the
        // outer construct, so that is what the backend names. The reason
        // identifies WHICH receiver shape stopped it, so a later change that
        // covers array receivers cannot leave this passing for the old cause.
        let Support::Unsupported(reason) = bytecode.execute("nope.foo()") else {
            unreachable!("an unbound receiver must remain declined")
        };
        assert_eq!(reason, "call unbound receiver");

        let Support::Unsupported(reason) = bytecode.execute("for [x] in [[1]] { x }") else {
            unreachable!("this backend covers no destructuring iteration yet")
        };
        assert_eq!(reason, "statement for");

        // A declined construct leaves the comparison INSUFFICIENT, so a row
        // relying on it stays held rather than passing on one backend.
        let interpreter = Interpreter;
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let Agreement::Insufficient { ran, declined } = compare_backends("nope.foo()", &backends)
        else {
            unreachable!("only one backend ran it")
        };
        assert_eq!(ran, vec!["interpreter"]);
        assert_eq!(declined.len(), 1);
    }

    #[test]
    fn backends_agree_on_authored_array_methods() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, expected, control) in [
            (
                "[1, 2].map({ |x|; x * 2 })",
                "[2, 4]",
                "[1, 2].map({ |x|; x * 3 })",
            ),
            (
                "[1, 2].each({ |x|; x }).length()",
                "2",
                "[1].each({ |x|; x }).length()",
            ),
            (
                "[1, 2, 3].select({ |x|; x > 1 })",
                "[2, 3]",
                "[1, 2, 3].select({ |x|; x > 2 })",
            ),
            (
                "[1, 2, 3].reduce(0, { |a, x|; a + x })",
                "6",
                "[1, 2, 3].reduce(1, { |a, x|; a + x })",
            ),
            ("[1, 2].length()", "2", "[1].length()"),
            (
                "let a = [1]; let b = a; a.push(2); b.pop()",
                "2",
                "let a = [1]; let b = a; a.push(3); b.pop()",
            ),
            ("[1, 2].join(\"-\")", "\"1-2\"", "[1, 2].join(\":\")"),
            (
                "[1, 2, 3].find({ |x|; x > 1 })",
                "2",
                "[1, 2, 3].find({ |x|; x > 2 })",
            ),
            ("[1, 2, 3].count()", "3", "[1, 2].count()"),
            (
                "[1, 2, 3].count({ |x|; x > 1 })",
                "2",
                "[1, 2, 3].count({ |x|; x > 2 })",
            ),
            ("[1, 2, 3].sum()", "6", "[1, 2].sum()"),
            ("[3, 1, 2].min()", "1", "[3, 2].min()"),
            ("[3, 1, 2].max()", "3", "[1, 2].max()"),
            ("[3, 1, 2].sort()", "[1, 2, 3]", "[2, 1].sort()"),
            ("[1, 2, 3].include?(2)", "true", "[1, 2, 3].include?(4)"),
            ("[1, 2, 3].index_of(2)", "1", "[1, 2, 3].index_of(3)"),
            ("[1, 2].concat([3, 4])", "[1, 2, 3, 4]", "[1].concat([3])"),
            ("[1, 2, 3].slice(1, 2)", "[2, 3]", "[1, 2, 3].slice(0, 2)"),
            ("[1, 2, 3].take(2)", "[1, 2]", "[1, 2, 3].take(1)"),
            ("[1, 2, 3].drop(2)", "[3]", "[1, 2, 3].drop(1)"),
            ("[1, 1, 2].uniq()", "[1, 2]", "[1, 3, 3].uniq()"),
            ("[1, [2, [3]]].flatten()", "[1, 2, 3]", "[[1], 2].flatten()"),
            (
                "[1, 2].all?({ |x|; x > 0 })",
                "true",
                "[0, 1].all?({ |x|; x > 0 })",
            ),
            (
                "[0, 2].any?({ |x|; x > 1 })",
                "true",
                "[0, 1].any?({ |x|; x > 1 })",
            ),
            (
                "[1, 2].each_with_index({ |x, i|; x + i })",
                "[1, 2]",
                "[1].each_with_index({ |x, i|; x + i })",
            ),
        ] {
            let source =
                format!("module M {{ public fun r() -> Object {{ {expression} }} }} M.r()");
            let control = format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()");
            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends cover {expression}")
            };
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{expression}"
            );
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = compare_backends(&control, &backends)
            else {
                unreachable!("both backends cover the negative control for {expression}")
            };
            assert_ne!(control_observation, observation, "{expression}");
        }
    }

    #[test]
    fn backends_agree_on_authored_hash_and_string_methods() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, control) in [
            ("%{ 1: 2 }.length()", "%{ 1: 2, 3: 4 }.length()"),
            ("\"åb\".length()", "\"åbc\".length()"),
            ("\" a \".trim()", "\" b \".trim()"),
            ("\"a,b\".split(\",\")", "\"a:b\".split(\",\")"),
            (
                "let h = %{ 1: 2 }.merge(%{ 1: 3 }); h[1]",
                "let h = %{ 1: 2 }.merge(%{ 1: 4 }); h[1]",
            ),
            ("(5).to_string()", "(6).to_string()"),
            ("true.to_string()", "false.to_string()"),
            ("nil.to_string()", "(5).to_string()"),
            (":iris.to_string()", ":other.to_string()"),
        ] {
            let source =
                format!("module M {{ public fun r() -> Object {{ {expression} }} }} M.r()");
            let negative = format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()");
            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends cover {expression}")
            };
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = compare_backends(&negative, &backends)
            else {
                unreachable!("both backends cover the negative control for {expression}")
            };
            assert_ne!(control_observation, observation, "{expression}");
        }
    }

    #[test]
    fn backends_agree_on_json_values_and_produced_collections() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, expected, control) in [
            (
                "JSON.decode(\"[1,{\\\"a\\\":[true,false,null]}]\")",
                "[1, {\"a\": [true, false, nil]}]",
                "JSON.decode(\"[2,{\\\"a\\\":[true,false,null]}]\")",
            ),
            ("JSON.decode(\"[]\")", "[]", "JSON.decode(\"[1]\")"),
            (
                "JSON.decode(\"{}\")",
                "{}",
                "JSON.decode(\"{\\\"a\\\":1}\")",
            ),
            (
                "let h = JSON.decode(\"{\\\"a\\\":1,\\\"b\\\":2}\"); h[\"a\"]",
                "1",
                "let h = JSON.decode(\"{\\\"a\\\":1,\\\"b\\\":2}\"); h[\"b\"]",
            ),
            (
                "mut total = 0; for pair in JSON.decode(\"{\\\"a\\\":1,\\\"b\\\":2}\") { total = total + pair[1] }; total",
                "3",
                "mut total = 0; for pair in JSON.decode(\"{\\\"a\\\":1,\\\"b\\\":3}\") { total = total + pair[1] }; total",
            ),
            (
                "JSON.encode([1,true,false,nil,%{ \"a\": [2] }])",
                "\"[1,true,false,null,{\\\"a\\\":[2]}]\"",
                "JSON.encode([2,true,false,nil,%{ \"a\": [2] }])",
            ),
        ] {
            let source =
                format!("module M {{ public fun r() -> Object {{ {expression} }} }} M.r()");
            let negative = format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()");
            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends cover JSON in {expression}")
            };
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{expression}"
            );
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = compare_backends(&negative, &backends)
            else {
                unreachable!("both backends cover the JSON negative control for {expression}")
            };
            assert_ne!(control_observation, observation, "{expression}");
        }
    }

    #[test]
    fn backends_agree_on_json_float_refusals_and_catchable_syntax_errors() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, control) in [
            (
                "try { JSON.decode(\"[\") } catch JSONSyntaxError { :caught }",
                "try { JSON.decode(\"[]\") } catch JSONSyntaxError { :caught }",
            ),
            (
                "try { JSON.decode(\"1.5\") } catch JSONSyntaxError { :caught }",
                "try { JSON.decode(\"1\") } catch JSONSyntaxError { :caught }",
            ),
            (
                "try { JSON.encode(1.5) } catch SerializationError { :caught }",
                "try { JSON.encode(1) } catch SerializationError { :caught }",
            ),
        ] {
            let source =
                format!("module M {{ public fun r() -> Object {{ {expression} }} }} M.r()");
            let negative = format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()");
            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends cover JSON failure behavior in {expression}")
            };
            assert_eq!(
                observation,
                Observation::Value(":caught".to_owned()),
                "{expression}"
            );
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = compare_backends(&negative, &backends)
            else {
                unreachable!("both backends cover the JSON failure negative control")
            };
            assert_ne!(control_observation, observation, "{expression}");
        }
    }

    #[test]
    fn backends_interpolate_strings_in_source_order() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, expected, control) in [
            ("let n = 5; \"n=${n}\"", "\"n=5\"", "let n = 6; \"n=${n}\""),
            ("\"v=${1 + 2}\"", "\"v=3\"", "\"v=${1 + 3}\""),
            (
                "C.new(); \"${C.new()}${C.new()}\"",
                "\"12\"",
                "C.new(); \"${C.new()}\"",
            ),
            ("\"cost $5\"", "\"cost $5\"", "\"cost $6\""),
        ] {
            let declaration = if expression.starts_with("C.new") {
                "class C { shared mut @@n: Integer = 0 public fun to_string() -> String { @@n = @@n + 1; @@n.to_string() } } "
            } else {
                ""
            };
            let source = format!(
                "{declaration}module M {{ public fun r() -> Object {{ {expression} }} }} M.r()"
            );
            let negative = format!(
                "{declaration}module M {{ public fun r() -> Object {{ {control} }} }} M.r()"
            );
            let agreement = compare_backends(&source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends cover interpolation in {expression}: {agreement:?}")
            };
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{expression}"
            );
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = compare_backends(&negative, &backends)
            else {
                unreachable!("both backends cover the negative control for {expression}")
            };
            assert_ne!(control_observation, observation, "{expression}");
        }
    }

    #[test]
    fn backends_reject_non_string_interpolation_conversion_results() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "class C { public fun to_string() -> Object { 7 } } module M { public fun r() -> Object { \"bad=${C.new()}\" } } M.r()";
        let control = "class C { public fun to_string() -> Object { \"7\" } } module M { public fun r() -> Object { \"ok=${C.new()}\" } } M.r()";

        let agreement = compare_backends(source, &backends);
        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!(
                "both backends enforce the interpolation conversion contract: {agreement:?}"
            )
        };
        assert_eq!(
            observation,
            Observation::Error(format!("{:?}", EvaluationError::TypeContractError))
        );
        let Agreement::Agreed {
            observation: control_observation,
            ..
        } = compare_backends(control, &backends)
        else {
            unreachable!("both backends accept the String-returning negative control")
        };
        assert_eq!(
            control_observation,
            Observation::Value("\"ok=7\"".to_owned())
        );
        assert_ne!(control_observation, observation);
    }

    #[test]
    fn backends_concatenate_strings_through_dynamic_conversion() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, expected, control) in [
            ("\"a\" + \"b\"", "\"ab\"", "\"a\" + \"c\""),
            ("\"n=\" + 5", "\"n=5\"", "\"n=\" + 6"),
        ] {
            let source =
                format!("module M {{ public fun r() -> Object {{ {expression} }} }} M.r()");
            let negative = format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()");
            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends cover String + in {expression}")
            };
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{expression}"
            );
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = compare_backends(&negative, &backends)
            else {
                unreachable!("both backends cover the negative control for {expression}")
            };
            assert_ne!(control_observation, observation, "{expression}");
        }
    }

    #[test]
    fn backends_agree_on_the_covered_operator_surface() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, control) in [
            ("2 + 3", "2 + 4"),
            ("2.0 + 3.0", "2.0 + 4.0"),
            ("\"a\" + \"b\"", "\"a\" + \"c\""),
            ("6 & 3", "6 | 3"),
            ("2 < 3", "3 < 2"),
            ("true == false", "true == true"),
            ("nil == nil", "nil != nil"),
            ("-5", "-6"),
            ("+5", "+6"),
            ("~5", "~6"),
            ("false && 1", "true && 1"),
            ("true || 1", "false || 1"),
        ] {
            let source =
                format!("module M {{ public fun r() -> Object {{ {expression} }} }} M.r()");
            let negative = format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()");
            let agreement = compare_backends(&source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends cover {expression}: {agreement:?}")
            };
            let control_agreement = compare_backends(&negative, &backends);
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = control_agreement
            else {
                unreachable!(
                    "both backends cover the control for {expression}: {control_agreement:?}"
                )
            };
            assert_ne!(control_observation, observation, "{expression}");
        }
    }

    #[test]
    fn backends_agree_on_literal_match_expressions() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, expected, control) in [
            (
                "match 2 { 1 => :one, 2 => :two, _ => :other }",
                ":two",
                "match 3 { 1 => :one, 2 => :two, _ => :other }",
            ),
            (
                "match \"b\" { \"a\" => 1, \"b\" => { 2 }, else => 3 }",
                "2",
                "match \"c\" { \"a\" => 1, \"b\" => { 2 }, else => 3 }",
            ),
        ] {
            let source =
                format!("module M {{ public fun r() -> Object {{ {expression} }} }} M.r()");
            let negative = format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()");
            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends cover literal match {expression}")
            };
            assert_eq!(observation, Observation::Value(expected.to_owned()));
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = compare_backends(&negative, &backends)
            else {
                unreachable!("both backends cover the match negative control")
            };
            assert_ne!(control_observation, observation);
        }
    }

    #[test]
    fn backends_keep_array_size_absent() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "module M { public fun r() -> Object { [1].size() } } M.r()";
        let control = "module M { public fun r() -> Object { [1].length() } } M.r()";

        let agreement = compare_backends(source, &backends);
        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends reject the absent selector")
        };
        assert!(
            matches!(observation, Observation::Error(ref error) if error.contains("MessageNotFound"))
        );
        let Agreement::Agreed {
            observation: control_observation,
            ..
        } = compare_backends(control, &backends)
        else {
            unreachable!("both backends cover length")
        };
        assert_eq!(control_observation, Observation::Value("1".to_owned()));
    }

    /// Both backends dispatch arithmetic through the SAME kernel, so a shared
    /// rejection is a real shared observation rather than two guesses.
    #[test]
    fn backends_agree_on_a_rejected_program() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let Agreement::Agreed { observation, .. } = compare_backends("1 / 0", &backends) else {
            unreachable!("both backends reject division by zero")
        };
        let Observation::Error(_) = observation else {
            unreachable!("division by zero is an error observation")
        };
    }

    /// A typed catch must skip a non-matching handler inside a method frame;
    /// otherwise the bytecode backend can appear correct at top level while
    /// losing the exception register when control crosses a call boundary.
    #[test]
    fn backends_agree_when_a_filtered_catch_falls_through() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "module M { public fun r() -> Object { try { raise 1 } catch e: Symbol { 10 } catch e: Integer { e + 1 } } } M.r()";

        let agreement = compare_backends(source, &backends);
        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends run filtered catches in method frames")
        };

        assert_eq!(observation, Observation::Value("2".to_owned()));
        assert_ne!(observation, Observation::Value("10".to_owned()));
    }

    #[test]
    fn backends_agree_on_exception_context_values_and_unused_bindings() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, expected, control) in [
            (
                "try { raise :x } catch e, context { context.value }",
                ":x",
                "try { raise :y } catch e, context { context.value }",
            ),
            (
                "try { raise :x } catch e, context { e }",
                ":x",
                "try { raise :y } catch e, context { e }",
            ),
        ] {
            let source =
                format!("module M {{ public fun r() -> Object {{ {expression} }} }} M.r()");
            let negative = format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()");

            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends bind the exception context for {expression}")
            };
            assert_eq!(observation, Observation::Value(expected.to_owned()));
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = compare_backends(&negative, &backends)
            else {
                unreachable!("both backends run the negative control for {expression}")
            };
            assert_ne!(control_observation, observation);
        }
    }

    #[test]
    fn backends_agree_on_exception_context_causes() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        for (expression, expected, control) in [
            (
                "try { try { raise :first } catch e, first { raise :second } } catch e, second { second.cause.value }",
                ":first",
                "try { try { raise :other } catch e, first { raise :second } } catch e, second { second.cause.value }",
            ),
            (
                "try { try { raise :first } catch e, first { raise :second from first } } catch e, second { second.cause.value }",
                ":first",
                "try { try { raise :other } catch e, first { raise :second from first } } catch e, second { second.cause.value }",
            ),
            (
                "try { try { raise :first } catch e, first { raise :second from nil } } catch e, second { second.cause }",
                "nil",
                "try { try { raise :first } catch e, first { raise :second } } catch e, second { second.cause }",
            ),
        ] {
            let source =
                format!("module M {{ public fun r() -> Object {{ {expression} }} }} M.r()");
            let negative = format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()");

            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends preserve the cause for {expression}")
            };
            assert_eq!(observation, Observation::Value(expected.to_owned()));
            let Agreement::Agreed {
                observation: control_observation,
                ..
            } = compare_backends(&negative, &backends)
            else {
                unreachable!("both backends run the negative control for {expression}")
            };
            assert_ne!(control_observation, observation);
        }
    }

    #[test]
    fn backends_give_each_explicit_raise_a_distinct_context_identity() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "module M { public fun r() -> Object { try { try { raise :x } catch e, first { raise e } } catch e, second { second.same?(second.cause) } } } M.r()";
        let control = "module M { public fun r() -> Object { try { raise :x } catch e, context { context.same?(context) } } } M.r()";

        let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
            unreachable!("both backends distinguish propagation identities")
        };
        assert_eq!(observation, Observation::Value("false".to_owned()));
        let Agreement::Agreed {
            observation: control_observation,
            ..
        } = compare_backends(control, &backends)
        else {
            unreachable!("both backends run the identity negative control")
        };
        assert_eq!(control_observation, Observation::Value("true".to_owned()));
        assert_ne!(control_observation, observation);
    }

    #[test]
    fn backends_propagate_an_inner_context_to_an_outer_handler() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "module M { public fun r() -> Object { try { try { raise :inner } catch e: Integer, ignored { :wrong } } catch e, context { context.value } } } M.r()";
        let control = "module M { public fun r() -> Object { try { try { raise :other } catch e: Integer, ignored { :wrong } } catch e, context { context.value } } } M.r()";

        let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
            unreachable!("both backends propagate the inner context")
        };
        assert_eq!(observation, Observation::Value(":inner".to_owned()));
        let Agreement::Agreed {
            observation: control_observation,
            ..
        } = compare_backends(control, &backends)
        else {
            unreachable!("both backends run the nested negative control")
        };
        assert_ne!(control_observation, observation);
    }

    /// Float agreement is compared by BITS, which is what makes a one-ulp
    /// divergence between backends detectable at all.
    #[test]
    fn backends_agree_on_float_arithmetic_bit_for_bit() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let Agreement::Agreed { observation, .. } = compare_backends("0.1 + 0.2", &backends) else {
            unreachable!("both backends run float arithmetic")
        };
        assert_eq!(
            observation,
            Observation::Value(format!("f64:{:#x}", (0.1_f64 + 0.2_f64).to_bits()))
        );
    }

    /// `IRIS-V1-RUNTIME-V066`: the exact IEEE-754 result bits of a Float32
    /// multiply, which both backends must reach identically. Comparing by BITS
    /// is what makes a one-ulp divergence detectable at all.
    #[test]
    fn backends_agree_on_exact_float32_result_bits() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let Agreement::Agreed { observation, .. } = compare_backends(
            "(Float32.from_bits(0x3f800001) * Float32.from_bits(0x3f800001)).to_bits()",
            &backends,
        ) else {
            unreachable!("both backends run Float32 arithmetic")
        };

        // 0x3f800001 squared is 0x3f800002 under roundTiesToEven.
        assert_eq!(observation, Observation::Value("1065353218".to_owned()));
    }

    /// `C114` requires `from_bits(b).to_bits() == b` for EVERY interchange
    /// pattern including signaling NaN, which must not be quieted in transit.
    #[test]
    fn backends_agree_on_the_signaling_nan_round_trip() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let Agreement::Agreed { observation, .. } =
            compare_backends("Float32.from_bits(0x7f800001).to_bits()", &backends)
        else {
            unreachable!("both backends round-trip a signaling NaN")
        };
        assert_eq!(observation, Observation::Value("2139095041".to_owned()));
    }

    /// `C113` requires RangeError outside a width's range. The bytecode
    /// backend must answer the SAME error, not invent its own.
    #[test]
    fn backends_agree_on_an_out_of_range_from_bits() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for source in [
            "Float32.from_bits(-1).to_bits()",
            "Float32.from_bits(4294967296).to_bits()",
        ] {
            let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
                unreachable!("both backends reject an out-of-range width")
            };
            assert_eq!(
                observation,
                Observation::Error("Runtime(Numeric(Range))".to_owned())
            );
        }
    }

    /// `IRIS-V1-RUNTIME-V073`: the exact C146 public hashes agree across
    /// backends. The values are pinned against the NORMATIVE table in C146,
    /// so agreement on a wrong number would still fail.
    #[test]
    fn backends_agree_on_the_normative_public_hashes() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        // C146 rows V001, V008, V009, V010.
        for (source, expected) in [
            ("(0).hash()", "4379003086384345280"),
            ("Float64.from_bits(0).hash()", "4379003086384345280"),
            ("nil.hash()", "11850167709044604115"),
            ("false.hash()", "17921396551637717540"),
            ("true.hash()", "14186115676603356736"),
        ] {
            let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
                unreachable!("both backends compute a public hash")
            };
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    /// Every construct the bytecode backend covers must AGREE with the
    /// reference, not merely run. Coverage without agreement would be worse
    /// than no coverage: a differential row would then pass on a wrong answer
    /// both backends happened to share.
    #[test]
    fn backends_agree_across_every_covered_construct() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected) in [
            // Bindings, sequencing and slot reuse on rebinding.
            ("let a = 2; let b = 3; a * b", "6"),
            ("let x = 10; x - 4", "6"),
            ("let a = 1; let a = 2; a", "2"),
            ("let n = [1, 2]; n", "[1, 2]"),
            // Comparisons.
            ("1 < 2", "true"),
            ("5 >= 5", "true"),
            ("2 == 2", "true"),
            ("1 != 2", "true"),
            ("3 <=> 2", "1"),
            // Bitwise and shift.
            ("6 & 3", "2"),
            ("1 << 4", "16"),
        ] {
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends cover: {source}: {agreement:?}")
            };
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    fn assert_agreement(source: &str, expected: &str) {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let agreement = compare_backends(source, &backends);
        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends must run: {source}: {agreement:?}")
        };
        assert_eq!(
            observation,
            Observation::Value(expected.to_owned()),
            "{source}"
        );
        assert_ne!(
            observation,
            Observation::Value(format!("wrong:{expected}")),
            "{source}"
        );
    }

    #[test]
    fn backends_agree_on_iteration_protocol() {
        for (body, expected, control) in [
            (
                "class C { public fun iterator() -> Object { [7].iterator() } }; mut sum = 0; for x in C.new() { sum = sum + x }; sum",
                "7",
                "class C { public fun iterator() -> Object { [8].iterator() } }; mut sum = 0; for x in C.new() { sum = sum + x }; sum",
            ),
            (
                "let it = [1, 2].iterator(); [it.next().value, it.next().value, it.next().done?]",
                "[1, 2, true]",
                "let it = [1, 3].iterator(); [it.next().value, it.next().value, it.next().done?]",
            ),
            (
                "let it = [].iterator(); [it.close(), it.close(), it.next().done?]",
                "[nil, nil, true]",
                "let it = [1].iterator(); [it.close(), it.close(), it.next().yield?]",
            ),
            (
                "let it = [].iterator(); let a = it.next(); let b = it.next(); [a.done?, b.done?]",
                "[true, true]",
                "let it = [1].iterator(); let a = it.next(); let b = it.next(); [a.done?, b.done?]",
            ),
            (
                "class C { public fun iterator() -> Object { [7, 9].iterator() } }; mut seen = 0; for x in C.new() { seen = x; break }; seen",
                "7",
                "class C { public fun iterator() -> Object { [8, 9].iterator() } }; mut seen = 0; for x in C.new() { seen = x; break }; seen",
            ),
            (
                "mut sum = 0; for x in [1, 2] { sum = sum + x }; for pair in %{ :a: 3 } { sum = sum + pair[1] }; for x in (4 ..= 5) { sum = sum + x }; sum",
                "15",
                "mut sum = 0; for x in [1] { sum = sum + x }; for pair in %{ :b: 4 } { sum = sum + pair[1] }; for x in (6 ..= 6) { sum = sum + x }; sum",
            ),
        ] {
            let source = if body.starts_with("class C")
                && let Some((declaration, method_body)) = body.split_once("}; ")
            {
                format!(
                    "{declaration}}} module M {{ public fun r() -> Object {{ {method_body} }} }} M.r()"
                )
            } else {
                format!("module M {{ public fun r() -> Object {{ {body} }} }} M.r()")
            };
            let control = if control.starts_with("class C")
                && let Some((declaration, method_body)) = control.split_once("}; ")
            {
                format!(
                    "{declaration}}} module M {{ public fun r() -> Object {{ {method_body} }} }} M.r()"
                )
            } else {
                format!("module M {{ public fun r() -> Object {{ {control} }} }} M.r()")
            };
            assert_agreement(&source, expected);
            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let Agreement::Agreed { observation, .. } = compare_backends(&control, &backends)
            else {
                unreachable!("both backends must run the iteration negative control: {control}")
            };
            assert_ne!(
                observation,
                Observation::Value(expected.to_owned()),
                "{control}"
            );
        }
    }

    #[test]
    fn backends_agree_on_source_iteration_results() {
        for (body, expected, control) in [
            (
                "let step = Iteration.yield(nil); [step.yield?, step.done?, step.value]",
                "[true, false, nil]",
                "let step = Iteration.yield(1); [step.yield?, step.done?, step.value]",
            ),
            (
                "let first = Iteration.done; let second = Iteration.done; [first.done?, first.yield?, first.same?(second)]",
                "[true, false, true]",
                "let first = Iteration.yield(1); let second = Iteration.done; [first.done?, first.yield?, second.yield?]",
            ),
            (
                "class It { public fun initialize() -> Nil { @n = 0; nil } public fun next() -> Object { @n = @n + 1; if @n > 2 { Iteration.done } else { Iteration.yield(@n) } } public fun close() -> Nil { nil } } class C { public fun iterator() -> Object { It.new() } } mut sum = 0; for x in C.new() { sum = sum + x }; sum",
                "3",
                "class It { public fun initialize() -> Nil { @n = 0; nil } public fun next() -> Object { @n = @n + 1; if @n > 1 { Iteration.done } else { Iteration.yield(@n) } } public fun close() -> Nil { nil } } class C { public fun iterator() -> Object { It.new() } } mut sum = 0; for x in C.new() { sum = sum + x }; sum",
            ),
            (
                "class It { public fun next() -> Object { Iteration.done } public fun close() -> Nil { nil } } let it = It.new(); let first = it.next(); let second = it.next(); first.same?(second)",
                "true",
                "Iteration.yield(nil).done?",
            ),
        ] {
            let wrap = |body: &str| {
                let boundary = body.find(" mut sum").or_else(|| body.find(" let it"));
                boundary.map_or_else(
                    || format!("module M {{ public fun r() -> Object {{ {body} }} }} M.r()"),
                    |boundary| {
                        let (declarations, method_body) = body.split_at(boundary);
                        format!(
                            "{declarations} module M {{ public fun r() -> Object {{ {method_body} }} }} M.r()"
                        )
                    },
                )
            };
            let source = wrap(body);
            let control = wrap(control);
            assert_agreement(&source, expected);

            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let Agreement::Agreed { observation, .. } = compare_backends(&control, &backends)
            else {
                unreachable!(
                    "both backends must run the iteration-result negative control: {control}"
                )
            };
            assert_ne!(
                observation,
                Observation::Value(expected.to_owned()),
                "{control}"
            );
        }
    }

    #[test]
    fn backends_agree_that_iteration_done_has_no_value() {
        let source = "module M { public fun r() -> Object { Iteration.done.value } } M.r()";
        let control = "module M { public fun r() -> Object { Iteration.yield(nil).value } } M.r()";
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
            unreachable!("both backends must report Iteration.done.value")
        };
        assert!(
            matches!(observation, Observation::Error(ref error) if error.contains("IteratorState"))
        );

        let Agreement::Agreed { observation, .. } = compare_backends(control, &backends) else {
            unreachable!("both backends must run the yielded-nil negative control")
        };
        assert_eq!(observation, Observation::Value("nil".to_owned()));
        assert_ne!(observation, Observation::Error("IteratorState".to_owned()));
    }

    #[test]
    fn backends_agree_on_iteration_failures() {
        for (body, expected_fragment, control_fragment) in [
            (
                "[].iterator().next().value",
                "IteratorState",
                "MessageNotFound",
            ),
            (
                "for x in 5 { x }",
                "receiver_class: \"Integer\", selector: \"iterator\"",
                "receiver_class: \"String\", selector: \"iterator\"",
            ),
            (
                "let values = [1, 2]; for x in values { values[0] = 9 }",
                "ConcurrentModification",
                "IteratorState",
            ),
            (
                "class C { public fun iterator() -> Object { [1].iterator() } }; for x in C.new() { raise :boom }",
                "Raised",
                "ConcurrentModification",
            ),
        ] {
            let source = if body.starts_with("class C")
                && let Some((declaration, method_body)) = body.split_once("}; ")
            {
                format!(
                    "{declaration}}} module M {{ public fun r() -> Object {{ {method_body} }} }} M.r()"
                )
            } else {
                format!("module M {{ public fun r() -> Object {{ {body} }} }} M.r()")
            };
            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let agreement = compare_backends(&source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!(
                    "both backends must run the iteration failure: {source}: {agreement:?}"
                )
            };
            let Observation::Error(error) = observation else {
                unreachable!("iteration failure must raise: {source}")
            };
            assert!(error.contains(expected_fragment), "{source}: {error}");
            assert!(!error.contains(control_fragment), "{source}: {error}");
        }
    }

    #[test]
    fn backends_agree_on_exception_control_flow() {
        for (source, expected) in [
            ("try { 7 } catch error { error }", "7"),
            ("try { raise 7 } catch error { error }", "7"),
            (
                "try { try { raise 8 } catch inner { inner } } catch outer { outer }",
                "8",
            ),
            (
                "module M { public fun fail() -> Object { raise 9 } } try { M.fail() } catch error { error }",
                "9",
            ),
            (
                "mut mark = 0; try { 3 } finally { mark = 1 }; mark",
                "[3, 1]",
            ),
            (
                "mut mark = 0; try { raise 4 } catch error { error } finally { mark = 1 }; mark",
                "[4, 1]",
            ),
            ("try { 5 } catch error { error } finally { 0 }", "5"),
        ] {
            assert_agreement(source, expected);
        }
    }

    #[test]
    fn backends_agree_on_closure_capture_and_invocation() {
        for (source, expected) in [
            ("let x = 3; let f = { ||; x }; f.call()", "3"),
            ("let x = 3; let f = { |y|; x + y }; f.call(4)", "7"),
            ("let f = { |x|; let x = 8; x }; f.call(1)", "8"),
            ("let f = { |x|; return x + 1; 99 }; f.call(4)", "5"),
            (
                "module M { public fun make(x: Integer) -> Object { { ||; x } } } let f = M.make(6); f.call()",
                "6",
            ),
        ] {
            assert_agreement(source, expected);
        }
    }

    /// The subset boundary is explicit. A construct outside it must DECLINE,
    /// so a differential row relying on it stays held rather than passing on
    /// one backend.
    #[test]
    fn the_covered_subset_has_an_explicit_boundary() {
        let bytecode = Bytecode;

        for (source, construct) in [
            ("class A { }", "empty program"),
            ("for [x] in [[1]] { x }", "statement for"),
            ("unbound_name", "name unbound"),
            // `if`, `while`, and built-in indexes ARE covered now, so a
            // genuinely unsupported receiver remains outside the subset.
            ("1[0]", "index receiver"),
        ] {
            let Support::Unsupported(reason) = bytecode.execute(source) else {
                unreachable!("this backend does not cover: {source}")
            };
            assert_eq!(reason, construct, "{source}");
        }
    }

    /// The design review's first vertical-slice milestone: Integer, Bool/Nil,
    /// local variables, arithmetic and comparison, function definition and
    /// CALL, `if`, RECURSION, and `return` - with the interpreter and the
    /// bytecode backend producing identical results for the same program.
    #[test]
    fn backends_agree_across_the_first_vertical_slice() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected) in [
            // Function definition and call.
            (
                "module M { public fun add(a: Integer, b: Integer) -> Integer { a + b } }\nM.add(2, 3)",
                "5",
            ),
            // A zero-argument call still needs a valid argument window.
            (
                "module M { public fun zero() -> Integer { 7 } }\nM.zero()",
                "7",
            ),
            // `if` as a value, both arms.
            (
                "module M { public fun pick(n: Integer) -> Integer { if n > 0 { 1 } else { 0 } } }\nM.pick(-3)",
                "0",
            ),
            // Explicit `return`.
            (
                "module M { public fun f(n: Integer) -> Integer { return n + 1 } }\nM.f(1)",
                "2",
            ),
            // Recursion: each frame gets its own register file, so a callee
            // cannot disturb its caller's registers.
            (
                "module M { public fun fact(n: Integer) -> Integer { \
                   if n <= 1 { 1 } else { n * M.fact(n - 1) } } }\nM.fact(5)",
                "120",
            ),
            // Two recursive calls in one expression, which is where a shared
            // register file would corrupt the first result.
            (
                "module M { public fun fib(n: Integer) -> Integer { \
                   if n < 2 { n } else { M.fib(n - 1) + M.fib(n - 2) } } }\nM.fib(10)",
                "55",
            ),
        ] {
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends cover: {source}: {agreement:?}")
            };
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    /// Loops, mutable bindings and assignment. A loop is a BACKWARD jump,
    /// which is why the verifier had to become a dataflow fixpoint: a body is
    /// entered before its own writes have happened.
    #[test]
    fn backends_agree_on_loops_and_mutation() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected) in [
            // A value carried ACROSS iterations, which needs assignment to
            // update the binding's own register rather than a fresh one.
            (
                "mut t = 0; mut i = 1; while i <= 10 { t = t + i; i = i + 1 } t",
                "[nil, 55]",
            ),
            // A loop whose body never runs.
            ("mut n = 0; while false { n = 1 } n", "[nil, 0]"),
            (
                "mut i = 0; mut acc = 1; while i < 5 { acc = acc * 2; i = i + 1 } acc",
                "[nil, 32]",
            ),
            // A loop inside a FRAME, where the register file is the callee's.
            (
                "module M { public fun sum(n: Integer) -> Integer { \
                   mut t = 0; mut i = 1; while i <= n { t = t + i; i = i + 1 } t } }\nM.sum(10)",
                "55",
            ),
            ("mut a = 1; a = 2; a", "[2, 2]"),
            // The top-level value convention: non-binding statement values,
            // one directly and several as an Array.
            ("1 + 1", "2"),
            ("let a = 1; a", "1"),
            ("1 + 1; 2 + 2", "[2, 4]"),
        ] {
            let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
                unreachable!("both backends cover: {source}")
            };
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_array_for_loops_inside_functions() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (body, expected, wrong) in [
            ("mut t = 0; for x in [] { t = t + x }; t", "0", "nil"),
            ("mut t = 0; for x in [4] { t = t + x }; t", "4", "0"),
            ("mut t = 0; for x in [1, 2, 3] { t = t + x }; t", "6", "3"),
            (
                "mut t = 0; for x in [1, 2, 3] { if x == 2 { break }; t = t + x }; t",
                "1",
                "6",
            ),
            (
                "mut t = 0; for x in [1, 2, 3] { if x == 2 { continue }; t = t + x }; t",
                "4",
                "6",
            ),
            (
                "mut t = 0; for x in [1, 2] { for y in [3, 4] { t = t + x * y } }; t",
                "21",
                "14",
            ),
        ] {
            let source = format!("module M {{ public fun r() -> Object {{ {body} }} }} M.r()");
            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends cover: {source}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_initialized_annotated_bindings() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (body, expected, wrong) in [
            ("let x: Integer = 4; x + 1", "5", "4"),
            ("mut x: Integer = 4; x = x + 2; x", "6", "4"),
        ] {
            let source = format!("module M {{ public fun r() -> Object {{ {body} }} }} M.r()");
            let Agreement::Agreed { observation, .. } = compare_backends(&source, &backends) else {
                unreachable!("both backends cover initialized annotated bindings: {source}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_declared_class_values_inside_functions() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, wrong) in [
            (
                "class A { } module M { public fun r() -> Object { A } } M.r()",
                "<class>",
                "nil",
            ),
            (
                "class A { public fun v() -> Integer { 3 } } module M { public fun make(c: Object) -> Object { c.new().v() } public fun r() -> Object { M.make(A) } } M.r()",
                "3",
                "<class>",
            ),
            (
                "module N { public fun v() -> Integer { 1 } } module M { public fun r() -> Object { N } } M.r()",
                ":N",
                "nil",
            ),
        ] {
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends cover: {source}: {agreement:?}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_user_defined_class_state_and_dispatch() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, wrong) in [
            (
                "class Box { public fun initialize(value: Integer) { @value = value } public fun value() { @value } } Box.new(41).value()",
                "41",
                "nil",
            ),
            (
                "class Counter { public fun initialize() { @n = 2 } public fun add(x: Integer) { self.value() + x } public fun value() { @n } } Counter.new().add(3)",
                "5",
                "3",
            ),
            (
                "class Parent { public fun initialize(value: Integer) { @value = value } public fun value() { @value } } class Child extends Parent { } Child.new(9).value()",
                "9",
                "nil",
            ),
        ] {
            let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
                unreachable!("both backends must run the supported class subset: {source}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_symbols_identity_hashes_and_indexes() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, wrong) in [
            (
                "module M { public fun run() { :ready } } M.run()",
                ":ready",
                ":other",
            ),
            (
                "module M { public fun run() { nil same? nil } } M.run()",
                "true",
                "false",
            ),
            (
                "module M { public fun run() { nil same? false } } M.run()",
                "false",
                "true",
            ),
            (
                "module M { public fun run() { let a = [1]; a same? a } } M.run()",
                "true",
                "false",
            ),
            (
                "module M { public fun run() { [10, 20][1] } } M.run()",
                "20",
                "nil",
            ),
            (
                "module M { public fun run() { [10][-1] } } M.run()",
                "10",
                "nil",
            ),
            (
                "module M { public fun run() { %{ :answer: 42 }[:answer] } } M.run()",
                "42",
                "nil",
            ),
            (
                "module M { public fun run() { %{ :answer: 42 }[:missing] } } M.run()",
                "nil",
                "42",
            ),
        ] {
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                eprintln!("{agreement:?}");
                unreachable!("both backends must run the covered value constructs: {source}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_class_methods_and_native_receiver_calls() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, wrong) in [
            (
                "class Math { public class fun twice(x: Integer) { x * 2 } } Math.twice(6)",
                "12",
                "6",
            ),
            ("(7).hash()", "15185519486979246657", "7"),
        ] {
            let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
                unreachable!("both backends must dispatch the covered call: {source}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    /// An out-of-range index WRITE raises rather than being discarded.
    ///
    /// A read past the end answers nil, so a write is easy to implement as
    /// the same silent miss. The backend did exactly that: the element was
    /// never stored and the program carried on believing it had been, which
    /// only surfaced when a real program indexed an empty Array.
    #[test]
    fn an_out_of_range_index_write_raises_in_both_backends() {
        for source in [
            "module M { public fun r() -> Object { let a = []; a[0] = 1; a } } M.r()",
            "module M { public fun r() -> Object { let a = [1]; a[5] = 1; a } } M.r()",
        ] {
            let agreement = compare_backends(source, &[&Interpreter, &Bytecode]);
            let Agreement::Agreed { observation, .. } = &agreement else {
                unreachable!("both backends must observe the same refusal: {agreement:?}")
            };
            assert_eq!(observation, &Observation::Error("IndexError".to_owned()));
        }

        // Control: an IN-RANGE write stores and answers the value, so the
        // refusal above is about the position rather than about writes.
        let agreement = compare_backends(
            "module M { public fun r() -> Object { let a = [1]; a[0] = 9; a } } M.r()",
            &[&Interpreter, &Bytecode],
        );
        let Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must store an in-range write: {agreement:?}")
        };
        assert_eq!(observation, &Observation::Value("[9]".to_owned()));
    }

    /// A deferred binding assigned inside a BRANCH is declined, not lowered.
    ///
    /// Assignment used to discharge the deferral wherever it appeared, so a
    /// branch that may not run still marked the binding assigned. The read
    /// after it lowered to a register the verifier then proved unwritten,
    /// which surfaces as a machine failure - a COMPILER defect wearing a
    /// program error's clothes. Declining keeps the boundary honest.
    #[test]
    fn a_deferred_binding_assigned_in_a_branch_is_declined() {
        let bytecode = Bytecode;

        for source in [
            "module M { public fun r() -> Object { mut x: Integer; if true { x = 1 }; x } } M.r()",
            "module M { public fun r() -> Object { mut x: Integer; if true { x = 1 } else { x = 2 }; x } } M.r()",
        ] {
            let Support::Unsupported(reason) = bytecode.execute(source) else {
                unreachable!("a branch is not every path: {source}")
            };
            assert_eq!(reason, "deferred read before assignment", "{source}");
        }

        // Control: an assignment on the straight line DOES discharge it, so
        // the decline is about the branch rather than about deferral itself.
        let agreement = compare_backends(
            "module M { public fun r() -> Object { mut x: Integer; x = 5; x } } M.r()",
            &[&Interpreter, &Bytecode],
        );
        let Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("a straight-line assignment must run: {agreement:?}")
        };
        assert_eq!(observation, &Observation::Value("5".to_owned()));
    }

    /// A float answers text, so it can be printed and interpolated.
    ///
    /// Every other built-in value family answered `to_string`, but neither
    /// float width did, so `print(1.5)` failed on the CONVERSION rather than
    /// on anything the program did. An integral value keeps its trailing
    /// `.0`, which is what keeps `1.0` distinguishable from the Integer `1`.
    #[test]
    fn both_backends_render_floats_as_text() {
        for (source, expected) in [
            (
                "module M { public fun r() -> Object { (1.5).to_string() } } M.r()",
                "\"1.5\"",
            ),
            (
                "module M { public fun r() -> Object { (2.0).to_string() } } M.r()",
                "\"2.0\"",
            ),
            (
                "module M { public fun r() -> Object { \"v=${1.5}\" } } M.r()",
                "\"v=1.5\"",
            ),
        ] {
            let agreement = compare_backends(source, &[&Interpreter, &Bytecode]);
            let Agreement::Agreed { observation, .. } = &agreement else {
                unreachable!("both backends must render the float: {agreement:?}")
            };
            assert_eq!(
                observation,
                &Observation::Value(expected.to_owned()),
                "{source}"
            );
        }

        // Control: the Integer renders WITHOUT a trailing `.0`, so the
        // suffix above is the float's own text rather than decoration.
        let agreement = compare_backends(
            "module M { public fun r() -> Object { (2).to_string() } } M.r()",
            &[&Interpreter, &Bytecode],
        );
        let Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must render the Integer: {agreement:?}")
        };
        assert_eq!(observation, &Observation::Value("\"2\"".to_owned()));
    }

    /// Mutating an Array while iterating it RAISES rather than drifting.
    ///
    /// C026 versions every length-changing or element-replacing operation so
    /// an active iterator detects the change on its next advance. The backend
    /// advanced by index without comparing that version, so a loop that grew
    /// its own Array simply ran longer and answered a plausible number no
    /// test disputed - the silent-wrong-answer class, not a crash.
    ///
    /// The version must be captured OUTSIDE the loop. Capturing it at the top
    /// re-reads it every iteration and always finds it current, which passes
    /// this test's shape while checking nothing.
    #[test]
    fn mutating_an_array_while_iterating_it_raises_in_both_backends() {
        for source in [
            "module M { public fun r() -> Object { let a = [1,2]; mut n = 0; \
             for x in a { n = n + 1; if n < 5 { a.push(9) } }; n } } M.r()",
            "module M { public fun r() -> Object { let a = [1,2,3]; mut n = 0; \
             for x in a { n = n + 1; a.pop() }; n } } M.r()",
        ] {
            let agreement = compare_backends(source, &[&Interpreter, &Bytecode]);
            let Agreement::Agreed { observation, .. } = &agreement else {
                unreachable!("both backends must raise: {agreement:?}")
            };
            assert_eq!(
                observation,
                &Observation::Error("ConcurrentModification".to_owned()),
                "{source}"
            );
        }

        // Control: an unmutated loop still runs to completion, so the raise
        // above is about the mutation rather than about iterating at all.
        let agreement = compare_backends(
            "module M { public fun r() -> Object { let a = [1,2,3]; mut n = 0; \
             for x in a { n = n + x }; n } } M.r()",
            &[&Interpreter, &Bytecode],
        );
        let Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("an unmutated loop must run: {agreement:?}")
        };
        assert_eq!(observation, &Observation::Value("6".to_owned()));
    }

    /// A `for` over a Hash binds `(key, value)` pairs, and a value with no
    /// iterator names the selector it lacks.
    ///
    /// The backend accepted `for k in someHash` and then failed in the
    /// machine with a bare type error: it verified, so the compiler had
    /// promised something the machine could not do. C021 makes a Hash element
    /// an immutable Tuple, and the reference asks the receiver for an
    /// `iterator`, so a String or Integer reports the MISSING SELECTOR rather
    /// than a type error - `for x in 5` should say what 5 lacks.
    #[test]
    fn backends_agree_on_what_a_for_loop_can_traverse() {
        for (source, expected) in [
            (
                "module M { public fun r() -> Object { mut out = []; \
                 for k in %{ :a: 1, :b: 2 } { out.push(k) }; out } } M.r()",
                Observation::Value("[[:a, 1], [:b, 2]]".to_owned()),
            ),
            (
                "module M { public fun r() -> Object { mut n = 0; for k in %{} { n = n + 1 }; n } } M.r()",
                Observation::Value("0".to_owned()),
            ),
            // A Hash changed mid-loop raises on the next advance, exactly as
            // an Array does, rather than walking a structure that moved.
            (
                "module M { public fun r() -> Object { let h = %{ :a: 1 }; mut n = 0; \
                 for k in h { n = n + 1; h[:z] = 9 }; n } } M.r()",
                Observation::Error("ConcurrentModification".to_owned()),
            ),
            (
                "module M { public fun r() -> Object { for c in \"ab\" { c } } } M.r()",
                Observation::Error(
                    "MessageNotFound { receiver_class: \"String\", selector: \"iterator\" }"
                        .to_owned(),
                ),
            ),
            (
                "module M { public fun r() -> Object { for c in 5 { c } } } M.r()",
                Observation::Error(
                    "MessageNotFound { receiver_class: \"Integer\", selector: \"iterator\" }"
                        .to_owned(),
                ),
            ),
        ] {
            let agreement = compare_backends(source, &[&Interpreter, &Bytecode]);
            let Agreement::Agreed { observation, .. } = &agreement else {
                unreachable!("both backends must agree: {agreement:?}")
            };
            assert_eq!(observation, &expected, "{source}");
        }

        // Controls: the traversals that already worked still do, so the
        // change above is about WHAT can be traversed rather than about
        // rebuilding the loop.
        for (source, expected) in [
            (
                "module M { public fun r() -> Object { mut n = 0; for x in [1,2,3] { n = n + x }; n } } M.r()",
                "6",
            ),
            (
                "module M { public fun r() -> Object { mut n = 0; for x in 1..<3 { n = n + x }; n } } M.r()",
                "3",
            ),
        ] {
            let agreement = compare_backends(source, &[&Interpreter, &Bytecode]);
            let Agreement::Agreed { observation, .. } = &agreement else {
                unreachable!("both backends must agree: {agreement:?}")
            };
            assert_eq!(
                observation,
                &Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    /// `o.p = v` is a `p=` SEND, and a class without that setter is refused
    /// by NAME rather than by an interned selector number.
    ///
    /// Writing the field directly would succeed where the reference refuses,
    /// and would bypass a property setter's body where one exists. The
    /// refusal has to match too: two backends that both reject a program but
    /// describe the rejection differently still disagree, and a raw dispatch
    /// error carries only `Selector(10002)` where the reference names the
    /// Class and the selector.
    #[test]
    fn member_assignment_sends_a_setter_in_both_backends() {
        let with_setter = "class C { public fun initialize() -> Nil { @v = 1; nil } \
             public property fun v() -> Integer { @v } \
             public property fun v=(n: Integer) -> Nil { @v = n; nil } } \
             module M { public fun r() -> Object { let c = C.new(); c.v = 5; c.v } } M.r()";
        let agreement = compare_backends(with_setter, &[&Interpreter, &Bytecode]);
        let Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("the setter must run in both backends: {agreement:?}")
        };
        assert_eq!(observation, &Observation::Value("5".to_owned()));

        // Control: without the setter the send is refused, naming the Class
        // and the `v=` selector, so the success above is the setter's doing
        // rather than a field write that would have worked either way.
        let without_setter = "class C { public fun initialize() -> Nil { @v = 1; nil } } \
             module M { public fun r() -> Object { let c = C.new(); c.v = 5; 1 } } M.r()";
        let agreement = compare_backends(without_setter, &[&Interpreter, &Bytecode]);
        let Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must refuse alike: {agreement:?}")
        };
        assert_eq!(
            observation,
            &Observation::Error(
                "MessageNotFound { receiver_class: \"C\", selector: \"v=\" }".to_owned()
            )
        );
    }

    /// `catch e, context` binds the propagation context both backends build.
    ///
    /// C056 gives every raise a FRESH context with a distinct identity, C057
    /// chains an explicit `from cause` and lets `from nil` suppress chaining,
    /// and C067 makes context equality IDENTITY rather than structural - two
    /// contexts wrapping the same value are different contexts.
    #[test]
    fn backends_agree_on_the_bound_exception_context() {
        for (source, expected) in [
            (
                "module M { public fun r() -> Object { \
                 try { raise :x } catch e, ctx { ctx.value } } } M.r()",
                ":x",
            ),
            // No `from`, so nothing is chained.
            (
                "module M { public fun r() -> Object { \
                 try { raise :x } catch e, ctx { ctx.cause } } } M.r()",
                "nil",
            ),
            (
                "module M { public fun r() -> Object { try { \
                 try { raise :inner } catch a, c1 { raise :outer from c1 } \
                 } catch b, c2 { c2.cause.value } } } M.r()",
                ":inner",
            ),
            // C057: `from nil` SUPPRESSES the chaining that would otherwise
            // happen while handling another exception.
            (
                "module M { public fun r() -> Object { try { \
                 try { raise :inner } catch a, c1 { raise :outer from nil } \
                 } catch b, c2 { c2.cause } } } M.r()",
                "nil",
            ),
            // C067: separate propagation events are separate contexts even
            // when they carry the same value.
            (
                "module M { public fun r() -> Object { \
                 let a = try { raise :x } catch e, c { c }; \
                 let b = try { raise :x } catch e, c { c }; a.same?(b) } } M.r()",
                "false",
            ),
            (
                "module M { public fun r() -> Object { \
                 let a = try { raise :x } catch e, c { c }; a.same?(a) } } M.r()",
                "true",
            ),
        ] {
            let agreement = compare_backends(source, &[&Interpreter, &Bytecode]);
            let Agreement::Agreed { observation, .. } = &agreement else {
                unreachable!("both backends must agree: {agreement:?}")
            };
            assert_eq!(
                observation,
                &Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    /// A refusal names the same Class in both backends.
    ///
    /// The backend's class-name table stopped at the families it had reached,
    /// so a value outside it reported `Object`. Both backends then rejected
    /// the program while DESCRIBING the rejection differently, which is a
    /// disagreement even though neither ran it.
    #[test]
    fn a_refusal_names_the_same_class_in_both_backends() {
        let readonly = "module M { public fun r() -> Object { \
             try { raise :x } catch e, ctx { ctx.suppressed.to_string() } } } M.r()";
        let agreement = compare_backends(readonly, &[&Interpreter, &Bytecode]);
        let Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must refuse alike: {agreement:?}")
        };
        assert_eq!(
            observation,
            &Observation::Error(
                "MessageNotFound { receiver_class: \"ReadonlyArray\", selector: \"to_string\" }"
                    .to_owned()
            )
        );

        // Control: a family that was ALREADY named keeps its name, so the fix
        // added entries rather than renaming what worked.
        let tuple = "module M { public fun r() -> Object { (1, 2).to_string() } } M.r()";
        let agreement = compare_backends(tuple, &[&Interpreter, &Bytecode]);
        let Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("both backends must refuse alike: {agreement:?}")
        };
        assert_eq!(
            observation,
            &Observation::Error(
                "MessageNotFound { receiver_class: \"Tuple\", selector: \"to_string\" }".to_owned()
            )
        );
    }

    /// A named runtime failure is an ORDINARY CATCHABLE Iris error.
    ///
    /// The backend returned these straight out of the frame, so a `try` around
    /// them never saw them: the handler was on the stack and simply never
    /// consulted. C056 hands the raised value to the catch, and the reference
    /// binds the specification's name as a Symbol, so `catch e { e }` answers
    /// `:IndexError` rather than dying.
    ///
    /// Uncaught, the SAME failure must still surface as itself. Converting it
    /// eagerly reported `Raised(:IndexError)` where the reference reports
    /// IndexError - a program that never wrote a handler would have seen the
    /// error change shape because handlers exist elsewhere in the machine.
    #[test]
    fn a_named_runtime_failure_is_catchable_in_both_backends() {
        for (source, expected) in [
            (
                "module M { public fun r() -> Object { \
                 try { let a = []; a[0] = 1; :no } catch e { e } } } M.r()",
                ":IndexError",
            ),
            (
                "module M { public fun r() -> Object { \
                 try { [].iterator().next().value } catch e { e } } } M.r()",
                ":IteratorStateError",
            ),
            (
                "module M { public fun r() -> Object { try { [1].size() } catch e { e } } } M.r()",
                ":MessageNotFound",
            ),
            (
                "module M { public fun r() -> Object { try { let a = [1,2]; \
                 for x in a { a[0] = 9 } } catch e { e } } } M.r()",
                ":ConcurrentModificationError",
            ),
        ] {
            let agreement = compare_backends(source, &[&Interpreter, &Bytecode]);
            let Agreement::Agreed { observation, .. } = &agreement else {
                unreachable!("the handler must answer it: {agreement:?}")
            };
            assert_eq!(
                observation,
                &Observation::Value(expected.to_owned()),
                "{source}"
            );
        }

        // Control: with NO handler the failure is still itself, so the
        // conversion above is the catch's doing rather than a rename.
        let uncaught = "module M { public fun r() -> Object { \
             let a = [1,2]; for x in a { a[0] = 9 } } } M.r()";
        let agreement = compare_backends(uncaught, &[&Interpreter, &Bytecode]);
        let Agreement::Agreed { observation, .. } = &agreement else {
            unreachable!("an uncaught failure must surface as itself: {agreement:?}")
        };
        assert_eq!(
            observation,
            &Observation::Error("ConcurrentModification".to_owned())
        );
    }

    #[test]
    fn harder_constructs_remain_precisely_declined() {
        let bytecode = Bytecode;
        let source = "for [x] in [[1]] { x }";

        let Support::Unsupported(reason) = bytecode.execute(source) else {
            unreachable!("the VM must not approximate the declined construct: {source}")
        };
        // A declined construct must be named for ITSELF rather than for
        // whatever call encloses it, so the boundary says what is missing.
        assert!(!reason.starts_with("call"), "{source}: {reason}");
        assert_eq!(reason, "statement for", "{source}");
    }

    #[test]
    fn exact_identity_and_hash_reproductions_agree() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, wrong) in [
            (
                "class C { } module M { public fun r() -> Object { let a = C.new(); a.same?(a) } } M.r()",
                "true",
                "false",
            ),
            (
                "module M { public fun r() -> Object { let a = [1]; a.same?(a) } } M.r()",
                "true",
                "false",
            ),
            (
                "module M { public fun r() -> Object { let a = [1]; let b = [1]; a.same?(b) } } M.r()",
                "false",
                "true",
            ),
        ] {
            let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
                unreachable!("both backends must run the exact reproduction: {source}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }

        let bytecode = Bytecode;
        for source in [
            "module M { public fun r() -> Object { %{ a: 1 } } } M.r()",
            "module M { public fun r() -> Object { let h = %{ a: 1 }; h[:a] } } M.r()",
        ] {
            let Support::Unsupported(reason) = bytecode.execute(source) else {
                unreachable!("the VM must decline a bare-name Hash key: {source}")
            };
            assert_ne!(reason, "name", "{source}");
            assert_eq!(reason, "hash key name", "{source}");
            assert!(
                matches!(Interpreter.execute(source), Support::Ran(_)),
                "{source}"
            );
        }
    }

    #[test]
    fn added_constructs_work_inside_module_methods() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, wrong) in [
            (
                "module M { public fun r() -> Object { :wrapped } } M.r()",
                ":wrapped",
                ":other",
            ),
            (
                "module M { public fun r() -> Object { [4, 5][1] } } M.r()",
                "5",
                "nil",
            ),
            (
                "class C { public class fun value() { 9 } } module M { public fun r() -> Object { C.value() } } M.r()",
                "9",
                "nil",
            ),
            (
                "module M { public fun r() -> Object { (7).hash() } } M.r()",
                "15185519486979246657",
                "7",
            ),
            (
                "module M { public fun r() -> Object { %{ :a: 1 }[:a] } } M.r()",
                "1",
                "nil",
            ),
        ] {
            let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
                unreachable!("both backends must run the wrapped construct: {source}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_index_assignment_and_aliasing() {
        for (source, expected, wrong) in [
            (
                "module M { public fun r() -> Object { let a = [1, 2]; let b = a; a[0] = 9; b } } M.r()",
                "[9, 2]",
                "[1, 2]",
            ),
            (
                "module M { public fun r() -> Object { let h = %{ :a: 1 }; h[:a] = 9; h[:a] } } M.r()",
                "9",
                "1",
            ),
        ] {
            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
                unreachable!("both backends must run indexed mutation: {source}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_bare_member_binding() {
        let source = "class C { public fun v() -> Integer { 3 } } module M { public fun r() -> Object { C.new().v } } M.r()";
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
            unreachable!("both backends must bind a bare member")
        };
        assert_eq!(observation, Observation::Value("<method>".to_owned()));
        assert_ne!(observation, Observation::Value("3".to_owned()));

        let missing = "class C { } module M { public fun r() -> Object { C.new().nope } } M.r()";
        for backend in backends {
            let Support::Ran(observation) = backend.execute(missing) else {
                unreachable!("{} must report a missing bare member", backend.name())
            };
            assert!(
                matches!(observation, Observation::Error(ref error) if error.contains("MissingMethod"))
            );
            assert_ne!(observation, Observation::Value("nil".to_owned()));
        }
    }

    #[test]
    fn backends_agree_when_bound_methods_are_called() {
        for (source, expected, wrong) in [
            (
                "class C { public fun v() -> Integer { 3 } } module M { public fun r() -> Object { let m = C.new().v; m.call() } } M.r()",
                "3",
                "<method>",
            ),
            (
                "class C { public fun add(x: Integer) -> Integer { x + 2 } } module M { public fun r() -> Object { let m = C.new().add; m.call(5) } } M.r()",
                "7",
                "5",
            ),
            (
                "class C { public fun initialize() { @n = 1 } public fun add(x: Integer) { @n = @n + x } public fun value() { @n } } module M { public fun r() -> Object { let o = C.new(); let m = o.add; m.call(4); o.value() } } M.r()",
                "5",
                "1",
            ),
            (
                "class C { public fun v() -> Integer { 9 } } module M { public fun make(o: Object) -> Object { o.v } public fun use(m: Object) -> Object { m.call() } public fun r() -> Object { M.use(M.make(C.new())) } } M.r()",
                "9",
                "<method>",
            ),
        ] {
            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must call the bound method: {source}: {agreement:?}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_global_bindings_inside_module_methods() {
        for (source, expected, wrong) in [
            (
                "global let $g = 3; module M { public fun r() -> Object { $g } } M.r()",
                "[3, 3]",
                "[3, nil]",
            ),
            (
                "global mut $g = 3; module M { public fun r() -> Object { $g = 4; $g } } M.r()",
                "[3, 4]",
                "[3, 3]",
            ),
        ] {
            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run package globals: {source}: {agreement:?}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }

        let source = "class C { shared let @@value: Integer = 1 public fun value() -> Integer { @@value } } module M { public fun r() -> Object { C.new().value() } } M.r()";
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
            unreachable!("both backends must read an immutable shared binding")
        };
        assert_ne!(observation, Observation::Value("nil".to_owned()));
        assert_eq!(observation, Observation::Value("1".to_owned()));
    }

    #[test]
    fn backends_agree_on_contract_conformance_and_dispatch() {
        for (source, expected, wrong) in [
            (
                "contract Named { fun name() -> String } class User for Named { public impl fun name() -> String { \"iris\" } } module M { public fun r() -> Object { let view = User.new() as Named; view..name() } } M.r()",
                "\"iris\"",
                "<method>",
            ),
            (
                "contract Named { fun name() -> String } class User for Named { public impl fun name() -> String { \"iris\" } } module M { public fun r() -> Object { User.contracts().length() } } M.r()",
                "1",
                "0",
            ),
        ] {
            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run Contract machinery: {source}: {agreement:?}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }

        let bytecode = Bytecode;
        for (source, expected) in [
            (
                "contract Named { fun name() -> String } class User for Named { } module M { public fun r() -> Object { User.new() as Named } } M.r()",
                "contract requirement absent",
            ),
            (
                "contract Named { fun name() -> String } class User { public impl fun name() -> String { \"iris\" } } module M { public fun r() -> Object { 1 } } M.r()",
                "contract implementation undeclared",
            ),
        ] {
            let Support::Unsupported(reason) = bytecode.execute(source) else {
                unreachable!("the VM must precisely decline invalid Contract promises: {source}")
            };
            assert_ne!(reason, "declaration contract", "{source}");
            assert_eq!(reason, expected, "{source}");
        }
    }

    #[test]
    fn backends_agree_on_deferred_and_class_body_bindings() {
        for (source, expected, wrong) in [
            (
                "class Counter { shared mut @@n: Integer = 1 public fun bump() -> Integer { @@n = @@n + 1 } } module M { public fun r() -> Object { [Counter.new().bump(), Counter.new().bump()] } } M.r()",
                "[2, 3]",
                "[2, 2]",
            ),
            (
                "module M { public fun r() -> Object { mut x: Integer; x = 5; x } } M.r()",
                "5",
                "nil",
            ),
        ] {
            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run binding state: {source}: {agreement:?}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }

        let source = "module M { public fun r() -> Object { mut x: Integer; x } } M.r()";
        let (interpreter, bytecode) = both();
        let Support::Ran(observation) = interpreter.execute(source) else {
            unreachable!("the reference must diagnose a deferred read")
        };
        assert_ne!(observation, Observation::Value("nil".to_owned()));
        assert!(
            matches!(observation, Observation::Error(ref error) if error.contains("DefiniteAssignment"))
        );
        let Support::Unsupported(reason) = bytecode.execute(source) else {
            unreachable!("the VM must decline a read it cannot prove initialized")
        };
        assert_ne!(reason, "statement deferred");
        assert_eq!(reason, "deferred read before assignment");
    }

    #[test]
    fn backends_agree_on_property_methods_and_stored_properties() {
        for (source, expected, wrong) in [
            (
                "class Box { public property fun value() -> Integer { 7 } } module M { public fun r() -> Object { Box.new().value } } M.r()",
                "7",
                "<method>",
            ),
            (
                "class Box { property value: Integer = 3 } module M { public fun r() -> Object { Box.new().value } } M.r()",
                "3",
                "nil",
            ),
        ] {
            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run properties: {source}: {agreement:?}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn backends_agree_on_nested_closure_literals() {
        let source = "module M { public fun r() -> Object { let outer = { |x|; { |y|; x + y } }; outer.call(3).call(4) } } M.r()";
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
            unreachable!("both backends must run nested closures")
        };
        assert_ne!(observation, Observation::Value("4".to_owned()));
        assert_eq!(observation, Observation::Value("7".to_owned()));
    }

    #[test]
    fn added_values_remain_usable_across_method_frames() {
        for (source, expected, wrong) in [
            (
                "module M { public fun mutate(a: Object) -> Object { a[0] = 9 } public fun r() -> Object { let a = [1]; M.mutate(a); a[0] } } M.r()",
                "9",
                "1",
            ),
            (
                "global mut $g = [1]; module M { public fun mutate() -> Object { $g[0] = 8 } public fun r() -> Object { M.mutate(); $g[0] } } M.r()",
                "[[8], 8]",
                "[[1], 1]",
            ),
            (
                "module M { public fun make() -> Object { { |x|; { |y|; x + y } } } public fun use(f: Object) -> Object { f.call(40).call(2) } public fun r() -> Object { M.use(M.make()) } } M.r()",
                "42",
                "2",
            ),
        ] {
            let (interpreter, bytecode) = both();
            let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must use the produced value: {source}: {agreement:?}")
            };
            assert_ne!(
                observation,
                Observation::Value(wrong.to_owned()),
                "{source}"
            );
            assert_eq!(
                observation,
                Observation::Value(expected.to_owned()),
                "{source}"
            );
        }
    }

    #[test]
    fn bytecode_declines_name_and_expression_forms_precisely() {
        let bytecode = Bytecode;
        for (source, expected) in [
            (
                "module M { public fun r() -> Object { missing } } M.r()",
                "name unbound",
            ),
            (
                "module M { public fun r() -> Object { missing = 1 } } M.r()",
                "name assignment unbound",
            ),
            (
                "module M { public fun r() -> Object { @@missing } } M.r()",
                "expression class variable",
            ),
        ] {
            let Support::Unsupported(reason) = bytecode.execute(source) else {
                unreachable!("the VM must decline the unsupported form: {source}")
            };
            assert_ne!(reason, "name", "{source}");
            assert_ne!(reason, "expression", "{source}");
            assert_eq!(reason, expected, "{source}");
        }
    }

    #[test]
    fn backends_agree_on_inherited_override_methods() {
        let source = "class A { public fun value() -> Integer { 1 } } class B extends A { public override fun value() -> Integer { 2 } } module M { public fun r() -> Object { B.new().value() } } M.r()";
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let agreement = compare_backends(source, &backends);
        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends must run an inherited override: {agreement:?}")
        };
        assert_ne!(observation, Observation::Value("1".to_owned()));
        assert_eq!(observation, Observation::Value("2".to_owned()));
    }

    #[test]
    fn backends_agree_on_declarative_class_reopen_dispatch() {
        let source = "class A { public fun value() -> Integer { 1 } } open class A { public override fun value() -> Integer { 2 } } module M { public fun r() -> Object { A.new().value() } } M.r()";
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let agreement = compare_backends(source, &backends);
        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends must publish a declarative reopen: {agreement:?}")
        };
        assert_ne!(observation, Observation::Value("1".to_owned()));
        assert_eq!(observation, Observation::Value("2".to_owned()));
    }

    #[test]
    fn backends_agree_on_integer_range_boundaries() {
        let cases = [
            (
                "module M { public fun r() -> Object { mut total = 0 for x in 1..<4 { total = total + x } total } } M.r()",
                "6",
                "3",
            ),
            (
                "module M { public fun r() -> Object { mut total = 0 for x in 1..=3 { total = total + x } total } } M.r()",
                "6",
                "3",
            ),
            (
                "module M { public fun r() -> Object { mut total = 0 for x in 2..<2 { total = total + x } total } } M.r()",
                "0",
                "2",
            ),
            (
                "module M { public fun r() -> Object { mut total = 0 for x in 3..=1 { total = total + x } total } } M.r()",
                "6",
                "0",
            ),
        ];
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, negative) in cases {
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run {source}: {agreement:?}")
            };
            assert_ne!(observation, Observation::Value(negative.to_owned()));
            assert_eq!(observation, Observation::Value(expected.to_owned()));
        }
    }

    #[test]
    fn backends_agree_when_a_range_is_bound_without_iteration() {
        let source = "module M { public fun r() -> Object { let r = 1..<4; r } } M.r()";
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        let agreement = compare_backends(source, &backends);
        let Agreement::Agreed { observation, .. } = agreement else {
            unreachable!("both backends must preserve a bound range value: {agreement:?}")
        };
        assert_ne!(observation, Observation::Value("nil".to_owned()));
        assert_eq!(observation, Observation::Value("<range>".to_owned()));
    }

    #[test]
    fn backends_agree_on_default_parameter_values() {
        let source = "module M { public fun f(a: Integer, b: Integer = 2) -> Object { a + b } public fun r() -> Object { M.f(3) } } M.r()";
        let explicit = "module M { public fun f(a: Integer, b: Integer = 2) -> Object { a + b } public fun r() -> Object { M.f(3, 4) } } M.r()";
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (program, expected, negative) in [(source, "5", "3"), (explicit, "7", "5")] {
            let agreement = compare_backends(program, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run a defaulted call: {agreement:?}")
            };
            assert_ne!(observation, Observation::Value(negative.to_owned()));
            assert_eq!(observation, Observation::Value(expected.to_owned()));
        }
    }

    #[test]
    fn bytecode_declines_unimplemented_parameter_channels_precisely() {
        let bytecode = Bytecode;
        for (source, construct) in [
            (
                "module M { public fun f(*items) -> Object { items } } M.f(1)",
                "parameter rest",
            ),
            (
                "module M { public fun f(key item) -> Object { item } } M.f(item: 1)",
                "parameter keyword",
            ),
            (
                "module M { public fun f(&block) -> Object { block } } M.f() { 1 }",
                "parameter block",
            ),
        ] {
            assert_eq!(
                bytecode.execute(source),
                Support::Unsupported(construct.to_owned())
            );
            assert_ne!(
                bytecode.execute(source),
                Support::Ran(Observation::Value("nil".to_owned()))
            );
        }
    }

    #[test]
    fn backends_agree_on_tuple_values_and_indexing() {
        let cases = [
            (
                "module M { public fun r() -> Object { let t = (1, 2); t } } M.r()",
                "[1, 2]",
                "nil",
            ),
            (
                "module M { public fun r() -> Object { let t = (1, 2); t[1] } } M.r()",
                "2",
                "1",
            ),
            (
                "module M { public fun pick(t: Object) -> Object { t[0] } public fun r() -> Object { M.pick((4, 5)) } } M.r()",
                "4",
                "5",
            ),
        ];
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, negative) in cases {
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run a tuple program: {agreement:?}")
            };
            assert_ne!(observation, Observation::Value(negative.to_owned()));
            assert_eq!(observation, Observation::Value(expected.to_owned()));
        }
    }

    #[test]
    fn backends_agree_on_reified_nominal_types() {
        let cases = [
            (
                "module M { public fun r() -> Object { (Integer).type } } M.r()",
                "<type>",
                "class",
            ),
            (
                "module M { public fun r() -> Object { (Integer).type.kind() } } M.r()",
                ":nominal",
                ":class",
            ),
        ];
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, negative) in cases {
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run a reified Type: {agreement:?}")
            };
            assert_ne!(observation, Observation::Value(negative.to_owned()));
            assert_eq!(observation, Observation::Value(expected.to_owned()));
        }
    }

    #[test]
    fn backends_agree_on_builtin_class_values_and_type_queries() {
        let cases = [
            (
                "module M { public fun r() -> Object { Integer } } M.r()",
                "<class>",
                "<type>",
            ),
            (
                "module M { public fun r() -> Object { Integer.name } } M.r()",
                "nil",
                ":Integer",
            ),
            (
                "module M { public fun r() -> Object { Integer.type.kind() } } M.r()",
                ":nominal",
                ":class",
            ),
            (
                "module M { public fun r() -> Object { Integer.type.subtype?(Object.type) } } M.r()",
                "true",
                "false",
            ),
            (
                "module M { public fun r() -> Object { Object.type.assignable?(Float64.type) } } M.r()",
                "true",
                "false",
            ),
        ];
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, negative) in cases {
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run a builtin Class value: {agreement:?}")
            };
            assert_ne!(observation, Observation::Value(negative.to_owned()));
            assert_eq!(observation, Observation::Value(expected.to_owned()));
        }
    }

    #[test]
    fn backends_agree_on_nominal_is_tests() {
        let cases = [
            (
                "module M { public fun r() -> Object { 1 is Integer } } M.r()",
                "true",
                "false",
            ),
            (
                "module M { public fun r() -> Object { 1 is Float64 } } M.r()",
                "false",
                "true",
            ),
            (
                "class A { } class B extends A { } module M { public fun r() -> Object { B.new() is A } } M.r()",
                "true",
                "false",
            ),
            (
                "class A { } class B { } module M { public fun r() -> Object { A.new() is B } } M.r()",
                "false",
                "true",
            ),
        ];
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];

        for (source, expected, negative) in cases {
            let agreement = compare_backends(source, &backends);
            let Agreement::Agreed { observation, .. } = agreement else {
                unreachable!("both backends must run a nominal `is` test: {agreement:?}")
            };
            assert_ne!(observation, Observation::Value(negative.to_owned()));
            assert_eq!(observation, Observation::Value(expected.to_owned()));
        }
    }
}
