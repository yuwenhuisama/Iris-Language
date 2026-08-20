//! A bytecode backend for Iris.
//!
//! `IRIS-V1-TRACE-C011` puts the VM out of scope for this specification wave,
//! so the instruction set is an engineering choice rather than a mandated one.
//! What IS constrained is agreement: a differential row asserts that every
//! backend produces the same semantic observation, and
//! `IRIS-V1-CONFORMANCE-C068` forbids comparing private bytecode layout, so
//! only the observable result matters here.
//!
//! This backend is deliberately PARTIAL. It compiles the constructs it fully
//! understands and DECLINES everything else, rather than approximating. A
//! backend that guesses would make a differential row agree for the wrong
//! reason, which is worse than leaving the row held.
//!
//! Arithmetic dispatches through [`iris_runtime::Kernel`], the same kernel the
//! tree-walking evaluator uses. Reimplementing bigint or IEEE-754 arithmetic
//! here would create exactly the silent numeric divergence that
//! `IRIS-V1-RUNTIME-V052` and `IRIS-V1-RUNTIME-V066` exist to detect, and the
//! rows would then be comparing two copies of the same bug.

mod compile;
mod machine;

pub use compile::{CompileError, FloatWidth, Instruction, Program, Register, compile};
pub use machine::{Machine, MachineError, VerifyError, run, verify};

#[cfg(test)]
mod tests {
    use super::*;

    fn program(source: &str) -> Program {
        let Ok(program) = compile(source) else {
            unreachable!("this backend covers: {source}")
        };
        program
    }

    /// The IR is three-address: every instruction names its operands and its
    /// destination, so an instruction's meaning does not depend on execution
    /// history. `1 + 2` is two loads and one binary, with NO push or pop.
    #[test]
    fn lowering_is_three_address_with_explicit_registers() {
        let compiled = program("1 + 2");

        assert_eq!(
            compiled.instructions,
            vec![
                Instruction::LoadInteger {
                    destination: 0,
                    digits: "1".to_owned()
                },
                Instruction::LoadInteger {
                    destination: 1,
                    digits: "2".to_owned()
                },
                Instruction::Binary {
                    destination: 2,
                    selector: "+",
                    left: 0,
                    right: 1
                },
            ]
        );
    }

    /// A rebinding SHADOWS rather than overwrites. Reusing the earlier
    /// register would corrupt any instruction still naming it.
    #[test]
    fn rebinding_a_name_uses_a_fresh_register() {
        let compiled = program("let a = 1; let a = 2; a");

        let destinations: Vec<Register> = compiled
            .instructions
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::Move { destination, .. } => Some(*destination),
                _ => None,
            })
            .collect();
        let [first, second] = destinations.as_slice() else {
            unreachable!("two bindings emit two moves")
        };
        assert_ne!(first, second);
        assert_eq!(compiled.result, *second);
    }

    /// Every covered program VERIFIES. The old C++ VM read operands with no
    /// verifier at all, which made corrupt bytecode undefined behaviour.
    #[test]
    fn every_covered_program_verifies() {
        for source in [
            "1 + 2",
            "let a = 2; let b = 3; a * b",
            "[1, 2, 3]",
            "-5",
            "Float32.from_bits(0x3f800001).to_bits()",
            "nil.hash()",
            "6 & 3",
        ] {
            assert_eq!(verify(&program(source)), Ok(()), "{source}");
        }
    }

    /// A malformed program is REFUSED rather than executed. This is the check
    /// whose absence was a P0 defect in the old VM.
    #[test]
    fn reading_an_unwritten_register_is_refused() {
        let mut compiled = program("1 + 2");
        // Point the addition at a register nothing ever writes.
        compiled.instructions[2] = Instruction::Binary {
            destination: 2,
            selector: "+",
            left: 0,
            right: 1,
        };
        compiled.registers = 4;
        compiled.instructions.push(Instruction::Move {
            destination: 2,
            source: 3,
        });

        assert_eq!(
            verify(&compiled),
            Err(VerifyError::ReadBeforeWrite { register: 3 })
        );
        assert_eq!(
            run(&compiled),
            Err(MachineError::Invalid(VerifyError::ReadBeforeWrite {
                register: 3
            }))
        );
    }

    /// An out-of-range register is refused rather than indexed, which is what
    /// turns corrupt bytecode into a reportable error instead of a crash.
    #[test]
    fn an_out_of_range_register_is_refused() {
        let mut compiled = program("1 + 2");
        compiled.instructions.push(Instruction::Move {
            destination: 99,
            source: 0,
        });

        assert_eq!(
            verify(&compiled),
            Err(VerifyError::RegisterOutOfRange { register: 99 })
        );
    }

    /// An Array range that leaves the register file is refused too.
    #[test]
    fn an_out_of_range_array_range_is_refused() {
        let mut compiled = program("[1, 2]");
        compiled.instructions.push(Instruction::BuildArray {
            destination: 0,
            first: 0,
            count: 99,
        });

        assert_eq!(
            verify(&compiled),
            Err(VerifyError::ArrayRangeOutOfRange {
                first: 0,
                count: 99
            })
        );
    }
}

/// Tests pinning `docs/iris-ir.md` to actual behaviour.
///
/// A design document that drifts from the implementation is worse than none:
/// it becomes confidently wrong. Each test here corresponds to a claim in that
/// document, so widening the IR without updating the document fails the build.
#[cfg(test)]
mod ir_document_tests {
    use super::*;

    fn program(source: &str) -> Program {
        let Ok(program) = compile(source) else {
            unreachable!("this backend covers: {source}")
        };
        program
    }

    fn destinations(program: &Program) -> Vec<Register> {
        program
            .instructions
            .iter()
            .map(Instruction::destination)
            .collect()
    }

    /// Section 2.1: the COMPILER emits single-assignment form.
    #[test]
    fn the_compiler_writes_each_register_exactly_once() {
        for source in [
            "1 + 2",
            "let a = 1; let a = 2; a",
            "[1, 2, 3]",
            "let a = 2; let b = 3; a * b",
            "Float32.from_bits(1).to_bits()",
        ] {
            let written = destinations(&program(source));
            let mut seen = std::collections::HashSet::new();
            for register in &written {
                assert!(
                    seen.insert(*register),
                    "{source} writes r{register} more than once"
                );
            }
        }
    }

    /// Section 2.1: the VERIFIER does NOT require single assignment, so a
    /// future optimisation pass may reuse a register. Conflating the two would
    /// let such a pass assume a guarantee that was never checked.
    #[test]
    fn the_verifier_permits_reusing_a_register() {
        let mut reused = program("1 + 2");
        reused.instructions.push(Instruction::Move {
            destination: 0,
            source: 1,
        });

        assert_eq!(verify(&reused), Ok(()));
    }

    /// Section 2.2: a rebinding SHADOWS into a fresh register, because an
    /// earlier instruction may still name the old one.
    #[test]
    fn rebinding_shadows_into_a_fresh_register() {
        let compiled = program("let a = 1; let a = 2; a");

        let moves: Vec<Register> = compiled
            .instructions
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::Move { destination, .. } => Some(*destination),
                _ => None,
            })
            .collect();
        let [first, second] = moves.as_slice() else {
            unreachable!("two bindings emit two moves")
        };
        assert_ne!(first, second);
        assert_eq!(compiled.result, *second, "the program answers the newest");
    }

    /// Section 1: `1 + 2` is exactly two loads and one binary - no push, no
    /// pop, every operand named.
    #[test]
    fn lowering_is_three_address() {
        assert_eq!(
            program("1 + 2").instructions,
            vec![
                Instruction::LoadInteger {
                    destination: 0,
                    digits: "1".to_owned(),
                },
                Instruction::LoadInteger {
                    destination: 1,
                    digits: "2".to_owned(),
                },
                Instruction::Binary {
                    destination: 2,
                    selector: "+",
                    left: 0,
                    right: 1,
                },
            ]
        );
    }

    /// Section 3.4: array elements occupy a CONTIGUOUS range, which is what
    /// lets the instruction name a range instead of an operand list.
    #[test]
    fn array_elements_are_contiguous() {
        let compiled = program("[1, 2]");

        let Some(Instruction::BuildArray {
            first,
            count,
            destination,
        }) = compiled.instructions.last()
        else {
            unreachable!("an array literal ends in BuildArray")
        };
        assert_eq!(*count, 2);
        assert_eq!(compiled.result, *destination);
        // The named range is exactly the moves that precede it.
        let moved: Vec<Register> = compiled
            .instructions
            .iter()
            .filter_map(|instruction| match instruction {
                Instruction::Move { destination, .. } => Some(*destination),
                _ => None,
            })
            .collect();
        assert_eq!(moved, vec![*first, first + 1]);
    }

    /// Section 4.3: the verifier does NOT check types. A type error is a
    /// RUN-TIME raise through the kernel, which is correct for a dynamically
    /// typed language.
    #[test]
    fn verification_accepts_a_program_that_raises_at_run_time() {
        let mismatched = program("1 + \"text\"");

        assert_eq!(verify(&mismatched), Ok(()));
        // A type mismatch RAISES rather than failing verification.
        assert!(matches!(run(&mismatched), Err(MachineError::Kernel(_))));
    }

    /// Section 5: the coverage boundary is pinned by name, so widening the IR
    /// without updating the document fails here.
    #[test]
    fn the_coverage_boundary_matches_the_document() {
        for (source, construct) in [
            ("{ |x|; x }", "closure"),
            ("class A { }", "declaration"),
            ("mut a = 1; a", "statement"),
            ("if true { 1 } else { 2 }", "statement"),
            ("unbound_name", "name"),
            (":symbol", "symbol"),
            ("(1, 2)", "tuple"),
            ("while false { 1 }", "statement"),
        ] {
            let Err(declined) = compile(source) else {
                unreachable!("the document says this is declined: {source}")
            };
            assert_eq!(declined.construct, construct, "{source}");
        }
    }
}
