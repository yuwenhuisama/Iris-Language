use std::process::Command;

const GIVEN: &str = r#"
class State {
    public class property saved: Object = nil
    public class property transforms: Integer = 0
}
class Preserve {}
impl Preserve for MethodDecorator {
    public fun plan(declaration, arguments) -> Plan { Plan.empty }
    public fun transform(declaration, arguments, context) -> Transformation {
        Transformation.wrap_method({ |invocation: Invocation, next: Closure<(ArgumentChanges) -> Object>| -> Object;
            next.call()
        })
    }
}
class Escape {}
impl Escape for MethodDecorator {
    public fun plan(declaration, arguments) -> Plan { Plan.empty }
    public fun transform(declaration, arguments, context) -> Transformation {
        let captured = 41
        State.saved = { || -> Integer; captured + 1 }
        State.transforms = State.transforms + 1
        FAILURE
    }
}
class Target {
    @Preserve() public fun value() -> Integer { 7 }
    @Preserve() public fun sibling() -> Integer { 9 }
}
let target = Target.new()
let retained = target.value
let method = Target.method(:value)
let sibling = Target.method(:sibling)
let revision = Target.active_revision
let commit = Reflection::Class.revision(Target).fetch(:commit_id)
"#;

const WHEN: &str = r#"
let failed = try {
    Target.open() { |candidate|;
        public override fun value() -> Integer { 700 }
        candidate.define_method(:staged, { || -> Integer; 99 })
        @Escape() public fun extra() -> Integer { 900 }
    }
    false
} catch error { true }
"#;

const THEN: &str = r#"
print(failed)
print(State.transforms)
print(Target.active_revision == revision)
print(Reflection::Class.revision(Target).fetch(:commit_id) == commit)
print(Target.method(:value) same? method)
print(Target.method(:sibling) same? sibling)
print(Target.method(:staged) == nil)
print(Target.method(:extra) == nil)
print(target.value())
print(retained.call())
print(target.sibling())
let saved = State.saved
print(saved.call())
let captured = 98
let later = { || -> Integer; captured + 1 }
print(saved same? later)
print(later.call())
print(saved.call())
print(State.saved same? saved)
"#;

fn check(flags: &[&str], failure: &str) -> Result<(), Box<dyn std::error::Error>> {
    let given = format!("{}{WHEN}{THEN}", GIVEN.replace("FAILURE", failure));
    let parsed = iris_parser::parse(&given);
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    let when = Command::new(env!("CARGO_BIN_EXE_iris"))
        .args(flags)
        .args(["-e", &given])
        .output()?;

    let stdout = String::from_utf8(when.stdout)?;
    let stderr = String::from_utf8(when.stderr)?;
    assert_eq!(when.status.code(), Some(0), "{stdout}\n{stderr}");
    assert_eq!(
        stdout, "true\n1\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n7\n7\n9\n42\nfalse\n99\n42\ntrue\n",
        "stderr: {stderr}"
    );
    assert!(stderr.is_empty(), "{stderr}");
    Ok(())
}

#[test]
fn meta_c138_c042_reference_preserves_escape_when_transform_returns_invalid_result()
-> Result<(), Box<dyn std::error::Error>> {
    check(&[], "nil")
}

#[test]
fn meta_c138_c042_vm_preserves_escape_when_transform_returns_invalid_result()
-> Result<(), Box<dyn std::error::Error>> {
    check(&["--vm"], "nil")
}

#[test]
fn meta_c138_c042_reference_preserves_escape_when_transform_raises()
-> Result<(), Box<dyn std::error::Error>> {
    check(&[], "raise 123")
}

#[test]
fn meta_c138_c042_vm_preserves_escape_when_transform_raises()
-> Result<(), Box<dyn std::error::Error>> {
    check(&["--vm"], "raise 123")
}
