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

    #[test]
    fn verified_calls_never_reach_machine_dispatch_errors() {
        for source in [
            "class C { } module M { public fun r() -> Object { let a = C.new(); a.same?(a) } } M.r()",
            "module M { public fun r() -> Object { let a = [1]; a.same?(a) } } M.r()",
            "module M { public fun r() -> Object { let a = [1]; let b = [1]; a.same?(b) } } M.r()",
            "module M { public fun r() -> Object { let f = { |x|; x }; f.call(1) } } M.r()",
            "class C { public fun v() -> Integer { 3 } } module M { public fun r() -> Object { let m = C.new().v; m.call() } } M.r()",
            "module M { public fun r() -> Object { (7).hash() } } M.r()",
            "class C { public class fun v() -> Integer { 3 } } module M { public fun r() -> Object { C.v() } } M.r()",
            "class A { public fun v() -> Integer { 1 } } class B extends A { public override fun v() -> Integer { 2 } } module M { public fun r() -> Object { B.new().v() } } M.r()",
            "class A { public fun v() -> Integer { 1 } } open class A { public override fun v() -> Integer { 2 } } module M { public fun r() -> Object { A.new().v() } } M.r()",
            "class A { public fun v() -> Integer { 3 } } module M { public fun make(c: Object) -> Object { c.new().v() } public fun r() -> Object { M.make(A) } } M.r()",
            "class A { public class fun v() -> Integer { 5 } } module M { public fun invoke(c: Object) -> Object { c.v() } public fun r() -> Object { M.invoke(A) } } M.r()",
            "module M { public fun r() -> Object { [1, 2].map({ |x|; x * 2 }) } } M.r()",
            "module M { public fun r() -> Object { [1, 2].each({ |x|; x }) } } M.r()",
            "module M { public fun r() -> Object { [1, 2].select({ |x|; x > 1 }) } } M.r()",
            "module M { public fun r() -> Object { [1, 2].reduce(0, { |a, x|; a + x }) } } M.r()",
            "module M { public fun r() -> Object { let a = [1]; let b = a; a.push(2); b.pop() } } M.r()",
            "module M { public fun r() -> Object { [1, 2].join(\"-\") } } M.r()",
            "module M { public fun r() -> Object { [1, 2].find({ |x|; x > 1 }) } } M.r()",
            "module M { public fun r() -> Object { [1, 2].count({ |x|; x > 0 }) } } M.r()",
            "module M { public fun r() -> Object { [2, 1].sort() } } M.r()",
            "module M { public fun r() -> Object { [1, 2].all?({ |x|; x > 0 }) } } M.r()",
            "module M { public fun r() -> Object { [1, 2].each_with_index({ |x, i|; x + i }) } } M.r()",
            "module M { public fun r() -> Object { let h = %{ 1: 2 }.merge(%{ 1: 3 }); h[1] } } M.r()",
            "module M { public fun r() -> Object { (5).to_string() } } M.r()",
            "module M { public fun r() -> Object { %{ 1: 2 }.keys() } } M.r()",
            "module M { public fun r() -> Object { \" a \".trim() } } M.r()",
        ] {
            let program = program(source);
            assert_eq!(verify(&program), Ok(()), "{source}");
            let result = run(&program);
            assert!(
                !matches!(
                    result,
                    Err(MachineError::UnknownSelector(_)
                        | MachineError::Construction(iris_runtime::ConstructionError::Dispatch(_)))
                ),
                "{source}: {result:?}"
            );
            assert!(result.is_ok(), "{source}: {result:?}");
        }
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

    #[test]
    fn catch_entry_does_not_inherit_writes_skipped_by_a_raise() {
        let mut compiled = program("try { 1 } catch error { error }");
        compiled.registers = 3;
        compiled.instructions = vec![
            Instruction::EnterTry {
                handler: 5,
                cleanup: 6,
                exception: 0,
            },
            Instruction::LoadInteger {
                destination: 1,
                digits: "1".to_owned(),
            },
            Instruction::LeaveTry,
            Instruction::Move {
                destination: 2,
                source: 1,
            },
            Instruction::Jump { target: 6 },
            Instruction::Move {
                destination: 2,
                source: 1,
            },
        ];
        compiled.result = 2;

        assert_eq!(
            verify(&compiled),
            Err(VerifyError::ReadBeforeWrite { register: 1 })
        );
        assert_ne!(verify(&compiled), Ok(()));

        compiled.instructions[5] = Instruction::Move {
            destination: 2,
            source: 0,
        };
        assert_eq!(verify(&compiled), Ok(()));
        assert_ne!(
            verify(&compiled),
            Err(VerifyError::ReadBeforeWrite { register: 1 })
        );
    }

    #[test]
    fn filtered_catch_fallthrough_requires_the_exception_register() {
        let mut compiled = program("try { raise 1 } catch e: Symbol { 2 } catch e: Integer { e }");
        let Some(filter) = compiled
            .instructions
            .iter()
            .position(|instruction| matches!(instruction, Instruction::CatchMatch { .. }))
        else {
            unreachable!("a typed catch lowers a filter instruction")
        };
        compiled.instructions[filter] = Instruction::CatchMatch {
            destination: 2,
            exception: compiled.registers as Register - 1,
            class: "Symbol".to_owned(),
        };

        assert_eq!(
            verify(&compiled),
            Err(VerifyError::ReadBeforeWrite {
                register: compiled.registers as Register - 1
            })
        );
        assert_ne!(verify(&compiled), Ok(()));
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
    ///
    /// The structural pass checks each register a range NAMES before it checks
    /// the range as a whole, so a range running off the end is caught as the
    /// first non-existent register it names. Either refusal is correct; what
    /// matters is that the program does not execute.
    #[test]
    fn an_out_of_range_array_range_is_refused() {
        let mut compiled = program("[1, 2]");
        compiled.instructions.push(Instruction::BuildArray {
            destination: 0,
            first: 0,
            count: 99,
        });

        assert!(matches!(
            verify(&compiled),
            Err(VerifyError::ArrayRangeOutOfRange { .. } | VerifyError::RegisterOutOfRange { .. })
        ));

        // A range that stays IN range but is not fully written is refused as
        // a definite-assignment failure rather than a structural one.
        let mut unwritten = program("[1, 2]");
        let first = Register::try_from(unwritten.registers).unwrap_or_default();
        unwritten.registers += 2;
        unwritten.instructions.push(Instruction::BuildArray {
            destination: 0,
            first,
            count: 2,
        });
        assert_eq!(
            verify(&unwritten),
            Err(VerifyError::ReadBeforeWrite { register: first })
        );
    }

    /// Definite assignment is a DATAFLOW question, not a linear one.
    ///
    /// A single pass over the instruction list is unsound once control flow
    /// exists: a forward jump that SKIPS a write leaves the scan believing the
    /// register was written, because the scan walked past an instruction that
    /// execution never runs. That program then read an unwritten register.
    #[test]
    fn a_write_skipped_by_a_jump_does_not_count_as_written() {
        let mut skipped = program("1 + 2");
        skipped.registers = 6;
        skipped.result = 2;
        skipped.instructions = vec![
            Instruction::LoadInteger {
                destination: 0,
                digits: "1".to_owned(),
            },
            Instruction::Jump { target: 3 },
            Instruction::LoadInteger {
                destination: 1,
                digits: "9".to_owned(),
            },
            Instruction::Move {
                destination: 2,
                source: 1,
            },
        ];

        assert_eq!(
            verify(&skipped),
            Err(VerifyError::ReadBeforeWrite { register: 1 })
        );
    }

    /// The converse: a register written on BOTH paths of a branch IS written,
    /// so the merge keeps what every path establishes rather than refusing
    /// everything a branch touches.
    #[test]
    fn a_write_on_every_path_counts_as_written() {
        let mut branched = program("1 + 2");
        branched.registers = 4;
        branched.result = 2;
        branched.instructions = vec![
            Instruction::LoadBool {
                destination: 0,
                value: true,
            },
            Instruction::JumpUnless {
                condition: 0,
                target: 3,
            },
            Instruction::LoadInteger {
                destination: 1,
                digits: "1".to_owned(),
            },
            Instruction::LoadInteger {
                destination: 1,
                digits: "2".to_owned(),
            },
            Instruction::Move {
                destination: 2,
                source: 1,
            },
        ];

        assert_eq!(verify(&branched), Ok(()));
    }

    /// A loop is a BACKWARD jump, and the fixpoint must terminate on it.
    #[test]
    fn a_loop_verifies_and_terminates() {
        let compiled = program("mut i = 0; while i < 3 { i = i + 1 } i");

        assert_eq!(verify(&compiled), Ok(()));
        assert!(
            compiled
                .instructions
                .iter()
                .any(|instruction| matches!(instruction, Instruction::Jump { .. }))
        );
    }

    #[test]
    fn classes_construct_store_and_dispatch_on_the_receiver() {
        let source = "class Box { public fun initialize(value: Integer) { @value = value } public fun value() { @value } } let box = Box.new(41); box.value()";

        let compiled = program(source);
        let result = run(&compiled);

        assert_ne!(result, Ok(iris_runtime::Value::Integer(0_u8.into())));
        assert_eq!(result, Ok(iris_runtime::Value::Integer(41_u8.into())));
        assert_eq!(verify(&compiled), Ok(()));
    }

    #[test]
    fn methods_bind_self_and_dispatch_through_the_receiver_class() {
        let source = "class Counter { public fun initialize() { @value = 2 } public fun add(n: Integer) { self.value() + n } public fun value() { @value } } Counter.new().add(3)";

        let result = run(&program(source));

        assert_ne!(result, Ok(iris_runtime::Value::Integer(3_u8.into())));
        assert_eq!(result, Ok(iris_runtime::Value::Integer(5_u8.into())));
    }

    #[test]
    fn subclasses_dispatch_inherited_instance_methods() {
        let source = "class Parent { public fun initialize(value: Integer) { @value = value } public fun value() { @value } } class Child extends Parent { } Child.new(9).value()";

        let result = run(&program(source));

        assert_ne!(result, Ok(iris_runtime::Value::Nil));
        assert_eq!(result, Ok(iris_runtime::Value::Integer(9_u8.into())));
    }

    #[test]
    fn unsupported_class_shapes_are_declined_precisely() {
        for (source, expected, old_generic_error) in [
            ("@sealed() class A { } 1", "class decorator", "declaration"),
            ("open class A { } 1", "class reopen target", "declaration"),
            ("class A<T> { } 1", "class generics", "declaration"),
            ("class A for C { } 1", "class implements", "declaration"),
            ("class A mixin M { } 1", "class mixin", "declaration"),
            (
                "class A<T> where T: Object { } 1",
                "class constraints",
                "declaration",
            ),
            (
                "class A meta deny instance_state { } 1",
                "class meta deny",
                "declaration",
            ),
        ] {
            let Err(error) = compile(source) else {
                unreachable!("unsupported class shape must be declined: {source}")
            };
            assert_ne!(error.construct, old_generic_error, "{source}");
            assert_eq!(error.construct, expected, "{source}");
        }
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
            .filter_map(Instruction::destination)
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
            ("class A { }", "empty program"),
            ("for [x] in [[1]] { x }", "statement for"),
            ("unbound_name", "name unbound"),
            ("1[0]", "index receiver"),
            ("(1, 2)", "tuple"),
            ("try { 1 } catch e, context { e }", "try exception context"),
        ] {
            let Err(declined) = compile(source) else {
                unreachable!("the document says this is declined: {source}")
            };
            assert_eq!(declined.construct, construct, "{source}");
        }
    }

    /// Section 4.1: a jump outside the body is REFUSED. The design review
    /// records a `SPR` opcode in the old VM that fell through into the next
    /// case and overwrote its result; a checked target makes that class of
    /// defect a verification failure rather than silent corruption.
    #[test]
    fn a_jump_outside_the_body_is_refused() {
        let mut compiled = program("1 + 2");
        compiled.instructions.push(Instruction::Jump { target: 99 });

        assert_eq!(
            verify(&compiled),
            Err(VerifyError::JumpOutOfRange { target: 99 })
        );
    }

    /// Section 4.1: a call naming a function the program does not define is
    /// refused rather than indexed.
    #[test]
    fn a_call_to_an_unknown_function_is_refused() {
        let mut compiled = program("1 + 2");
        compiled.instructions.push(Instruction::Call {
            destination: 0,
            function: 7,
            first: 0,
            count: 1,
        });

        assert_eq!(
            verify(&compiled),
            Err(VerifyError::UnknownFunction { function: 7 })
        );
    }

    /// Section 2: each frame owns its register file, and a function's
    /// parameters occupy its LEADING registers.
    #[test]
    fn a_function_frame_binds_parameters_to_leading_registers() {
        let compiled = program(
            "module M { public fun add(a: Integer, b: Integer) -> Integer { a + b } }\nM.add(1, 2)",
        );

        let [function] = compiled.functions.as_slice() else {
            unreachable!("one module function was declared")
        };
        assert_eq!(function.name, "M.add");
        assert_eq!(function.parameters, 2);
        // The body reads r0 and r1 - the parameters - without any load.
        assert_eq!(
            function.instructions.first(),
            Some(&Instruction::Binary {
                destination: 2,
                selector: "+",
                left: 0,
                right: 1,
            })
        );
        // Every path out of a frame goes through one Return.
        assert_eq!(
            function.instructions.last(),
            Some(&Instruction::Return { value: 2 })
        );
    }

    /// Section 4.1: a function body is verified too, not just the top level.
    /// A parameter counts as written before the first instruction, since it
    /// arrives pre-bound.
    #[test]
    fn function_bodies_are_verified_with_parameters_pre_bound() {
        let compiled = program(
            "module M { public fun add(a: Integer, b: Integer) -> Integer { a + b } }\nM.add(1, 2)",
        );
        assert_eq!(verify(&compiled), Ok(()));

        // A body reading a register it never wrote is refused.
        let mut broken = compiled;
        broken.functions[0].instructions.insert(
            0,
            Instruction::Move {
                destination: 0,
                source: 2,
            },
        );
        assert_eq!(
            verify(&broken),
            Err(VerifyError::ReadBeforeWrite { register: 2 })
        );
    }
}

#[cfg(test)]
mod catch_edge_audit {
    use super::*;

    /// An independent check that the catch edge is not the try body's exit.
    ///
    /// This is the exception-shaped form of the unsoundness a linear scan had:
    /// a register written PART WAY through a protected region is not written
    /// when a raise transfers control out of that region, so treating the
    /// handler as an ordinary fall-through would let it read an unwritten
    /// register.
    ///
    /// The shape is taken from what the lowering actually emits for
    /// `try { 1 } catch e { e }`, with the handler changed to read the try
    /// body's register instead of the exception it is given.
    #[test]
    fn a_write_inside_the_try_body_is_not_written_at_the_handler() {
        let Ok(mut compiled) =
            compile("module M { public fun r() -> Object { try { 1 } catch e { e } } } M.r()")
        else {
            unreachable!("the backend covers this source")
        };
        let Some(function) = compiled.functions.first_mut() else {
            unreachable!("the program declares one function")
        };

        // Register 2 is written by the try BODY, so a raise reaches the
        // handler without it.
        function.instructions[7] = Instruction::Move {
            destination: 0,
            source: 2,
        };
        assert_eq!(
            verify(&compiled),
            Err(VerifyError::ReadBeforeWrite { register: 2 })
        );

        // Control: reading the exception register the edge DOES write is
        // admitted, so the refusal is about the skipped write rather than
        // about handlers being refused wholesale.
        let Some(function) = compiled.functions.first_mut() else {
            unreachable!("the program declares one function")
        };
        function.instructions[7] = Instruction::Move {
            destination: 0,
            source: 1,
        };
        assert_eq!(verify(&compiled), Ok(()));
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "temporary coverage measurement harness over a local corpus file"
)]
mod coverage_probe {
    use super::*;

    fn unb64(s: &str) -> String {
        const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut acc = 0u32;
        let mut bits = 0u32;
        let mut out: Vec<u8> = Vec::new();
        for c in s.bytes() {
            if c == b'=' {
                break;
            }
            let Some(i) = T.iter().position(|&t| t == c) else {
                continue;
            };
            acc = (acc << 6) | i as u32;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((acc >> bits) as u8);
            }
        }
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn measure() {
        let raw = std::fs::read_to_string("/tmp/srcs.tsv").unwrap();
        let mut counts: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        let mut ok = 0usize;
        let mut total = 0usize;
        for line in raw.lines() {
            let Some((_, b)) = line.split_once('\t') else {
                continue;
            };
            total += 1;
            match compile(&unb64(b)) {
                Ok(_) => ok += 1,
                Err(e) => *counts.entry(e.construct.clone()).or_default() += 1,
            }
        }
        println!("COV total={total} compiled={ok}");
        let mut v: Vec<_> = counts.into_iter().collect();
        v.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        for (k, n) in v {
            println!("GAP {n:5} {k}");
        }
    }
}
