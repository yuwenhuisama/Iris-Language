#![expect(
    clippy::expect_used,
    reason = "language revision fixtures must compile"
)]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{MachineError, compile, run};

fn execute(source: &str) -> Result<Value, MachineError> {
    run(&compile(source).expect("v1.36 source compiles"))
}

#[test]
fn nonnull_preserves_one_evaluation_and_raises_canonical_type_error() {
    let preserved = execute(
        "global mut $calls = 0; class Item {}; module Factory { public fun make() { $calls = $calls + 1; Item.new() } }; let item = Factory.make()!; %[item same? item, $calls]",
    );
    let rejected = execute(
        "try { nil! } catch error: TypeError, context { %[error is? TypeError, context.value same? error] }",
    );

    assert_eq!(
        preserved,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(1_u8.into()),
        ])))
    );
    assert_eq!(
        rejected,
        Ok(Value::Array(ArrayRef::new(vec![Value::Bool(true); 2])))
    );
}

#[test]
fn instance_fields_initialize_per_instance_and_guard_writes() {
    let initialized = execute(
        "global mut $next = 0; class Item { mut @id: Integer = ($next = $next + 1); public fun id() -> Integer { @id } } let first = Item.new(); let second = Item.new(); %[first.id(), first.id(), second.id()]",
    );
    let immutable = execute(
        "class Item { let @id = 1; public fun replace() { @id = 2 } } Item.new().replace()",
    );
    let typed = execute(
        "class Item { mut @id: Integer = 1; public fun replace() { @id = \"bad\" } } Item.new().replace()",
    );

    assert_eq!(
        initialized,
        Ok(Value::Array(ArrayRef::new(
            [1_u8, 1, 2]
                .map(|value| Value::Integer(value.into()))
                .to_vec(),
        )))
    );
    assert_eq!(immutable, Err(MachineError::ImmutableBinding));
    assert_eq!(typed, Err(MachineError::TypeContractError));
}

#[test]
fn inherited_instance_field_guards_apply_in_subclass_methods() {
    let source = "class Base { let @id: Integer = 1 } class Child extends Base { public fun replace() { @id = 2 } } Child.new().replace()";

    assert_eq!(execute(source), Err(MachineError::ImmutableBinding));
}

#[test]
fn duplicate_instance_fields_are_rejected() {
    let source = "class Item { let @id = 1; mut @id = 2 } Item.new()";

    assert_eq!(execute(source), Err(MachineError::TypeContractError));
}

#[test]
fn static_impl_is_order_independent_and_populates_both_method_surfaces() {
    let source = "impl Box for Show { public fun show() -> String { \"box\" } } contract Show { fun show() -> String } class Box {} let box = Box.new(); %[box.show(), (box as Show)..show()]";

    assert_eq!(
        execute(source),
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Text("box".into()),
            Value::Text("box".into()),
        ])))
    );
}

#[test]
fn empty_impl_reuses_a_compatible_class_method() {
    let source = "contract Show { fun show() -> String } class Box { public fun show() -> String { \"box\" } } impl Box for Show {} (Box.new() as Show)..show()";

    assert_eq!(execute(source), Ok(Value::Text("box".into())));
}

#[test]
fn impl_rejects_duplicate_missing_extra_and_mismatched_methods() {
    for source in [
        "contract C { fun m() -> Nil } class A {} impl A for C { fun m() -> Nil { nil } } impl A for C { fun m() -> Nil { nil } } A",
        "contract C { fun m() -> Nil } class A {} impl A for C {} A",
        "contract C { fun m() -> Nil } class A {} impl A for C { fun m() -> Nil { nil } fun extra() -> Nil { nil } } A",
        "contract C { fun m(value: Integer) -> Nil } class A {} impl A for C { fun m(value: String) -> Nil { nil } } A",
    ] {
        assert_eq!(
            execute(source),
            Err(MachineError::TypeContractError),
            "{source}"
        );
    }
}

#[test]
fn impl_rejects_unknown_classes_and_contracts() {
    for source in [
        "contract C {} impl Missing for C {} nil",
        "class A {} impl A for Missing {} nil",
    ] {
        assert_eq!(
            execute(source),
            Err(MachineError::TypeContractError),
            "{source}"
        );
    }
}

#[test]
fn duplicate_origins_are_rejected_instead_of_merged() {
    for source in [
        "class Same {} class Same {} Same",
        "module Same {} module Same {} Same",
    ] {
        assert_eq!(
            execute(source),
            Err(MachineError::ParseDiagnostic),
            "{source}"
        );
    }
}

#[test]
fn class_and_module_origins_remain_distinct_namespaces() {
    let source = "class Same {} module Same {} Same";

    assert_eq!(execute(source), Err(MachineError::ParseDiagnostic));
}

#[test]
fn open_class_mixin_and_body_publish_atomically() {
    let source = "module Named { public fun name() -> String { \"mixed\" } } class Item {} open class Item mixin Named { public fun own() -> String { \"own\" } } let item = Item.new(); %[item.name(), item.own()]";

    assert_eq!(
        execute(source),
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Text("mixed".into()),
            Value::Text("own".into()),
        ])))
    );
}

#[test]
fn open_class_mixin_revalidates_static_impl_obligations() {
    let mismatched = "contract C { fun draw(value: Integer) -> Nil } module Paint { public fun draw(value: String) -> Nil { nil } } class Item {} open class Item mixin Paint {} impl Item for C {} Item";
    let matching = "contract C { fun draw(value: Integer) -> Nil } module Paint { public fun draw(value: Integer) -> Nil { nil } } class Item {} open class Item mixin Paint {} impl Item for C {} Item.new().draw(1)";

    assert_eq!(execute(mismatched), Err(MachineError::TypeContractError));
    assert_eq!(execute(matching), Err(MachineError::TypeContractError));
}

#[test]
fn open_replaces_a_compatible_static_impl_body() {
    let source = "contract C { fun m() -> Integer } class A {} impl A for C { public fun m() -> Integer { 1 } } open class A { public override fun m() -> Integer { 2 } } let value = A.new(); value.m() + (value as C)..m()";

    assert_eq!(execute(source), Ok(Value::Integer(4_u8.into())));
}

#[test]
fn dynamic_open_rejects_a_static_impl_replacement_without_promised_return_type() {
    let source = "contract C { fun m() -> Integer } class A {} impl A for C { public fun m() -> Integer { 1 } } A.open() { |candidate| candidate.define_method(:m) { 2 } }";

    assert_eq!(execute(source), Err(MachineError::TypeContractError));
}
