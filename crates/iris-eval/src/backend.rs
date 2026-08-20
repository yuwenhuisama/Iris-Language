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
        // outer construct, so that is what the backend names.
        let Support::Unsupported(reason) = bytecode.execute("[1, 2].map({ |x|; x })") else {
            unreachable!("this backend covers no calls, arrays or closures yet")
        };
        assert_eq!(reason, "call");

        let Support::Unsupported(reason) = bytecode.execute("[1, 2]") else {
            unreachable!("this backend covers no arrays yet")
        };
        assert_eq!(reason, "array");

        // A declined construct leaves the comparison INSUFFICIENT, so a row
        // relying on it stays held rather than passing on one backend.
        let interpreter = Interpreter;
        let backends: Vec<&dyn Backend> = vec![&interpreter, &bytecode];
        let Agreement::Insufficient { ran, declined } =
            compare_backends("[1, 2].map({ |x|; x })", &backends)
        else {
            unreachable!("only one backend ran it")
        };
        assert_eq!(ran, vec!["interpreter"]);
        assert_eq!(declined.len(), 1);
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
}
