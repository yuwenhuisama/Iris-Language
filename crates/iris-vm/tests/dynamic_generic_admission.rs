use iris_runtime::Value;
use iris_vm::{compile, run};

#[test]
fn aliases_reject_when_generic_arity_is_incomplete() {
    for binding in ["Box", "Box<Integer>", "Box<Integer, String, Nil>"] {
        let given = format!(
            "class Effects {{ public class property count: Integer = 0 }}; class Box<T, U> {{ public property mark: Integer = Effects.count = Effects.count + 1; public fun initialize() {{ Effects.count = Effects.count + 1 }} }}; let rejected = try {{ let target = {binding}; target.new(); false }} catch error {{ true }}; rejected && Effects.count == 0"
        );
        let when = run(&compile(&given).expect("alias compiles"));
        assert_eq!(when, Ok(Value::Bool(true)), "{binding}");
    }
}

#[test]
fn owner_bindings_are_resolved_when_constructing_directly_or_through_alias() {
    for construction in ["Box<T>.new()", "let target = Box<T>; target.new()"] {
        let given = format!(
            "contract Comparable<T> {{}}; class Key<T> {{}}; impl Key<T> for Comparable<Key<T>> {{}}; class Box<T> where T: Comparable<T> {{}}; class Owner<T> {{ public fun build() {{ {construction} }} }}; Owner<Key<Integer>>.new().build() is? Box<Key<Integer>>"
        );
        let when = run(&compile(&given).expect("owner source compiles"));
        assert_eq!(when, Ok(Value::Bool(true)), "{construction}");
    }
}

#[test]
fn owner_bound_rejection_precedes_effects_when_contract_arguments_differ() {
    for conformance in ["Comparable<Object>", "Comparable"] {
        for construction in ["Box<T>.new()", "let target = Box<T>; target.new()"] {
            let given = format!(
                "class Effects {{ public class property count: Integer = 0 }}; contract Comparable<T> {{}}; class Key {{}}; impl Key for {conformance} {{}}; class Box<T> where T: Comparable<T> {{ public property mark: Integer = Effects.count = Effects.count + 1; public fun initialize() {{ Effects.count = Effects.count + 1 }} }}; class Owner<T> {{ public fun build() {{ {construction} }} }}; let owner = Owner<Key>.new(); let rejected = try {{ owner.build(); false }} catch error {{ true }}; rejected && Effects.count == 0"
            );
            let when = run(&compile(&given).expect("owner source compiles"));
            assert_eq!(when, Ok(Value::Bool(true)), "{conformance}: {construction}");
        }
    }
}

#[test]
fn nested_constructor_resolves_its_owner_when_parameter_names_overlap() {
    let given = "class Inner<T> {}; class Outer<T> { public fun initialize() { @value = Inner<T>.new() }; public fun value() { @value } }; Outer<Integer>.new().value() is? Inner<Integer>";
    let when = run(&compile(given).expect("nested source compiles"));
    assert_eq!(when, Ok(Value::Bool(true)));
}
