use super::Value;
use super::gc_tests::{fixture, run};

#[test]
fn structural_cycle_survives_when_reachable_from_either_region() {
    for root in ["holder", "cycle"] {
        let mut given = fixture();
        run(&mut given, "class Holder { public fun keep(value) { @held = value }; public fun read() { @held } }; let holder = Holder.new(); let cycle = %[holder]; holder.keep(cycle); nil").unwrap();
        let held = super::gc_tests::object(given.names["holder"].value());
        let hash = given.runtime.identity_hash(held).unwrap();
        given.names.remove(root);

        given.collect_garbage().unwrap();

        assert_eq!(given.runtime.identity_hash(held), Ok(hash));
    }
}

#[test]
fn structural_cycle_is_reclaimed_when_both_external_roots_are_dropped() {
    let mut given = fixture();
    run(&mut given, "class Holder { public fun keep(value) { @held = value } }; let holder = Holder.new(); let cycle = %[holder]; holder.keep(cycle); nil").unwrap();
    let held = super::gc_tests::object(given.names["holder"].value());
    given.names.remove("holder");
    given.names.remove("cycle");

    given.collect_garbage().unwrap();

    assert_eq!(
        given.runtime.identity_hash(held),
        Err(iris_runtime::RuntimeError::UnknownObjectId(held))
    );
}

#[test]
fn intermediate_values_survive_when_nested_evaluation_collects() {
    for expression in [
        "%[Probe.new(), Worker.collect()][0].value()",
        "(Probe.new(), Worker.collect())[0].value()",
        "%{ :held: Probe.new(), :other: Worker.collect() }[:held].value()",
        "%[Probe.new()][Worker.index()].value()",
        "Worker.choose(Probe.new(), Worker.collect())",
        "(try { Probe.new() } finally { Worker.collect() }).value()",
        "match Probe.new() { held if Worker.test() => held.value() }",
        "Worker.defaults()",
        "Probe.new().self_after_gc().value()",
        "(%[Probe.new()][Worker.index()] = Probe.new()).value()",
    ] {
        let mut given = fixture();
        run(&mut given, "open class Probe { public fun self_after_gc() { NativeFixture.compact_gc(); self } }; module Worker { public fun collect() -> Nil { NativeFixture.compact_gc(); nil }; public fun index() -> Integer { NativeFixture.compact_gc(); 0 }; public fun test() -> Bool { NativeFixture.compact_gc(); true }; public fun choose(first, second) -> Integer { first.value() }; public fun defaults(first = Probe.new(), second = Worker.collect()) -> Integer { first.value() } }; nil").unwrap();

        let when = run(&mut given, expression);

        assert_eq!(when, Ok(Value::Integer(41u64.into())), "{expression}");

        let when = run(
            &mut given,
            &format!(
                "module AsyncProbe {{ public async fun run() -> Integer {{ {expression} }} }}; Host.run(AsyncProbe.run())"
            ),
        );

        assert_eq!(when, Ok(Value::Integer(41u64.into())), "async {expression}");
    }
}

#[test]
fn async_source_ownership_is_stable_when_iterator_is_exhausted() {
    let mut given = fixture();

    let when = run(
        &mut given,
        "module Worker { public async fun run() -> Array { let values = %[1]; let base = values.share_count(); let iterator = values.iterator(); let held = values.share_count(); iterator.next(); iterator.next(); %[held > base, values.share_count() == base] } }; Host.run(Worker.run())",
    );

    let Value::Array(values) = when.unwrap() else {
        panic!("expected Array")
    };
    assert_eq!(
        values.elements(),
        vec![Value::Bool(true), Value::Bool(true)]
    );
}
