use iris_runtime::Value;
use iris_vm::{MachineError, compile, run};

fn execute(source: &str) -> Result<Value, MachineError> {
    let program = compile(source);
    assert!(program.is_ok(), "compile: {program:?}");
    match program {
        Ok(program) => run(&program),
        Err(_) => Err(MachineError::UnsupportedConstruct),
    }
}

#[test]
fn decorator_impl_is_available_when_written_before_or_after_origin() {
    for contract in ["ClassDecorator", "MethodDecorator"] {
        for before in [true, false] {
            let implementation = format!(
                "impl Identity for {contract} {{ public fun plan(d, a) -> Plan {{ Plan.empty }} public fun transform(d, a, c) -> Transformation {{ Transformation.empty }} }}"
            );
            let origin = "class Identity {}";
            let decorated = match contract {
                "ClassDecorator" => {
                    "@Identity() class Target { public fun value() -> Integer { 7 } }"
                }
                _ => "class Target { @Identity() public fun value() -> Integer { 7 } }",
            };
            let given = if before {
                format!("{implementation} {origin} {decorated} Target.new().value()")
            } else {
                format!("{origin} {implementation} {decorated} Target.new().value()")
            };
            let when = execute(&given);
            assert_eq!(when, Ok(Value::Integer(7_u8.into())), "{given}");
        }
    }
}

#[test]
fn static_impl_publishes_when_origin_revision_is_one() {
    let given = "contract C { fun value() -> Integer } class A {} impl A for C { public fun value() -> Integer { 7 } } A.active_revision";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(1_u8.into())));
}

#[test]
fn inherited_requirements_work_when_sent_through_parent_views_and_guards() {
    let given = "contract Parent { fun value() -> Integer } contract Child extends Parent {} class A {} impl A for Child { public fun value() -> Integer { 7 } } module Guard { public fun guarded(value: Parent) -> Integer { (value as Parent)..value() } } let value = A.new(); value.value() + (value as Child)..value() + Guard.guarded(value)";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(21_u8.into())));
}

#[test]
fn generic_requirements_substitute_when_inherited_transitively() {
    let given = "contract Root<T> { fun value(input: T) -> T } contract Middle<U> extends Root<U> {} contract Child extends Middle<Integer> {} class A {} impl A for Child { public fun value(input: Integer) -> Integer { input } } module Guard { public fun guarded(value: Root<Integer>) -> Integer { (value as Root<Integer>)..value(3) } } let value = A.new(); value.value(1) + (value as Child)..value(2) + Guard.guarded(value)";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(6_u8.into())));
}

#[test]
fn generic_child_substitutes_when_impl_closes_child_arguments() {
    let given = "contract Parent<T> { fun value(input: T) -> T } contract Child<U> extends Parent<U> {} class A {} impl A for Child<Integer> { public fun value(input: Integer) -> Integer { input } } let value = A.new(); (value as Child<Integer>)..value(3) + (value as Parent<Integer>)..value(4)";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
}

#[test]
fn diamond_merges_when_requirements_have_compatible_signatures() {
    let given = "contract Root { fun value(input: Integer) -> Integer } contract Left extends Root {} contract Right extends Root { fun value(other: Integer) -> Integer } contract Child extends Left, Right {} class A {} impl A for Child { public fun value(input: Object) -> Integer { 7 } } (A.new() as Root)..value(1)";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
}

#[test]
fn contract_rejects_when_diamond_requirements_or_arguments_conflict() {
    for given in [
        "contract Left { fun value(input: Integer) -> Integer } contract Right { fun value(input: String) -> String } contract Child extends Left, Right {} 1",
        "contract Parent<T> {} contract Left extends Parent<Integer> {} contract Right extends Parent<String> {} contract Child extends Left, Right {} 1",
    ] {
        let when = execute(given);
        assert_eq!(when, Err(MachineError::TypeContractError));
    }
}

#[test]
fn subclass_inherits_when_base_establishes_generic_conformance() {
    let given = "contract Parent<T> { fun value() -> T } contract Child extends Parent<Integer> {} class Base {} impl Base for Child { public fun value() -> Integer { 7 } } class Derived extends Base { public override fun value() -> Integer { super() + 1 } } let value = Derived.new(); value.value() + (value as Child)..value() + (value as Parent<Integer>)..value()";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(24_u8.into())));
}

#[test]
fn static_impl_rejects_when_full_inherited_signature_changes() {
    for given in [
        "contract Parent { fun value() -> Integer } contract Child extends Parent {} class A { public async fun value() -> Integer { 1 } } impl A for Child {} 1",
        "contract Parent<T> { fun value(input: T) -> T } contract Child extends Parent<Integer> {} class A { public fun value(input: String) -> Integer { 1 } } impl A for Child {} 1",
        "contract Parent { fun value() -> Integer } class Base {} impl Base for Parent { public fun value() -> Integer { 7 } } class Derived extends Base { public override fun value() -> String { \"wrong\" } } 1",
    ] {
        let when = execute(given);
        assert_eq!(when, Err(MachineError::TypeContractError));
    }
}

#[test]
fn inherited_denial_applies_when_origin_has_static_impl() {
    let given = "contract Parent meta deny method_set {} contract Child extends Parent {} class A {} impl A for Child {} open class A { public fun added() -> Nil { nil } } 1";
    let when = execute(given);
    assert!(matches!(
        when,
        Err(MachineError::Class(
            iris_runtime::ClassError::MetaCapabilityDenied { .. }
        ))
    ));
}

#[test]
fn static_impl_can_publish_when_contract_denies_later_method_changes() {
    let given = "contract C meta deny method_set { fun value() -> Integer } class A {} impl A for C { public fun value() -> Integer { 7 } } A.new().value()";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
}

#[test]
fn static_impl_is_missing_when_only_future_open_supplies_method() {
    for given in [
        "contract C { fun value() -> Integer } class A {} open class A { public fun value() -> Integer { 7 } } impl A for C {} 1",
        "contract C { fun value() -> Integer } module M { public fun value() -> Integer { 7 } } class A {} open class A mixin M {} impl A for C {} 1",
        "contract C { fun value() -> Integer } class A {} A.open() { |candidate| candidate.define_method(:value) { || -> Integer; 7 } } impl A for C {} 1",
    ] {
        let when = execute(given);
        assert_eq!(when, Err(MachineError::TypeContractError));
    }
}

#[test]
fn dynamic_open_updates_both_dispatches_when_body_is_compatible() {
    let given = "contract C { fun value() -> Integer } class A {} impl A for C { public fun value() -> Integer { 7 } } let changed = A.open() { |candidate| candidate.define_method(:value, { || -> Integer; 9 }) }; let value = A.new(); value.value() + (value as C)..value()";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(18_u8.into())));
}

#[test]
fn dynamic_open_rolls_back_when_obligation_changes() {
    let given = "contract C { fun value() -> Integer } class A {} impl A for C { public fun value() -> Integer { 7 } } let revision = A.active_revision; let error = try { A.open() { |candidate| candidate.define_method(:value, { || -> String; \"bad\" }) } } catch error { error }; let value = A.new(); %[A.active_revision == revision, value.value() == 7, (value as C)..value() == 7]";
    let when = execute(given);
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(
                true
            );
            3
        ])))
    );
}

#[test]
fn inherited_static_impl_rejects_when_dynamic_open_removes_requirement() {
    let given = "contract C { fun value() -> Integer } class Base {} impl Base for C { public fun value() -> Integer { 7 } } class A extends Base {} let revision = A.active_revision; let rejected = try { A.open() { |candidate| candidate.undef_method(:value) } ; false } catch error { true }; let value = A.new(); %[rejected, A.active_revision == revision, (value as C)..value() == 7]";
    let when = execute(given);
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(
                true
            );
            3
        ])))
    );
}

#[test]
fn existing_view_casts_when_parent_arguments_match() {
    let given = "contract Parent<T> { fun value() -> T } contract Child extends Parent<Integer> {} class A {} impl A for Child { public fun value() -> Integer { 7 } } let child = A.new() as Child; (child as Parent<Integer>)..value()";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
}

#[test]
fn inherited_generic_guard_rejects_when_arguments_differ() {
    let given = "contract Parent<T> { fun value() -> T } contract Child extends Parent<Integer> {} class A {} impl A for Child { public fun value() -> Integer { 7 } } module Guard { public fun check(value: Parent<String>) -> Nil { nil } } Guard.check(A.new())";
    let when = execute(given);
    assert_eq!(when, Err(MachineError::TypeContractError));
}

#[test]
fn empty_impl_selects_when_origin_inherits_a_method() {
    let given = "contract C { fun value() -> Integer } class Base { public fun value() -> Integer { 7 } } class A extends Base {} impl A for C {} (A.new() as C)..value()";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
}

#[test]
fn empty_impl_selects_when_origin_mixes_in_a_method() {
    let given = "contract C { fun value() -> Integer } module M { public fun value() -> Integer { 7 } } class A mixin M {} impl A for C {} (A.new() as C)..value()";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
}

#[test]
fn generic_open_updates_when_closed_obligation_is_preserved() {
    let given = "contract Parent<T> { fun value() -> T } contract Child extends Parent<Integer> {} class A {} impl A for Child { public fun value() -> Integer { 7 } } open class A { public override fun value() -> Integer { 9 } } let value = A.new(); value.value() + (value as Child)..value() + (value as Parent<Integer>)..value()";
    let when = execute(given);
    assert_eq!(when, Ok(Value::Integer(27_u8.into())));
}
