use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

#[test]
fn retained_method_keeps_original_body_when_class_reopens_in_same_program() -> Result<(), String> {
    let given = compile(
        r#"
class Target {
 public fun value() -> Integer { 17 }
 public fun map_value(value: Integer) -> Integer { value + 17 }
}
let target = Target.new()
let retained = target.value
let callback = target.map_value
open class Target {
 public override fun value() -> Integer { 29 }
 public override fun map_value(value: Integer) -> Integer { value + 29 }
}
%[retained.call(), target.value(), %[1].map(&callback)[0]]
"#,
    )
    .map_err(|error| format!("{error:?}"))?;

    let when = run(&given);

    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(17_u64.into()),
            Value::Integer(29_u64.into()),
            Value::Integer(18_u64.into()),
        ])))
    );
    Ok(())
}
