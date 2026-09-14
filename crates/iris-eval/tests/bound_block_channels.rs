use iris_eval::{Session, evaluate};
use iris_runtime::Value;

const FIXTURE: &str = r#"
    class Saved {
        public class property original: Object = nil
        public class property replacement: Object = nil
        public class property entered: Integer = 0
    }
    class Relay {}
    impl Relay for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                if !(invocation.original_block same? Saved.original) { raise :original }
                next.call()
            })
        }
    }
    class Replace {}
    impl Replace for MethodDecorator {
        public fun plan(d, a) -> Plan { Plan.empty }
        public fun transform(d, a, c) -> Transformation {
            Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
                if !(invocation.original_block same? Saved.original) { raise :original }
                next.call(ArgumentChanges.new(block: Saved.replacement))
            })
        }
    }
    class Receiver {
        public property count: Integer = 10
        public fun value() -> Integer { self.count = self.count + 1; self.count }
    }
    class Target {
        @Relay() public fun relay(&block: Block<() -> Integer>) -> Integer {
            Saved.entered = Saved.entered + 1
            if !(block same? Saved.original) { raise :identity }
            block.call()
        }
        @Replace() public fun replace(&block: Block<() -> Integer>) -> Integer {
            Saved.entered = Saved.entered + 1
            if !(block same? Saved.replacement) { raise :identity }
            block.call()
        }
    }
"#;

#[test]
fn original_bound_block_retains_identity_and_receiver_when_relayed() {
    let given = format!(
        "{FIXTURE} let run = {{ || let receiver = Receiver.new(); let bound = receiver.value; Saved.original = bound;
         Target.new().relay(&bound); Saved.original.call() }}; run.call()"
    );
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(12u64.into())));
}

#[test]
fn replacement_bound_block_retains_identity_and_receiver_when_installed() {
    let given = format!(
        "{FIXTURE} let run = {{ || let receiver = Receiver.new(); let bound = receiver.value;
         let original = {{ || -> Integer; 99 }}; Saved.original = original; Saved.replacement = bound;
         Target.new().replace(&original); Saved.replacement.call() }}; run.call()"
    );
    let when = evaluate(&given);
    assert_eq!(when, Ok(Value::Integer(12u64.into())));
}

#[test]
fn bound_block_is_rejected_before_inner_when_selected_signature_differs() {
    for declaration in [
        "public fun wrong(value: Integer) -> Integer { value }",
        "public fun wrong() -> Object { 7 }",
        "public async fun wrong() -> Integer { 7 }",
    ] {
        for channel in ["relay", "replace"] {
            let mut given = Session::new().unwrap();
            given
                .evaluate(&format!(
                    "{FIXTURE} class Wrong {{ {declaration} }}; let bound = Wrong.new().wrong;
                 Saved.replacement = bound; Saved.original = bound; 0"
                ))
                .unwrap();
            if channel == "replace" {
                given
                    .evaluate("Saved.original = { || -> Integer; 99 }; 0")
                    .unwrap();
            }
            let when = given.evaluate(&format!(
                "try {{ Target.new().{channel}(&Saved.original); false }} catch error {{ error is? TypeError }}"
            ));
            assert_eq!(when, Ok(Value::Bool(true)), "{channel}: {declaration}");
            assert_eq!(
                given.evaluate("Saved.entered"),
                Ok(Value::Integer(0u64.into()))
            );
        }
    }
}

#[test]
fn bound_block_uses_captured_method_signature_when_slot_is_replaced() {
    let mut given = Session::new().unwrap();
    given.evaluate(&format!(
        "{FIXTURE} let receiver = Receiver.new(); let bound = receiver.value; Saved.original = bound;
         open class Receiver {{ override public fun value() -> Integer {{ 99 }} }};
         0"
    )).unwrap();
    let when = given.evaluate("Target.new().relay(&bound)");
    assert_eq!(when, Ok(Value::Integer(11u64.into())));
}

#[test]
fn async_bound_block_returns_task_when_signature_names_task_result() {
    let given = r#"
        class Receiver { public async fun value() -> Integer { 42 } }
        class Target {
            public fun relay(&block: Block<() -> Task<Integer>>) -> Task<Integer> { block.call() }
        }
        let bound = Receiver.new().value
        Host.run(Target.new().relay(&bound))
    "#;
    let when = evaluate(given);
    assert_eq!(when, Ok(Value::Integer(42u64.into())));
}
