#![expect(clippy::expect_used, reason = "tests require compiled source")]

use iris_runtime::Value;
use iris_vm::{compile, run};

#[test]
fn block_alias_equals_union_when_reified_directly() {
    let given =
        "(Block<() -> Integer>).type == (BoundMethod<() -> Integer> | Closure<() -> Integer>).type";
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn bound_block_preserves_receiver_when_called_through_cast() {
    let given = r#"
class Counter { public fun value() -> Integer { 42 } }
class Target { public fun apply(&block: Block<() -> Integer>) -> Integer { block.call() } }
let callback = Counter.new().value
let held = callback as Block<() -> Integer>
%[Target.new().apply(&held), held same? callback, callback is? Closure<() -> Integer>, callback is? BoundMethod<() -> String>]
"#;
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Integer(42_u64.into()),
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(false),
        ])))
    );
}

#[test]
fn async_bound_returns_task_when_cast_to_block() {
    let given = r#"
class Worker { public async fun answer() -> Integer { 42 } }
let callback = Worker.new().answer as Block<() -> Task<Integer>>
let task = callback.call()
%[task is? Task<Integer>, Host.run(task)]
"#;
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(42_u64.into()),
        ])))
    );
}

#[test]
fn replacement_accepts_bound_block_when_signature_matches() {
    let given = r#"
class Helper { public fun answer() -> Integer { 42 } }
class Patch {}
impl Patch for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   next.call(ArgumentChanges.new(block: Helper.new().answer))
  })
 }
}
class Target { @Patch() public fun value(&block: Block<() -> Integer> = nil) -> Integer { block.call() } }
Target.new().value()
"#;
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(when, Ok(Value::Integer(42_u64.into())));
}

#[test]
fn signature_defaults_are_omitted_when_callable_types_match() {
    let given = "let callback = { |value: Integer = 7| -> Integer; value }; callback is? Closure<(Integer) -> Integer> && callback is? Block<(Integer) -> Integer> && !(callback is? Closure<(Object) -> Integer>)";
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn ordinary_block_rejects_bound_method_when_signature_differs() {
    let given = r#"
class Helper { public fun answer() -> String { "wrong" } }
class Target { public fun value(&block: Block<() -> Integer>) -> Integer { raise :entered } }
try { Target.new().value(&Helper.new().answer) } catch error { error == :TypeContractError }
"#;
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn native_array_block_accepts_bound_method_when_receiver_is_retained() {
    let given = r#"
class Helper { public fun initialize() { @base = 40 } public fun answer(value: Integer) -> Integer { @base + value } }
let mapped = %[1, 2].map(&Helper.new().answer)
mapped[1]
"#;
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(when, Ok(Value::Integer(42_u64.into())));
}

#[test]
fn bound_types_remain_distinct_when_generic_receivers_share_method() {
    let given = r#"
class Box<T> { public fun initialize(value: T) { @value = value } public fun get() -> T { @value } }
let integer = Box<Integer>.new(42).get
let text = Box<String>.new("text").get
%[integer is? Block<() -> Integer>, text is? Block<() -> String>, integer is? Block<() -> String>, integer.call()]
"#;
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(false),
            Value::Integer(42_u64.into()),
        ])))
    );
}

#[test]
fn unsupported_callable_signature_rejects_when_type_is_unknown() {
    let given = "let callback = { || -> Missing; nil }; callback is? Closure<() -> Object>";
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(when, Ok(Value::Bool(false)));
}

#[test]
fn replacement_rejects_bound_block_when_signature_differs() {
    let given = r#"
class Helper { public fun answer() -> String { "wrong" } }
class Patch {}
impl Patch for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   next.call(ArgumentChanges.new(block: Helper.new().answer))
  })
 }
}
class Target { @Patch() public fun value(&block: Block<() -> Integer> = nil) -> Integer { raise :entered } }
try { Target.new().value() } catch error { error is? TypeError }
"#;
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(when, Ok(Value::Bool(true)));
}

#[test]
fn direct_cast_call_resolves_when_closure_signature_is_exact() {
    let given =
        "({ |value: Integer| -> Integer; value } as Closure<(Integer) -> Integer>).call(42)";
    let when = run(&compile(given).expect("source compiles"));
    assert_eq!(when, Ok(Value::Integer(42_u64.into())));
}

#[test]
fn wrapper_defaults_are_rejected_when_signature_type_would_match() {
    let given = r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
  Transformation.wrap_method({ |invocation: Invocation = nil, next: Closure<(ArgumentChanges) -> Object> = nil| -> Object; next.call() })
 }
}
class Target { @Wrap() public fun answer() -> Integer { 42 } }
Target.new().answer()
"#;
    let when = run(&compile(given).expect("source compiles"));
    assert!(
        matches!(when, Err(iris_vm::MachineError::Raised(_))),
        "{when:?}"
    );
}
