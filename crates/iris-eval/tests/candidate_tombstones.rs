use iris_eval::{EvaluationError, evaluate};
use iris_runtime::Value;

const CLASSES: &str = r#"
    class Parent { public fun total() -> Object { "bad" } }
    class Rule extends Parent { public override fun total() -> Integer { 10 } }
"#;

#[test]
fn undef_succeeds_when_callback_blocks_incompatible_ancestor() {
    let given = format!(
        "{CLASSES} let changed = Rule.open({{ |candidate|; candidate.undef_method(:total) }}); Rule.method(:total)"
    );
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Nil));
}

#[test]
fn undef_succeeds_when_callback_discards_intermediate_bad_replacement() {
    let given = format!(
        "{CLASSES} let changed = Rule.open({{ |candidate|; candidate.define_method(:total) {{ \"bad\" }}; candidate.undef_method(:total) }}); Rule.method(:total)"
    );
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Nil));
}

#[test]
fn removal_succeeds_when_active_ancestor_tombstone_blocks_fallback() {
    let given = r#"
        class Grandparent { public fun total() -> Object { "bad" } }
        class Parent extends Grandparent { }
        let hidden = Parent.undef_method(:total)
        class Rule extends Parent { public fun total() -> Integer { 10 } }
        let changed = Rule.open({ |candidate|; candidate.remove_method(:total) })
        Rule.method(:total)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Nil));
}

#[test]
fn removal_succeeds_when_staged_ancestor_tombstone_blocks_fallback() {
    let given = r#"
        class Grandparent { public fun total() -> Object { "bad" } }
        class Parent extends Grandparent { }
        class Rule extends Parent { public override fun total() -> Integer { 10 } }
        let changed = Parent.open({ |parent|;
            parent.undef_method(:total)
            Rule.open({ |candidate|; candidate.remove_method(:total) })
        })
        Rule.method(:total)
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Nil));
}

#[test]
fn removal_rejects_when_inherited_signature_breaks_promise() {
    let given = format!(
        "{CLASSES} let changed = Rule.open({{ |candidate|; candidate.remove_method(:total) }}); Rule.new().total()"
    );
    let when = evaluate(&given);
    assert_eq!(when, Err(EvaluationError::TypeContractError));
}

#[test]
fn removal_succeeds_when_inherited_signature_preserves_promise() {
    let given = r#"
        class Parent { public fun total() -> Integer { 11 } }
        class Rule extends Parent { public override fun total() -> Integer { 10 } }
        let changed = Rule.open({ |candidate|; candidate.remove_method(:total) })
        Rule.new().total()
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(11_u64.into())));
}
