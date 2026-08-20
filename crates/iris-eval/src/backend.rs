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
    fn backends_keep_array_size_absent() {
        let (interpreter, bytecode) = both();
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let source = "module M { public fun r() -> Object { [1].size() } } M.r()";
        let control = "module M { public fun r() -> Object { [1].length() } } M.r()";

        let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
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

        let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
            unreachable!("both backends run filtered catches in method frames")
        };

        assert_eq!(observation, Observation::Value("2".to_owned()));
        assert_ne!(observation, Observation::Value("10".to_owned()));
    }

    /// ExceptionContext carries propagation metadata the register VM does not
    /// yet model, so accepting it as the raised value would be false agreement.
    #[test]
    fn bytecode_declines_exception_context_binding_precisely() {
        let bytecode = Bytecode;
        let source = "module M { public fun r() -> Object { try { raise 1 } catch e, context { e } } } M.r()";

        let Support::Unsupported(reason) = bytecode.execute(source) else {
            unreachable!("the bytecode backend has no ExceptionContext model")
        };

        assert_eq!(reason, "try exception context");
        assert_ne!(reason, "try filtered catch");
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
        let Agreement::Agreed { observation, .. } = compare_backends(source, &backends) else {
            unreachable!("both backends must run: {source}")
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

    #[test]
    fn harder_constructs_remain_precisely_declined() {
        let bytecode = Bytecode;

        for (source, construct) in [
            (
                "try { 1 } catch error, context { error }",
                "try exception context",
            ),
            ("for [x] in [[1]] { x }", "statement for"),
        ] {
            let Support::Unsupported(reason) = bytecode.execute(source) else {
                unreachable!("the VM must not approximate the declined construct: {source}")
            };
            // A declined construct must be named for ITSELF rather than for
            // whatever call encloses it, so the boundary says what is missing.
            assert!(!reason.starts_with("call"), "{source}: {reason}");
            assert_eq!(reason, construct, "{source}");
        }
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

        let bytecode = Bytecode;
        for (source, construct) in [
            (
                "class C { shared let @@value: Integer = 1 } module M { public fun r() -> Object { 1 } } M.r()",
                "class body",
            ),
            (
                "module M { public fun r() -> Object { mut value: Integer; value } } M.r()",
                "statement deferred",
            ),
        ] {
            let Support::Unsupported(reason) = bytecode.execute(source) else {
                unreachable!("the VM must decline unsupported binding state: {source}")
            };
            assert_eq!(reason, construct, "{source}");
            assert_ne!(reason, "statement binding", "{source}");
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
}
