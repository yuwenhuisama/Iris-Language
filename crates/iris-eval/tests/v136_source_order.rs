use iris_eval::{EvaluationError, Session, evaluate};
use iris_runtime::{ArrayRef, Value};

#[test]
fn sends_observe_old_then_new_when_open_follows_a_binding() {
    let given = "class Item { public fun value() { :old } } let before = Item.new().value(); open class Item { override public fun value() { :new } } %[before, Item.new().value()]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("old".into()),
            Value::Symbol("new".into())
        ])))
    );
}

#[test]
fn bound_method_keeps_old_body_when_open_follows_capture() {
    let given = "class Item { public fun value() { :old } } let item = Item.new(); let saved = item.value; open class Item { override public fun value() { :new } } %[saved(), item.value()]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("old".into()),
            Value::Symbol("new".into())
        ])))
    );
}

#[test]
fn entered_frame_keeps_old_body_when_open_commits_while_suspended() {
    let given = "class Item { public async fun value(gate) -> Symbol { let ready = await gate; :old } } let gate = Gate.new(); let entered = Item.new().value(gate); open class Item { override public async fun value(gate) -> Symbol { :new } } let posted = Gate.complete(gate, 1); let before = Host.run(entered); let after = Host.run(Item.new().value(gate)); %[before, after]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("old".into()),
            Value::Symbol("new".into())
        ])))
    );
}

#[test]
fn lazy_class_initializer_sees_binding_when_accessed_after_binding() {
    let given =
        "let seed = 7; class Item { public class property value: Integer = seed } Item.value";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
}

#[test]
fn lazy_class_initializer_cannot_see_binding_when_accessed_before_binding() {
    let given = "class Item { public class property value: Integer = seed } let value = Item.value; let seed = 7; value";
    let when = evaluate(given);
    assert_eq!(when, Err(EvaluationError::NameError));
}

#[test]
fn open_resolves_collected_origin_when_origin_is_textually_later() {
    let given = "let before = Item.new().value(); open class Item { override public fun value() { :new } } let after = Item.new().value(); class Item { public fun value() { :old } } %[before, after, Item.active_revision]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("old".into()),
            Value::Symbol("new".into()),
            Value::Integer(2_u8.into())
        ])))
    );
}

#[test]
fn module_open_runs_at_its_entry_when_origin_is_textually_later() {
    let given = "let before = Helpers.value(); open module Helpers { override public fun value() { :new } } let after = Helpers.value(); module Helpers { public fun value() { :old } } %[before, after]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("old".into()),
            Value::Symbol("new".into())
        ])))
    );
}

#[test]
fn failed_open_preserves_prior_binding_and_revision_when_body_raises() {
    let mut given = Session::new().unwrap();
    let when = given.evaluate("class Item { public fun value() { :old } } let before = Item.new().value(); open class Item { override public fun value() { :new } raise :stop } let after = :unreachable; 0");
    assert_eq!(
        when,
        Err(EvaluationError::Raised(Value::Symbol("stop".into())))
    );
    assert_eq!(
        given.evaluate("%[before, Item.new().value(), Item.active_revision]"),
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Symbol("old".into()),
            Value::Symbol("old".into()),
            Value::Integer(1_u8.into())
        ])))
    );
    assert_eq!(given.evaluate("after"), Err(EvaluationError::NameError));
}

#[test]
fn instance_initializers_run_per_construction_when_origins_are_collected() {
    let given = "global mut $next = 0; class Item { let @id = ($next = $next + 1); public fun id() { @id } } let before = $next; let first = Item.new(); let second = Item.new(); %[before, first.id(), second.id()]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(0_u8.into()),
            Value::Integer(1_u8.into()),
            Value::Integer(2_u8.into())
        ])))
    );
}

#[test]
fn ordinary_origins_reject_executable_outer_bodies() {
    for given in [
        "class Item { raise :stop } Item",
        "module Helpers { raise :stop } Helpers",
    ] {
        let when = evaluate(given);
        assert_eq!(when, Err(EvaluationError::ParseDiagnostic));
    }
}

#[test]
fn lazy_initializer_runs_once_when_first_read_follows_origin_publication() {
    let given = "mut calls = 0; class Item { public class property value: Integer = (calls = calls + 1) } let before = calls; let first = Item.value; let second = Item.value; %[before, first, second, calls]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(0_u8.into()),
            Value::Integer(1_u8.into()),
            Value::Integer(1_u8.into()),
            Value::Integer(1_u8.into())
        ])))
    );
}

#[test]
fn assignment_supersedes_lazy_initializer_when_property_has_not_been_read() {
    let given = "class Item { public class property value: Integer = missing } let assigned = Item.value = 7; Item.value";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
}

#[test]
fn open_property_initializer_runs_at_transaction_entry_when_binding_exists() {
    let given = "class Item {} mut calls = 0; open class Item { public class property value: Integer = (calls = calls + 1) } %[calls, Item.value]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u8.into()),
            Value::Integer(1_u8.into())
        ])))
    );
}

#[test]
fn static_impl_is_published_once_when_textually_after_execution() {
    let given = "export class Item {} let before = (Item.new() as Read)..value(); open class Item { override public fun value() -> Integer { 2 } } impl Item for Read { public fun value() -> Integer { 1 } } export contract Read { fun value() -> Integer } %[before, Item.new().value(), Item.active_revision]";
    let when = evaluate(given);
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(1_u8.into()),
            Value::Integer(2_u8.into()),
            Value::Integer(2_u8.into())
        ])))
    );
}

#[test]
fn lazy_initializer_uses_declaring_owner_when_read_from_outside_class() {
    let given = "class Item { private class property seed: Integer = 7; public class property value: Integer = self.seed } Item.value";
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(7_u8.into())));
}
