#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

const SOURCE: &str = include_str!("closed_qualified.iris");

#[test]
fn async_method_arguments_remain_separate_when_owner_parameter_is_shadowed() {
    let given = include_str!("closed_qualified_async.iris")
        .replace(
            "async fun value(value: Element, gate) -> Element",
            "async fun value<T>(value: T, gate) -> T",
        )
        .replace(
            "async fun value(value: T, gate) -> T",
            "async fun value<T>(value: T, gate) -> T",
        )
        .replace("..value(7,", "..value<Integer>(7,")
        .replace("..value(\"s\",", "..value<String>(\"s\",");
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(7_u64.into()),
            Value::Text("s".into())
        ])))
    );
}

#[test]
fn fresh_chain_is_selected_when_qualified_body_is_opened() {
    let given = format!(
        r#"{SOURCE}
        let box = Box<Integer>.new()
        let first = (box as Named<Integer>)..value(7)
        open class Box {{ @Wrap() public override fun value(value: T) -> T {{ value + 10 }} }}
        %[first, (box as Named<Integer>)..value(7), Effects.calls]
    "#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Integer(17_u64.into()),
            Value::Integer(1_u64.into())
        ])))
    );
}

#[test]
fn closed_views_share_ordinary_body_when_owners_alternate() {
    let given = format!(
        r#"{SOURCE}
        let integer = Box<Integer>.new()
        let string = Box<String>.new()
        let first = (integer as Named<Integer>)..value(7)
        let middle = (string as Named<String>)..value("s")
        let last = (integer as Named<Integer>)..value(9)
        let ordinary = integer.value(11)
        %[first, middle, last, Effects.calls, ordinary,
         try {{ integer as Named<String>; false }} catch error {{ error is? TypeError }}]
    "#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Text("s".into()),
            Value::Integer(9_u64.into()),
            Value::Integer(3_u64.into()),
            Value::Integer(11_u64.into()),
            Value::Bool(true),
        ])))
    );
}

#[test]
fn wrong_input_has_zero_effect_when_impl_annotation_binds_owner_type() {
    let given = format!(
        r#"{SOURCE}
        let failed = try {{ (Box<Integer>.new() as Named<Integer>)..value("wrong"); false }}
        catch error {{ error is? TypeError }}
        %[failed, Effects.calls]
    "#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(0_u64.into())
        ])))
    );
}

#[test]
fn wrong_result_is_rejected_when_wrapper_skips_original() {
    let given = format!(
        r#"{}
        try {{ (Box<Integer>.new() as Named<Integer>)..value(1); false }}
        catch error {{ error is? TypeError }}
    "#,
        SOURCE.replace("next.call()", "\"wrong\"")
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn owner_and_method_arguments_are_separate_when_names_shadow() {
    let given = format!(r#"{}
        (Box<Integer>.new() as Named<Integer>)..value<String>("s")
    "#, SOURCE.replace("fun value(value: Element) -> Element", "fun value<T>(value: T) -> T")
        .replace("fun value(value: T) -> T", "fun value<T>(value: T) -> T")
        .replace("Effects.calls = calls", "if invocation.method_type_arguments[0] != String.type { raise :method }; if invocation.slot[2] != Named<Integer>.type { raise :qualifier }; Effects.calls = calls"));
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(when, Ok(Value::Text("s".into())));
}

#[test]
fn wrong_block_result_is_rejected_when_patch_precedes_inner_wrapper() {
    let given = r#"
        class Effects { public class property inner: Integer = 0 }
        class Patch {}
        impl Patch for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    next.call(ArgumentChanges.new(block: { || -> Object; 1 }))
                })
            }
        }
        class Inner {}
        impl Inner for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    Effects.inner = Effects.inner + 1; next.call()
                })
            }
        }
        contract Named<Element> { fun value(&block: Block<() -> Element>) -> Element }
        class Box<T> {}
        impl Box<T> for Named<T> {
            @Patch() @Inner() public fun value(&block: Block<() -> T>) -> T { raise :body }
        }
        let failed = try {
            (Box<Integer>.new() as Named<Integer>)..value() { || -> Integer; 1 }
            false
        } catch error { error is? TypeError }
        %[failed, Effects.inner]
    "#;
    let when = run(&compile(given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(0_u64.into())
        ])))
    );
}

#[test]
fn async_closed_qualifiers_survive_when_owners_suspend_together() {
    let given = include_str!("closed_qualified_async.iris");
    let when = run(&compile(given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(7_u64.into()),
            Value::Text("s".into())
        ])))
    );
}
