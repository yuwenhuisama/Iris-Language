use iris_runtime::{Capability, ClassError, Value};
use iris_vm::{Machine, MachineError};

fn execute(source: &str) -> Result<Value, MachineError> {
    let program = iris_vm::compile(source).expect("compile property capability fixture");
    Machine::new().expect("machine").execute(&program)
}

#[test]
fn rejects_addition_when_property_set_is_denied() {
    let given = "class Target meta deny property_set {}; open class Target { public property fun value() -> Integer { 1 } }; print(Target.new().value)";

    let when = execute(given);

    assert!(
        matches!(
            when,
            Err(MachineError::Class(ClassError::MetaCapabilityDenied {
                operation: Capability::PropertySet,
                ..
            }))
        ),
        "{when:?}"
    );
}

#[test]
fn allows_addition_when_only_property_body_is_denied() {
    let given = "class Target meta deny property_body, method_set, method_body {}; open class Target { public property fun value() -> Integer { 1 } }; Target.new().value";

    let when = execute(given);

    assert_eq!(when, Ok(Value::Integer(1_u64.into())));
}

#[test]
fn rejects_replacement_when_property_body_is_denied() {
    for origin in [
        "class Target meta deny property_body { public property fun value() -> Integer { 1 } }",
        "class Base { public property fun value() -> Integer { 1 } }; class Target extends Base meta deny property_body {}",
        "module Source { public property fun value() -> Integer { 1 } }; class Target mixin Source meta deny property_body {}",
    ] {
        let given = format!(
            "{origin}; open class Target {{ public override property fun value() -> Integer {{ 2 }} }}; Target.new().value"
        );

        let when = execute(&given);

        assert!(
            matches!(
                when,
                Err(MachineError::Class(ClassError::MetaCapabilityDenied {
                    operation: Capability::PropertyBody,
                    ..
                }))
            ),
            "{given}: {when:?}"
        );
    }
}

#[test]
fn allows_replacement_when_only_property_set_is_denied() {
    for origin in [
        "class Target meta deny property_set, method_set, method_body { public property fun value() -> Integer { 1 } }",
        "class Base { public property fun value() -> Integer { 1 } }; class Target extends Base meta deny property_set, method_set, method_body {}",
        "module Source { public property fun value() -> Integer { 1 } }; class Target mixin Source meta deny property_set, method_set, method_body {}",
    ] {
        let given = format!(
            "{origin}; open class Target {{ public override property fun value() -> Integer {{ 2 }} }}; Target.new().value"
        );

        let when = execute(&given);

        assert_eq!(when, Ok(Value::Integer(2_u64.into())), "{given}");
    }
}

#[test]
fn checks_addition_separately_when_the_other_accessor_exists() {
    for (existing, added, probe) in [
        (
            "public property fun value() -> Integer { 1 }",
            "public property fun value=(value: Integer) -> Integer { value + 1 }",
            "let target = Target.new(); target.value = 2",
        ),
        (
            "public property fun value=(value: Integer) -> Integer { value }",
            "public property fun value() -> Integer { 3 }",
            "Target.new().value",
        ),
    ] {
        for (denied, allowed) in [("property_set", false), ("property_body", true)] {
            let given = format!(
                "class Target meta deny {denied} {{ {existing} }}; open class Target {{ {added} }}; {probe}"
            );

            let when = execute(&given);

            if allowed {
                assert_eq!(when, Ok(Value::Integer(3_u64.into())), "{given}");
            } else {
                assert!(
                    matches!(
                        when,
                        Err(MachineError::Class(ClassError::MetaCapabilityDenied {
                            operation: Capability::PropertySet,
                            ..
                        }))
                    ),
                    "{given}: {when:?}"
                );
            }
        }
    }
}

#[test]
fn checks_setter_replacement_when_the_getter_is_unchanged() {
    for (denied, allowed) in [("property_body", false), ("property_set", true)] {
        let given = format!(
            "class Target meta deny {denied} {{ public property fun value() -> Integer {{ 1 }}; public property fun value=(value: Integer) -> Integer {{ value }} }}; open class Target {{ public override property fun value=(value: Integer) -> Integer {{ value + 1 }} }}; let target = Target.new(); target.value == 1 && (target.value = 2) == 3"
        );

        let when = execute(&given);

        if allowed {
            assert_eq!(when, Ok(Value::Bool(true)));
        } else {
            assert!(
                matches!(
                    when,
                    Err(MachineError::Class(ClassError::MetaCapabilityDenied {
                        operation: Capability::PropertyBody,
                        ..
                    }))
                ),
                "{when:?}"
            );
        }
    }
}

#[test]
fn preserves_static_promises_when_body_replacement_is_allowed() {
    let given = "class Target meta deny property_set { public property fun value() -> Integer { 1 } }; open class Target { public override property fun value() -> String { \"bad\" } }; Target.new().value";

    let when = execute(given);

    assert_eq!(when, Err(MachineError::TypeContractError));
}
