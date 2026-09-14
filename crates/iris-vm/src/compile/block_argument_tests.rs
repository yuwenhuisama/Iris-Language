#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "test fixtures require valid compilation and shape"
)]

use super::lowering::{Declarations, Lowering};
use super::{Instruction, Program};
use iris_runtime::Value;
use iris_syntax::Expression;

fn lower(expression: &Expression) -> Program {
    let mut program = super::compile("nil").expect("base program");
    let mut closures = Vec::new();
    let mut lowering = Lowering::new(
        Declarations {
            signatures: &[],
            classes: &[],
            contracts: &[],
            modules: &[],
        },
        0,
        &mut closures,
        &[],
        true,
    );
    program.result = lowering
        .expression(expression)
        .expect("canonical expression");
    program.registers = usize::from(lowering.next_register);
    program.instructions = lowering.instructions;
    program.functions = closures;
    program
}

#[test]
fn typed_value_survives_when_canonical_code_is_cloned() {
    let given = Expression::BlockArgument {
        value: Box::new(Expression::Literal("nil".into())),
    };
    let when = crate::run(&lower(&given).clone());
    assert_eq!(when, Ok(Value::BlockArgument(Box::new(Value::Nil))));
}

#[test]
fn ordered_values_survive_when_block_is_between_arguments() {
    let given = Expression::Array(vec![
        Expression::Literal("1".into()),
        Expression::BlockArgument {
            value: Box::new(Expression::Literal("nil".into())),
        },
        Expression::Literal("2".into()),
    ]);
    let when = crate::run(&lower(&given));
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::BlockArgument(Box::new(Value::Nil)),
            Value::Integer(2_u64.into()),
        ])))
    );
}

fn block(value: &str) -> Expression {
    Expression::BlockArgument {
        value: Box::new(Expression::Literal(value.into())),
    }
}

fn call_program(source: &str, arguments: Vec<Expression>) -> Program {
    let mut parsed = iris_parser::parse(source);
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    let iris_syntax::Statement::Expression(Expression::Call {
        arguments: held, ..
    }) = parsed
        .program
        .statements
        .last_mut()
        .expect("call statement")
    else {
        panic!("call")
    };
    *held = arguments;
    let statement = parsed.program.statements.last().expect("call").clone();
    *parsed.program.entries.last_mut().expect("entry") =
        iris_syntax::ProgramEntry::Statement(statement);
    super::compile_parsed(
        (
            source,
            &iris_native_host::NativeRegistry::new(),
            super::CompilationMode::Script,
        ),
        parsed,
    )
    .expect("canonical call compiles")
}

#[test]
fn block_binds_when_between_positionals_and_keyword() {
    let given = call_program(
        "class C { public fun f(a, b, key extra, &work = nil) { %[a, b, extra, work] } } C.new().f()",
        vec![
            Expression::Literal("1".into()),
            block("nil"),
            Expression::Literal("2".into()),
            Expression::KeywordArgument {
                name: "extra".into(),
                value: Box::new(Expression::Literal("3".into())),
            },
        ],
    );
    let when = crate::run(&given.clone());
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(1_u64.into()),
            Value::Integer(2_u64.into()),
            Value::Integer(3_u64.into()),
            Value::Nil,
        ])))
    );
}

#[test]
fn duplicate_blocks_fail_when_defaults_would_raise() {
    let given = call_program(
        "class C { public fun f(a = missing, &work = nil) { raise :body } } C.new().f()",
        vec![block("nil"), block("nil")],
    );
    let when = crate::run(&given);
    assert_eq!(when, Err(crate::MachineError::ArgumentError));
}

#[test]
fn supplied_nil_skips_when_block_default_would_raise() {
    let given = call_program(
        "class C { public fun f(&work = missing) { work } } C.new().f()",
        vec![block("nil")],
    );
    let when = crate::run(&given);
    assert_eq!(when, Ok(Value::Nil));
}

#[test]
fn required_typed_block_rejects_when_nil_is_supplied() {
    let given = call_program(
        "class C { public fun f(&work: Closure<() -> Integer>) { work.call() } } C.new().f()",
        vec![block("nil")],
    );
    let when = crate::run(&given);
    assert_eq!(when, Err(crate::MachineError::TypeContractError));
}

#[test]
fn positional_slot_rejects_when_block_is_supplied() {
    let given = call_program(
        "class C { public fun f(value) { value } } C.new().f()",
        vec![block("nil")],
    );
    let when = crate::run(&given);
    assert_eq!(when, Err(crate::MachineError::ArgumentError));
}

#[test]
fn bound_method_rejects_when_block_has_no_slot() {
    let given = call_program(
        "class C { public fun f(value) { value } } let method = C.new().f; method.call()",
        vec![block("nil")],
    );
    let when = crate::run(&given);
    assert_eq!(when, Err(crate::MachineError::ArgumentError));
}

#[test]
fn closure_rejects_when_block_has_no_slot() {
    let given = call_program(
        "let callback = { |value| value }; callback.call()",
        vec![block("nil")],
    );
    let when = crate::run(&given);
    assert_eq!(when, Err(crate::MachineError::ArgumentError));
}

#[test]
fn generic_inference_uses_positionals_when_block_comes_first() {
    let given = call_program(
        "class C { public class fun f<T>(value: T, &work = nil) -> T { value } } C.f()",
        vec![block("nil"), Expression::Literal("7".into())],
    );
    let when = crate::run(&given);
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn verifier_rejects_when_block_source_is_unwritten() {
    let mut given = super::compile("nil").expect("base program");
    given.registers = 2;
    given.instructions = vec![Instruction::MakeBlockArgument {
        destination: 0,
        value: 1,
    }];
    let when = crate::verify(&given);
    assert!(when.is_err());
}

#[test]
fn verifier_rejects_when_block_destination_is_outside_frame() {
    let mut given = super::compile("nil").expect("base program");
    given.instructions.push(Instruction::MakeBlockArgument {
        destination: 1,
        value: 0,
    });
    let when = crate::verify(&given);
    assert!(when.is_err());
}

#[test]
fn gc_preserves_object_when_only_block_transport_roots_it() {
    let mut given =
        super::compile("class C {} let holder = C.new(); NativeFixture.compact_gc(); holder")
            .expect("GC program");
    let (slot, value) = given
        .instructions
        .iter()
        .enumerate()
        .find_map(|(index, instruction)| match instruction {
            Instruction::PublishBinding { source, .. } => Some((index, *source)),
            _ => None,
        })
        .expect("holder binding");
    given.instructions.insert(
        slot,
        Instruction::MakeBlockArgument {
            destination: value,
            value,
        },
    );
    let mut machine = crate::Machine::new().expect("machine");
    let when = machine.execute(&given);
    let Value::Array(values) = when.expect("GC result") else {
        panic!("result array")
    };
    assert_eq!(
        values.elements().first(),
        Some(&Value::Integer(0_u64.into()))
    );
}

#[test]
fn decorated_duplicate_fails_before_wrapper_when_default_has_effect() {
    let given = call_program(
        r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   raise :wrapper
  })
 }
}
class C { @Wrap() public fun f(value = missing, &work = nil) { raise :body } }
C.new().f()
"#,
        vec![block("nil"), block("nil")],
    );
    let when = crate::run(&given);
    assert_eq!(when, Err(crate::MachineError::ArgumentError));
}

#[test]
fn using_rejects_duplicate_when_cleanup_would_run() {
    let given = call_program(
        "class C { public fun close() { raise :closed } } using()",
        vec![Expression::Name("nil".into()), block("nil"), block("nil")],
    );
    let when = crate::run(&given);
    assert_eq!(when, Err(crate::MachineError::ArgumentError));
}

#[test]
fn native_callback_rejects_duplicate_when_input_is_empty() {
    let given = call_program("%[].map()", vec![block("nil"), block("nil")]);
    let when = crate::run(&given);
    assert_eq!(when, Err(crate::MachineError::ArgumentError));
}

#[test]
fn old_parser_block_binds_when_sent_through_super() {
    let given = super::compile("class Base { public fun f(&work) { work.call() } } class Child extends Base { public override fun f(&work) { super() { 7 } } } Child.new().f() { 3 }").expect("super program");
    let when = crate::run(&given);
    assert_eq!(when, Ok(Value::Integer(7_u64.into())));
}

#[test]
fn qualified_block_binds_when_canonical_nil_is_supplied() {
    let given = call_program(
        "contract K { fun f(&work = nil) } class C {} impl C for K { public fun f(&work = nil) { work } } (C.new() as K)..f()",
        vec![block("nil")],
    );
    let when = crate::run(&given);
    assert_eq!(when, Ok(Value::Nil));
}
