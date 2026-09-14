#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

#[test]
fn nominal_owner_is_distinct_when_compared_with_its_class_object() {
    let given = r#"
        class Target {}
        %[Target.type == Target, Target.type != Target,
          Target == Target.type, Target != Target.type,
          Target.type == Target.type, Target.type != Target.type]
    "#;
    let when = run(&compile(given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(false),
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(false),
        ])))
    );
}

#[test]
fn closed_owner_is_distinct_when_compared_with_its_closed_class_object() {
    let given = r#"
        class Target<Element> {}
        let integer = Target<Integer>
        let string = Target<String>
        %[integer.type == integer, integer.type != integer,
          integer == integer.type, integer != integer.type,
          integer.type == integer.type, integer.type != string.type]
    "#;
    let when = run(&compile(given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(false),
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(true),
            Value::Bool(true),
            Value::Bool(true),
        ])))
    );
}
