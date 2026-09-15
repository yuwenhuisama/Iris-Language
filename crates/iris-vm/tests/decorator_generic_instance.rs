#![expect(clippy::expect_used, reason = "tests require compiled bytecode")]

use iris_runtime::{ArrayRef, Value};
use iris_vm::{compile, run};

const SOURCE: &str = r#"
class Effects { public class property transforms: Integer = 0; public class property bodies: Integer = 0 }
class Element { public class fun marker() -> Integer { 9 } }
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(d, a) -> Plan { Plan.empty }
 public fun transform(d, a, c) -> Transformation {
   if !(d.owner is? Type) || d.selector != :echo || d.kind != :method { raise :declaration }
   if c.reason == :closed_materialization && d.owner != Target.type { raise :owner }
  Effects.transforms = Effects.transforms + 1
  mut cached: Object = nil
  Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
   if Element.marker() != 9 { raise :lexical }
     if !(invocation.receiver is? Target) || !(invocation.slot[0] is? Type) || invocation.slot[0] != Target.type || invocation.slot[0] == Target { raise :receiver }
    if invocation.slot[2] != nil && invocation.slot[2] != Named.type || invocation.owner_type_arguments.length() != 0 { raise :slot }
   if invocation.method_type_arguments.length() != 1 { raise :arity }
   if invocation.signature.parameters[0].type != invocation.method_type_arguments[0] { raise :parameter }
   if invocation.signature.result != invocation.method_type_arguments[0] { raise :result }
   if cached == nil { cached = next.call() }
   cached
  })
 }
}
class Target {
 @Wrap() public fun echo<Element>(value: Element) -> Element { Effects.bodies = Effects.bodies + 1; let typed: Element = value; typed }
}
"#;

#[test]
fn closed_caches_are_separate_when_explicit_and_inferred_instance_calls_repeat() {
    let given = format!(
        r#"{SOURCE}
let target = Target.new()
%[target.echo<Integer>(7), target.echo<String>("first"), target.echo(8), target.echo("second"), Effects.transforms, Effects.bodies]
"#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Text("first".into()),
            Value::Integer(7_u64.into()),
            Value::Text("first".into()),
            Value::Integer(3_u64.into()),
            Value::Integer(2_u64.into()),
        ])))
    );
}

#[test]
fn body_is_not_entered_when_closed_patch_has_wrong_type() {
    let given = format!(
        r#"{}
let rejected = try {{ Target.new().echo<Integer>(7); false }} catch error {{ error is? TypeError }}
%[rejected, Effects.bodies]
"#,
        SOURCE.replace(
            "next.call()",
            "next.call(ArgumentChanges.new(positional: %{:value: \"wrong\"}))"
        )
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
fn result_is_rejected_when_wrapper_or_body_violates_closed_type() {
    for given in [
        SOURCE.replace("cached = next.call()", "cached = \"wrong\""),
        SOURCE.replace("let typed: Element = value; typed", "\"wrong\""),
    ] {
        let given = format!(
            "{given} try {{ Target.new().echo<Integer>(7); false }} catch error {{ error is? TypeError }}"
        );
        let when = run(&compile(&given).expect("compile"));
        assert_eq!(when, Ok(Value::Bool(true)));
    }
}

#[test]
fn input_is_rejected_when_cached_closed_chain_would_short_circuit() {
    let given = format!(
        r#"{SOURCE}
let target = Target.new(); let first = target.echo<Integer>(7)
let rejected = try {{ target.echo<Integer>("wrong"); false }} catch error {{ error is? TypeError }}
%[rejected, Effects.bodies]
"#
    );
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Bool(true),
            Value::Integer(1_u64.into())
        ])))
    );
}

#[test]
fn ordinary_generic_body_is_reused_when_empty_impl_binds_selector() {
    let given = format!(r#"{}
let target = Target.new()
%[target.echo<Integer>(7), (target as Named)..echo(1), target.echo<String>("hello"), Effects.transforms]
"#, SOURCE.replace("class Target {", "contract Named { fun echo<Element>(value: Element) -> Element } impl Target for Named {} class Target {"));
    let when = run(&compile(&given).expect("compile"));
    assert_eq!(
        when,
        Ok(Value::Array(ArrayRef::new(vec![
            Value::Integer(7_u64.into()),
            Value::Integer(7_u64.into()),
            Value::Text("hello".into()),
            Value::Integer(3_u64.into())
        ])))
    );
}

#[test]
fn generic_owner_class_method_remains_rejected_when_decorated() {
    let given = SOURCE
        .replace("class Target {", "class Target<Owner> {")
        .replace("@Wrap() public fun echo", "@Wrap() public class fun echo");
    let when = compile(&given);
    assert!(when.is_err());
}

fn qualified_source() -> String {
    SOURCE.replace("class Target {", "contract Named { fun echo<Element>(value: Element) -> Element } class Target {} impl Target for Named {")
        .replace("invocation.slot[2] != nil", "invocation.slot[2] != nil && invocation.slot[2] != Named.type || invocation.slot[1] != :echo")
        .replace("class Effects {", "class Effects { public class property wrappers: Integer = 0;")
        .replace("if Element.marker()", "Effects.wrappers = Effects.wrappers + 1; if Element.marker()")
}

#[test]
fn qualified_generic_shares_cache_when_closed_type_is_selected() {
    for (selected_type, first, second, expected) in [
        ("Integer", "7", "8", Value::Integer(7_u64.into())),
        (
            "String",
            "\"first\"",
            "\"second\"",
            Value::Text("first".into()),
        ),
    ] {
        let given = format!(r#"{}
let target = Target.new()
%[(target as Named)..echo<{selected_type}>({first}), target.echo({second}),
  (target as Named)..echo({second}), target.echo<{selected_type}>({second}), Effects.transforms, Effects.wrappers, Effects.bodies]
"#, qualified_source().replace("if cached == nil", &format!("if invocation.method_type_arguments[0] != {selected_type}.type {{ raise :closed_type }}; if cached == nil")));
        let when = run(&compile(&given).expect("compile qualified generic"));
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![
                expected.clone(),
                expected.clone(),
                expected.clone(),
                expected,
                Value::Integer(2_u64.into()),
                Value::Integer(4_u64.into()),
                Value::Integer(1_u64.into()),
            ])))
        );
    }
}

#[test]
fn qualified_generic_input_is_rejected_when_binding_violates_closed_type() {
    for call in ["echo<Integer>(\"wrong\")", "echo<String>(7)"] {
        let given = format!(
            r#"{}
let rejected = try {{ (Target.new() as Named)..{call}; false }} catch error {{ error is? TypeError }}
%[rejected, Effects.wrappers, Effects.bodies]
"#,
            qualified_source()
        );
        let when = run(&compile(&given).expect("compile qualified generic"));
        assert_eq!(
            when,
            Ok(Value::Array(ArrayRef::new(vec![
                Value::Bool(true),
                Value::Integer(0_u64.into()),
                Value::Integer(0_u64.into()),
            ])))
        );
    }
}
