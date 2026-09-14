use std::collections::{HashMap, HashSet};

use iris_runtime::{
    ArrayRef, ClassId, Method, MethodBody, MethodId, MethodOwner, ObjectId, Runtime, Selector,
    StaticSpine, Value, Visibility,
};

const SLOT: Selector = Selector::new(1);

fn class(runtime: &mut Runtime) -> ClassId {
    runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)
        .unwrap()
}

#[test]
fn capture_survives_when_heap_array_and_engine_edges_form_a_cycle() {
    // Given
    let mut runtime = Runtime::new();
    let class = class(&mut runtime);
    let holder = runtime.allocate(class).unwrap();
    let captured = runtime.allocate(class).unwrap();
    let dead = runtime.allocate(class).unwrap();
    let closure = ObjectId::new(100);
    let array = ArrayRef::new(vec![Value::Closure(closure)]);
    array.mutate(|values| values.push(Value::Array(array.clone())));
    runtime
        .assign_raw_ivar(holder, SLOT, Value::Array(array.clone()))
        .unwrap();
    runtime
        .assign_raw_ivar(captured, SLOT, Value::Object(holder))
        .unwrap();
    let captures = HashMap::from([(
        closure,
        vec![Value::Object(captured), Value::Closure(closure)],
    )]);
    let before = array.share_count();
    let mut visited = HashSet::new();

    // When
    let (freed, _) = runtime.collect_with_edges([&Value::Object(holder)], |value, edges| {
        if let Value::Closure(id) = value {
            if visited.insert(*id) {
                edges.extend(captures[id].iter().cloned());
            }
        }
    });

    // Then
    assert_eq!(freed, 1);
    assert!(runtime.class_of(holder).is_ok());
    assert!(runtime.class_of(captured).is_ok());
    assert!(runtime.class_of(dead).is_err());
    assert_eq!(visited, HashSet::from([closure]));
    assert_eq!(array.share_count(), before);
    array.mutate(Vec::clear);
}

#[test]
fn method_capture_survives_when_reached_directly_or_through_bound_method() {
    for bound in [false, true] {
        // Given
        let mut runtime = Runtime::new();
        let class = class(&mut runtime);
        let receiver = runtime.allocate(class).unwrap();
        let captured = runtime.allocate(class).unwrap();
        let method = Method::new(
            MethodId::new(7),
            MethodOwner::Class(class),
            SLOT,
            MethodBody::new(9),
            Visibility::Public,
        );
        let root = if bound {
            Value::BoundMethod(
                runtime
                    .registry_mut()
                    .bind_retained_instance(receiver, method)
                    .unwrap(),
            )
        } else {
            Value::Method(method)
        };
        let mut visited = HashSet::new();

        // When
        runtime.collect_with_edges([&root], |value, edges| {
            if let Value::Method(found) = value {
                assert_eq!(*found, method);
                if visited.insert(found.id()) {
                    edges.extend([Value::Object(captured), Value::Method(method)]);
                }
            }
        });

        // Then
        assert!(runtime.class_of(captured).is_ok());
        assert_eq!(runtime.class_of(receiver).is_ok(), bound);
        assert_eq!(visited, HashSet::from([method.id()]));
    }
}

#[test]
fn handle_kinds_remain_distinct_when_ids_overlap_and_unreachable_handles_exist() {
    // Given
    let mut runtime = Runtime::new();
    let class = class(&mut runtime);
    let object = runtime.allocate(class).unwrap();
    let dead = runtime.allocate(class).unwrap();
    runtime
        .assign_raw_ivar(dead, SLOT, Value::Closure(ObjectId::new(999)))
        .unwrap();
    let roots = [
        Value::Object(object),
        Value::Closure(object),
        Value::Task(object),
    ];
    let mut kinds = Vec::new();

    // When
    let reached = runtime.trace_with_edges(&roots, |value, _edges| match value {
        Value::Object(id) => kinds.push(("object", *id)),
        Value::Closure(id) => kinds.push(("closure", *id)),
        Value::Task(id) => kinds.push(("task", *id)),
        _ => {}
    });

    // Then
    assert_eq!(
        kinds,
        vec![("object", object), ("closure", object), ("task", object)]
    );
    assert_eq!(reached.ids(), vec![object]);
    assert!(runtime.class_of(dead).is_ok(), "traversal must not sweep");
}

#[test]
fn share_counts_are_restored_when_edges_temporarily_clone_array_captures() {
    // Given
    let runtime = Runtime::new();
    let capture = Value::Array(ArrayRef::new(vec![]));
    let Value::Array(array) = &capture else {
        unreachable!()
    };
    let roots = [Value::Array(ArrayRef::new(vec![Value::Closure(
        ObjectId::new(1),
    )]))];
    let Value::Array(root) = &roots[0] else {
        unreachable!()
    };
    let before = (array.share_count(), root.share_count());

    // When
    runtime.trace_with_edges(&roots, |value, edges| {
        if let Value::Closure(_) = value {
            edges.push(capture.clone());
        }
    });

    // Then
    assert_eq!((array.share_count(), root.share_count()), before);
}

#[test]
fn legacy_collection_keeps_its_root_only_behavior_without_engine_edges() {
    // Given
    let mut runtime = Runtime::new();
    let class = class(&mut runtime);
    let live = runtime.allocate(class).unwrap();
    let dead = runtime.allocate(class).unwrap();
    let root = Value::Array(ArrayRef::new(vec![Value::Object(live)]));

    // When
    let (freed, _) = runtime.collect_garbage([&root]);

    // Then
    assert_eq!(freed, 1);
    assert!(runtime.class_of(live).is_ok());
    assert!(runtime.class_of(dead).is_err());
}
