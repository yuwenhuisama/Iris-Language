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
