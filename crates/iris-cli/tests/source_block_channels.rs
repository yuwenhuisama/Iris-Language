use std::process::Command;

fn check(flags: &[&str], given: &str, expected: &str) {
    let when = Command::new(env!("CARGO_BIN_EXE_iris"))
        .args(flags)
        .args(["-e", given])
        .output()
        .expect("launch Iris CLI");

    let stderr = String::from_utf8_lossy(&when.stderr);
    assert_eq!(when.status.code(), Some(0), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&when.stdout), expected, "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
}

macro_rules! source_cases {
    ($($name:ident: $source:expr => $expected:expr;)*) => {
        $(mod $name {
            #[test]
            fn reference() { super::check(&[], $source, $expected); }
            #[test]
            fn vm() { super::check(&["--vm"], $source, $expected); }
        })*
    };
}

source_cases! {
    nil_presence_and_closure_identity_when_explicit: r#"
class Inspect {}
impl Inspect for MethodDecorator {
    public fun plan(declaration, arguments) -> Plan { Plan.empty }
    public fun transform(declaration, arguments, context) -> Transformation {
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            print(invocation.block_omitted)
            print(invocation.original_positional.length)
            next.call()
        })
    }
}
class Target {
    @Inspect()
    public fun value(callback: Closure<() -> Integer>, &block: Block<() -> Integer> = nil) -> Integer {
        if block == nil { callback.call() } else { print(callback same? block); block.call() }
    }
}
let callback = { || -> Integer; 7 }
print(Target.new().value(callback))
print(Target.new().value(callback, &nil))
print(Target.new().value(&callback, callback))
"# => "true\n1\n7\nfalse\n1\n7\nfalse\n1\ntrue\n7\n";

    duplicates_when_operands_precede_validation: r#"
class Inspect {}
impl Inspect for MethodDecorator {
    public fun plan(declaration, arguments) -> Plan { Plan.empty }
    public fun transform(declaration, arguments, context) -> Transformation {
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            print(:wrapper)
            next.call()
        })
    }
}
module Helpers {
    public fun receiver() { print(:receiver); Target.new() }
    public fun operand(label) { print(label); nil }
    public fun default_value() { print(:default); 1 }
}
class Target {
    @Inspect()
    public fun value(value = Helpers.default_value(), &block = nil) { print(:body) }
}
try { Helpers.receiver().value(&Helpers.operand(:first), &Helpers.operand(:second)) }
catch error { print(error is? ArgumentError) }
try { Helpers.receiver().value(&Helpers.operand(:mixed)) { ||; print(:closure_body) } }
catch error { print(error is? ArgumentError) }
"# => "receiver\nfirst\nsecond\ntrue\nreceiver\nmixed\ntrue\n";

    next_rejects_when_explicit_nil_block: r#"
class Inspect {}
impl Inspect for MethodDecorator {
    public fun plan(declaration, arguments) -> Plan { Plan.empty }
    public fun transform(declaration, arguments, context) -> Transformation {
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            try { next.call(&nil) } catch error { print(error is? ArgumentError) }
            next.call()
        })
    }
}
class Target {
    @Inspect()
    public fun value() -> Integer { print(:body); 7 }
}
print(Target.new().value())
"# => "true\nbody\n7\n";

    async_bridge_is_fresh_when_duplicate_blocks_fail: r#"
class Inspect {}
impl Inspect for MethodDecorator {
    public fun plan(declaration, arguments) -> Plan { Plan.empty }
    public fun transform(declaration, arguments, context) -> Transformation {
        Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
            print(:wrapper)
            await next.call()
        })
    }
}
module Helpers { public fun default_value() { print(:default); 1 } }
class Target {
    @Inspect()
    public async fun value(value = Helpers.default_value(), &block = nil) -> Integer { print(:body); 7 }
}
let target = Target.new()
let first = target.value(&nil, &nil)
let second = target.value(&nil) { || -> Integer; 9 }
print(first same? second)
print(:returned)
try { Host.run(first) } catch error { print(error is? ArgumentError) }
try { Host.run(second) } catch error { print(error is? ArgumentError) }
"# => "false\nreturned\ntrue\ntrue\n";

    operands_run_once_when_block_await_suspends: r#"
class Target {
    public fun value(left, right, &block) { print(left); print(right); block }
}
module Helpers {
    public fun receiver() { print(:receiver); Target.new() }
    public fun operand(label) { print(label); label }
    public async fun run(gate) {
        Helpers.receiver().value(Helpers.operand(:left), &await gate, Helpers.operand(:right))
    }
}
let gate = Gate.new()
let callback = { || -> Integer; 7 }
let task = Helpers.run(gate)
print(:parked)
Gate.complete(gate, callback)
print(Host.run(task) same? callback)
"# => "receiver\nleft\nparked\nright\nleft\nright\ntrue\n";
}
