use iris_runtime::Value;
use iris_vm::{compile, run};

#[test]
fn constructor_purity_is_checked_when_contract_is_planned() {
    let source = "class Impure { fun initialize() { print(:effect) } } impl Impure for ContractDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.empty } } @Impure() contract Named {} 0";
    let program = compile(source).expect("purity fixture compiles");
    let result = run(&program);
    assert_eq!(
        result,
        Err(iris_vm::MachineError::LexicalDiagnostic(
            "IRIS-DECORATOR-NONDETERMINISTIC"
        ))
    );
}

#[test]
fn contract_name_is_hidden_when_transform_has_not_published() {
    let source = "class Inspect {} impl Inspect for ContractDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { let hidden = Named; Transformation.empty } } @Inspect() contract Named {} 0";
    let program = compile(source).expect("publication fixture compiles");
    let result = run(&program);
    assert_eq!(result, Err(iris_vm::MachineError::NameError));
}

#[test]
fn fresh_instances_are_used_when_contract_phases_execute() {
    let source = r#"
class Fresh {
 fun initialize() { @count = 0; @kind = Plan.empty.kind }
}
impl Fresh for ContractDecorator {
 public fun plan(declaration, arguments) -> Plan {
  if @count != 0 || @kind != :contract { raise :reused }
  @count = 1
  Plan.empty
 }
 public fun transform(declaration, arguments, context) -> Transformation {
  if @count != 0 || @kind != :contract { raise :reused }
  @count = 1
  Transformation.empty
 }
}
@Fresh() @Fresh() contract Named { fun name() -> Symbol }
Named.reflect.decorators.length()
"#;
    let program = compile(source).expect("freshness fixture compiles");
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(2_u64.into())));
}

#[test]
fn reflection_is_readonly_when_contract_is_published() {
    let source = "contract Named { fun name() -> Symbol } let view = Named.reflect; %[try { view.requirements.append(:added) } catch error { error }, try { view[:name] = :Other } catch error { error }, view.requirements[0]]";
    let program = compile(source).expect("readonly fixture compiles");
    let result = run(&program);
    assert_eq!(
        result,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Symbol("ReadonlyMutationError".into()),
            Value::Symbol("ReadonlyMutationError".into()),
            Value::Symbol("name".into())
        ])))
    );
}

#[test]
fn wrong_target_is_rejected_when_contract_uses_class_decorator() {
    let source = "class Wrong {} impl Wrong for ClassDecorator { public fun plan(d, a) -> Plan { Plan.empty } public fun transform(d, a, c) -> Transformation { Transformation.empty } } @Wrong() contract Named {} 0";
    let result = compile(source);
    assert!(result.is_err());
}

#[test]
fn wrong_plan_is_rejected_when_contract_receives_transformation() {
    let source = "class Wrong {} impl Wrong for ContractDecorator { public fun plan(d, a) -> Plan { Transformation.empty } public fun transform(d, a, c) -> Transformation { Transformation.empty } } @Wrong() contract Named {} 0";
    let program = compile(source).expect("wrong plan fixture compiles");
    let result = run(&program);
    assert!(result.is_err());
}

#[test]
fn contract_requirements_are_preserved_when_empty_transform_runs() {
    let source = r#"
class Inspect {}
impl Inspect for ContractDecorator {
 public fun plan(declaration, arguments) -> Plan {
  if declaration.requirements.length() != 2 { raise :count }
  Plan.empty
 }
 public fun transform(declaration, arguments, context) -> Transformation {
  if context.kind != :contract { raise :kind }
  if declaration.requirements[0] != :name { raise :name }
  Transformation.empty
 }
}
@Inspect()
contract Named { fun name() -> Symbol; fun number(value: Integer) -> Integer }
Named.reflect.requirements.length()
"#;
    let program = compile(source).expect("contract fixture compiles");
    let result = run(&program);
    assert_eq!(result, Ok(Value::Integer(2_u64.into())));
}

#[test]
fn contract_operation_is_rejected_when_adding_method() {
    let source = r#"
class Add {}
impl Add for ContractDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.add_method(:added, { || -> Integer; 1 })
 }
}
@Add() contract Named { fun name() -> Symbol }
0
"#;
    let program = compile(source).expect("negative fixture compiles");
    let result = run(&program);
    assert_eq!(
        result,
        Err(iris_vm::MachineError::LexicalDiagnostic(
            "IRIS-DECORATOR-KIND"
        ))
    );
}
