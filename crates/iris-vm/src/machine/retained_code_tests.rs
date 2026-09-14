use super::Machine;
use crate::compile;
use iris_runtime::Value;

#[test]
fn retained_method_uses_original_function_zero_when_another_program_runs() {
    let mut machine = Machine::new().expect("machine");
    let old = compile("class Z { public fun value() { 41 } } Z.new().value").expect("old");
    let Value::BoundMethod(old_method) = machine.execute(&old).expect("old method") else {
        panic!("bound method");
    };
    let current =
        compile("class A { public fun apple() { 7 } public fun value() { 99 } } A.new().apple")
            .expect("current");
    let Value::BoundMethod(new_method) = machine.execute(&current).expect("new method") else {
        panic!("bound method");
    };
    drop(old);

    let when = [old_method, new_method]
        .map(|method| machine.invoke_bound_callable(&method, &[], (&current, &[])));

    assert_eq!(
        when,
        [
            Ok(Value::Integer(41_u64.into())),
            Ok(Value::Integer(7_u64.into()))
        ]
    );
}

#[test]
fn retained_closure_keeps_class_and_selector_mapping_when_host_drops_program() {
    let mut machine = Machine::new().expect("machine");
    let old = compile("class Z { public fun value() { 41 } } { Z.new().value() }").expect("old");
    let Value::Closure(callback) = machine.execute(&old).expect("closure") else {
        panic!("closure");
    };
    let current =
        compile("class A { public fun apple() { 7 } public fun value() { 99 } } A.new().apple()")
            .expect("current");
    machine.execute(&current).expect("current executes");
    drop(old);

    let when = machine.invoke_closure_value(callback, &[], &current, &[]);

    assert_eq!(when, Ok(Value::Integer(41_u64.into())));
}

#[test]
fn pending_task_resumes_original_code_when_another_program_runs() {
    let mut machine = Machine::new().expect("machine");
    let old = compile("module M { public async fun value(gate) { await gate; 41 } } let gate = Gate.new(); %[gate, M.value(gate)]").expect("old");
    let Value::Array(values) = machine.execute(&old).expect("pending task") else {
        panic!("array");
    };
    let held = values.elements();
    let Value::Gate(gate) = held[0] else {
        panic!("gate");
    };
    let task = held[1].clone();
    let current = compile("module N { public fun value() { 7 } } N.value()").expect("current");
    machine.execute(&current).expect("current executes");
    drop(old);
    machine.gates.insert(gate, Some(Value::Nil));

    let when = machine.observe_task(task, &current, &[]);

    assert_eq!(when, Ok(Value::Integer(41_u64.into())));
}

#[test]
fn historical_rollback_restores_old_bundle_after_new_body_publication() {
    let mut machine = Machine::new().expect("machine");
    let old = compile("class Z { public fun value() { 41 } } Z.new().value").expect("old");
    let Value::BoundMethod(bound) = machine.execute(&old).expect("method") else {
        panic!("method");
    };
    let iris_runtime::BoundReceiver::Object(object) = bound.receiver() else {
        panic!("object");
    };
    let class = machine.runtime.class_of(object).expect("class");
    let revision = machine
        .runtime
        .registry()
        .active(class)
        .expect("revision")
        .id();
    let current = compile("class A { public fun apple() { 7 } } A.new().apple").expect("current");
    let Value::BoundMethod(replacement) = machine.execute(&current).expect("replacement") else {
        panic!("method");
    };
    machine
        .runtime
        .registry_mut()
        .publish_method(
            class,
            bound.method().selector(),
            replacement.method().body(),
            iris_runtime::Visibility::Public,
        )
        .expect("publish");
    let replaced = machine
        .runtime
        .dispatch_instance(object, bound.method().selector())
        .expect("replaced");
    assert_eq!(
        machine.invoke_selected_method(replaced, vec![Value::Object(object)], &current, &[]),
        Ok(Value::Integer(7_u64.into()))
    );
    drop(old);
    machine
        .runtime
        .registry_mut()
        .rollback(class, revision)
        .expect("rollback");

    let selected = machine
        .runtime
        .dispatch_instance(object, bound.method().selector())
        .expect("restored");
    let when = machine.invoke_selected_method(selected, vec![Value::Object(object)], &current, &[]);

    assert_eq!(when, Ok(Value::Integer(41_u64.into())));
}

#[test]
fn retained_wrapper_and_suspended_inner_body_keep_original_bundle() {
    let mut machine = Machine::new().expect("machine");
    let old = compile(r#"
class Wrap {}
impl Wrap for MethodDecorator {
 public fun plan(declaration, arguments) -> Plan { Plan.empty }
 public fun transform(declaration, arguments, context) -> Transformation {
  Transformation.wrap_method({ async |invocation: Invocation, next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object;
   (await next.call() as Integer) + 1
  })
 }
}
class Z { @Wrap() public async fun value(gate) -> Integer { await gate; 40 } }
let gate = Gate.new()
%[gate, Z.new().value(gate)]
"#).expect("old");
    let Value::Array(values) = machine.execute(&old).expect("pending wrapped task") else {
        panic!("array");
    };
    let values = values.elements();
    let Value::Gate(gate) = values[0] else {
        panic!("gate");
    };
    let current = compile("class A { public fun apple() { 7 } } A.new().apple()").expect("current");
    machine.execute(&current).expect("current executes");
    drop(old);
    machine.gates.insert(gate, Some(Value::Nil));

    let when = machine.observe_task(values[1].clone(), &current, &[]);

    assert_eq!(when, Ok(Value::Integer(41_u64.into())));
}

#[test]
fn retained_module_closure_keeps_module_identity_when_names_overlap() {
    let mut machine = Machine::new().expect("machine");
    let old = compile("module M { public fun value() { 41 } } { M.value() }").expect("old");
    let Value::Closure(callback) = machine.execute(&old).expect("closure") else {
        panic!("closure");
    };
    let current =
        compile("module M { public fun apple() { 7 } public fun value() { 99 } } M.apple()")
            .expect("current");
    machine.execute(&current).expect("current executes");
    drop(old);

    let when = machine.invoke_closure_value(callback, &[], &current, &[]);

    assert_eq!(when, Ok(Value::Integer(41_u64.into())));
}

#[test]
fn new_bundle_selector_dispatch_reaches_method_owned_by_old_bundle() {
    let mut machine = Machine::new().expect("machine");
    let old = compile("class Z { public fun value() { 41 } } Z.new()").expect("old");
    let object = machine.execute(&old).expect("object");
    let current = compile("class A { public fun apple() { 7 } } { |target| target.value() }")
        .expect("current");
    let Value::Closure(callback) = machine.execute(&current).expect("callback") else {
        panic!("closure");
    };
    drop(old);

    let when = machine.invoke_closure_value(callback, &[object], &current, &[]);

    assert_eq!(when, Ok(Value::Integer(41_u64.into())));
}

#[test]
fn direct_registration_allocates_distinct_handles_for_each_program() {
    let mut machine = Machine::new().expect("machine");
    let old = compile("class Z { public fun value() { 41 } }").expect("old");
    let old_classes = machine.register_classes(&old).expect("old classes");
    let old_body = machine.code.body(0, &old).expect("old body");
    let current = compile("class A { public fun value() { 7 } }").expect("current");
    machine.register_classes(&current).expect("current classes");
    assert_ne!(old_body, machine.code.body(0, &current).expect("new body"));
    drop(old);

    let when = machine
        .resolve_method_body(old_body, &current)
        .expect("retained owner");

    assert_eq!(when.classes, old_classes);
    assert_eq!(
        machine.invoke_function(
            when.function,
            vec![Value::Nil],
            &when.program,
            &when.classes
        ),
        Ok(Value::Integer(41_u64.into()))
    );
}

#[test]
fn repeated_host_program_keeps_each_execution_class_mapping() {
    let mut machine = Machine::new().expect("machine");
    let program = compile("class Z {} { Z }").expect("program");
    let Value::Closure(first) = machine.execute(&program).expect("first") else {
        panic!("closure");
    };
    let original = machine
        .invoke_closure_value(first, &[], &program, &[])
        .expect("old class");
    machine.execute(&program).expect("second");

    let when = machine.invoke_closure_value(first, &[], &program, &[]);

    assert_eq!(when, Ok(original));
}

#[test]
fn retained_method_source_reports_owner_package_not_observer_package() {
    let mut machine = Machine::new().expect("machine");
    let mut old = compile("class Z { public fun value() { 41 } } Z.new().value").expect("old");
    old.package = Some(crate::native::PackageIdentity {
        package_id: "original".into(),
        api_major: 1,
        version: None,
    });
    let Value::BoundMethod(bound) = machine.execute(&old).expect("method") else {
        panic!("method");
    };
    let mut current = compile("nil").expect("current");
    current.package = Some(crate::native::PackageIdentity {
        package_id: "observer".into(),
        api_major: 1,
        version: None,
    });
    machine.execute(&current).expect("observer");
    drop(old);

    let when = machine
        .authored_send(&Value::Method(bound.method()), "source", &[], &current, &[])
        .expect("source");

    let Some(Value::Array(source)) = when else {
        panic!("source fields");
    };
    assert_eq!(source.get(0), Some(Value::Symbol("original".into())));
}

#[test]
fn suspended_closure_keeps_return_guard_when_new_bundle_has_matching_index() {
    let mut machine = Machine::new().expect("machine");
    let old = compile("let gate = Gate.new(); let callback = { async || -> Integer; await gate; :wrong }; %[gate, callback.call()]").expect("old");
    let Value::Array(values) = machine.execute(&old).expect("pending") else {
        panic!("array");
    };
    let values = values.elements();
    let Value::Gate(gate) = values[0] else {
        panic!("gate");
    };
    let current = compile("{ || -> Symbol; :allowed }").expect("current");
    machine.execute(&current).expect("current executes");
    drop(old);
    machine.gates.insert(gate, Some(Value::Nil));

    let when = machine.observe_task(values[1].clone(), &current, &[]);

    assert!(
        when.is_err(),
        "the original Integer return guard must reject Symbol"
    );
}
