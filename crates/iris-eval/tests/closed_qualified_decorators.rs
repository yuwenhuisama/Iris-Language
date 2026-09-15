use iris_eval::{Session, evaluate};
use iris_runtime::Value;

const WRAP: &str = r#"
class Wrap {}
impl Wrap for MethodDecorator {
    public fun plan(d, a) -> Plan { Plan.empty }
    public fun transform(d, a, c) -> Transformation {
        mut calls = 0
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            calls = calls + 1
            if invocation.slot[2] == nil {
                if invocation.slot[0] != Box<Integer>.type { raise :owner }
            } else {
                if invocation.owner_type_arguments[0] == Integer.type && invocation.slot[2] != Named<Integer>.type { raise :qualifier }
                if invocation.owner_type_arguments[0] == String.type && invocation.slot[2] != Named<String>.type { raise :qualifier }
            }
            if invocation.owner_type_arguments[0] == Integer.type && invocation.slot[0] != Box<Integer>.type { raise :owner }
            if invocation.owner_type_arguments[0] == String.type && invocation.slot[0] != Box<String>.type { raise :owner }
            Effects.calls = calls
            next.call()
        })
    }
}
class Effects { public class property calls: Integer = 0 }
contract Named<Element> { fun value(value: Element) -> Element }
class Box<T> {}
impl Box<T> for Named<T> {
    @Wrap() public fun value(value: T) -> T { value }
}
"#;

#[test]
fn closed_views_keep_exact_slots_when_owners_alternate() {
    let given = format!(
        r#"{WRAP}
        let integer = Box<Integer>.new()
        let string = Box<String>.new()
        let first = (integer as Named<Integer>)..value(7)
        let middle = (string as Named<String>)..value("s")
        let last = (integer as Named<Integer>)..value(9)
        let ordinary = integer.value(11)
        %[first, middle, last, Effects.calls, ordinary,
         try {{ integer as Named<String>; false }} catch error {{ true }}]
    "#
    );
    let when = evaluate(&given);
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(7u64.into()),
            Value::Text("s".into()),
            Value::Integer(9u64.into()),
            Value::Integer(3u64.into()),
            Value::Integer(11u64.into()),
            Value::Bool(true),
        ])))
    );
}

#[test]
fn owner_and_method_arguments_are_separate_when_names_shadow() {
    let given = r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                    if invocation.owner_type_arguments[0] != Integer.type { raise :owner }
                    if invocation.method_type_arguments[0] != String.type { raise :method }
                    if invocation.slot[2] != Named<Integer>.type { raise :qualifier }
                    next.call()
                })
            }
        }
        contract Named<Element> { fun value<T>(value: T) -> T }
        class Box<T> {}
        impl Box<T> for Named<T> {
            @Wrap() public fun value<T>(value: T) -> T { value }
        }
        (Box<Integer>.new() as Named<Integer>)..value<String>("s")
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Text("s".into())));
}

#[test]
fn async_closed_qualifiers_survive_when_owners_suspend_together() {
    let given = r#"
        class Wrap {}
        impl Wrap for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
                    let result = await next.call()
                    if invocation.owner_type_arguments[0] == Integer.type {
                        if invocation.slot[2] != Named<Integer>.type || invocation.slot[0] != Box<Integer>.type { raise :qualifier }
                    } else {
                        if invocation.slot[2] != Named<String>.type || invocation.slot[0] != Box<String>.type { raise :qualifier }
                    }
                    result
                })
            }
        }
        contract Named<Element> { async fun value(value: Element, gate) -> Element }
        class Box<T> {}
        impl Box<T> for Named<T> {
            @Wrap() public async fun value(value: T, gate) -> T { await gate; value }
        }
        let first_gate = Gate.new()
        let second_gate = Gate.new()
        let first = (Box<Integer>.new() as Named<Integer>)..value(7, first_gate)
        let second = (Box<String>.new() as Named<String>)..value("s", second_gate)
        let posted_second = Gate.complete(second_gate, nil)
        let second_result = Host.run(second)
        let posted_first = Gate.complete(first_gate, nil)
        %[first is? Task<Integer>, Host.run(first), second_result]
    "#;
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[true, 7, \"s\"]"));
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
    let when = evaluate(given);
    assert_eq!(when, evaluate("%[true, 0]"));
}

#[test]
fn wrong_input_is_rejected_when_closed_requirement_supplies_annotation() {
    let mut given = Session::new().unwrap();
    given.evaluate(&format!("{WRAP}; 0")).unwrap();
    let when = given.evaluate(
        r#"
        try { (Box<Integer>.new() as Named<Integer>)..value("wrong") }
        catch error { error is? TypeError }
    "#,
    );
    assert_eq!(when, Ok(Value::Bool(true)));
    assert_eq!(given.evaluate("Effects.calls"), evaluate("0"));
}

#[test]
fn wrong_result_is_rejected_when_qualified_wrapper_skips_original() {
    let given = r#"
        class Wrong {}
        impl Wrong for MethodDecorator {
            public fun plan(d, a) -> Plan { Plan.empty }
            public fun transform(d, a, c) -> Transformation {
                Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object; "wrong" })
            }
        }
        contract Named<Element> { fun value(value: Element) -> Element }
        class Box<T> {}
        impl Box<T> for Named<T> {
            @Wrong() public fun value(value: T) -> T { raise :body }
        }
        try { (Box<Integer>.new() as Named<Integer>)..value(1) }
        catch error { error is? TypeError }
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Bool(true)));
}
