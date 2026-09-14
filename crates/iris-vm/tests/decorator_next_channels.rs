#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

fn check_rejected_attempt(call: &str, error_type: &str, is_async: bool) {
    let (mode, result_type, wait, entry) = if is_async {
        (
            "async",
            "Task<Object>",
            "await",
            "Host.run(Target.new().value())",
        )
    } else {
        ("", "Object", "", "Target.new().value()")
    };
    let attempt = if is_async {
        format!(
            r#"
            let first = {call}
            let second = {call}
            if first same? second {{ raise :reused_task }}
            if !(first is? Task<Object>) {{ raise :wrong_task_type }}
            if first is? Task<Integer> {{ raise :wrong_task_type }}
let first_error = try {{ await first; false }} catch error {{ error is? {error_type} }}
let second_error = try {{ await second; false }} catch error {{ error is? {error_type} }}
            first_error && second_error
            "#
        )
    } else {
        format!("try {{ {call}; false }} catch error {{ error is? {error_type} }}")
    };
    let given = format!(
        r#"
class Effects {{
 public class property inner: Integer = 0
 public class property body: Integer = 0
 public class property rejected: Object = nil
 public class property before: Object = nil
}}
class Reject {{}}
impl Reject for MethodDecorator {{
 public fun plan(declaration, arguments) -> Plan {{ Plan.empty }}
 public fun transform(declaration, arguments, context) -> Transformation {{
  Transformation.wrap_method({{ {mode} |invocation: Invocation, next: Closure<(ArgumentChanges) -> {result_type}>| -> Object;
   let rejected = try {{ {attempt} }} catch error {{ raise }}
   Effects.rejected = rejected
   Effects.before = %[Effects.inner, Effects.body]
    let ordinary = Ordinary.new().accept({{ || -> Integer; 3 }})
    if ordinary != 3 {{ raise :leaked_block }}
   {wait} next.call(ArgumentChanges.empty)
  }})
 }}
}}
class Inner {{}}
impl Inner for MethodDecorator {{
 public fun plan(declaration, arguments) -> Plan {{ Plan.empty }}
 public fun transform(declaration, arguments, context) -> Transformation {{
  Transformation.wrap_method({{ {mode} |invocation: Invocation, next: Closure<(ArgumentChanges) -> {result_type}>| -> Object;
   Effects.inner = Effects.inner + 1
   {wait} next.call()
  }})
 }}
}}
class Ordinary {{ public fun accept(value: Closure<() -> Integer>) -> Integer {{ value.call() }} }}
class Target {{
 @Reject() @Inner()
 public {mode} fun value() -> Integer {{ Effects.body = Effects.body + 1; 7 }}
}}
let result = {entry}
%[Effects.rejected, Effects.before, result, Effects.inner, Effects.body]
"#
    );
    let parsed = iris_parser::parse(&given);
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    let when = run(&compile(&given).expect("next-channel scenario compiles"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Array(ArrayRef::new(vec![
                Value::Integer(0_u64.into()),
                Value::Integer(0_u64.into()),
            ])),
            Value::Integer(7_u64.into()),
            Value::Integer(1_u64.into()),
            Value::Integer(1_u64.into()),
        ])))
    );
}

macro_rules! rejection_cases {
    ($($name:ident: $call:literal, $error:literal, $is_async:literal;)*) => {
        $(#[test]
        fn $name() {
            check_rejected_attempt($call, $error, $is_async);
        })*
    };
}

rejection_cases! {
    sync_retry_when_keyword_rejected: "next.call(changes: ArgumentChanges.empty)", "ArgumentError", false;
    async_retry_when_keyword_rejected: "next.call(changes: ArgumentChanges.empty)", "ArgumentError", true;
    sync_retry_when_block_rejected: "next.call() { || -> Integer; 9 }", "ArgumentError", false;
    async_retry_when_block_rejected: "next.call() { || -> Integer; 9 }", "ArgumentError", true;
    sync_retry_when_positional_closure_rejected: "next.call({ || -> Integer; 9 })", "TypeError", false;
    async_retry_when_positional_closure_rejected: "next.call({ || -> Integer; 9 })", "TypeError", true;
    sync_retry_when_positional_value_rejected: "next.call(nil)", "TypeError", false;
    async_retry_when_positional_value_rejected: "next.call(nil)", "TypeError", true;
}
