use std::collections::HashSet;

use iris_runtime::{ObjectId, Runtime, Selector, StaticSpine, Value};

const SLOT: Selector = Selector::new(1);

#[test]
fn closure_captures_are_implicit_roots_when_class_storage_is_committed_or_active() {
    // Given
    let mut runtime = Runtime::new();
    let class = runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), None)
        .unwrap();
    let staged = runtime
        .registry_mut()
        .stage_class_origin(StaticSpine::new(2), None, &[])
        .unwrap();
    let captures: Vec<_> = (0..4).map(|_| runtime.allocate(class).unwrap()).collect();
    let handles = [
        ObjectId::new(100),
        ObjectId::new(101),
        ObjectId::new(102),
        ObjectId::new(103),
    ];
    runtime
        .assign_class_raw_ivar(class, SLOT, Value::Closure(handles[0]))
        .unwrap();
    runtime
        .declare_class_var(class, SLOT, Value::Closure(handles[1]), true)
        .unwrap();
    runtime
        .assign_staged_class_raw_ivar(staged, SLOT, Value::Closure(handles[2]))
        .unwrap();
    runtime
        .declare_class_var(staged, SLOT, Value::Closure(handles[3]), true)
        .unwrap();
    let mut visited = HashSet::new();

    // When
    let (freed, _) = runtime.collect_with_edges([], |value, edges| {
        if let Value::Closure(id) = value {
            if visited.insert(*id) {
                let index = handles.iter().position(|handle| handle == id).unwrap();
                edges.push(Value::Object(captures[index]));
            }
        }
    });

    // Then
    assert_eq!(freed, 0);
    assert_eq!(visited, HashSet::from(handles));
    assert!(captures.iter().all(|id| runtime.class_of(*id).is_ok()));
    assert!(runtime.staged_class_raw_ivar(staged, SLOT).is_ok());
}

#[test]
fn stale_provisional_handles_are_not_visited_when_registry_bypasses_runtime() {
    for committed in [false, true] {
        // Given
        let mut runtime = Runtime::new();
        let staged = runtime
            .registry_mut()
            .stage_class_origin(StaticSpine::new(1), None, &[])
            .unwrap();
        runtime
            .assign_staged_class_raw_ivar(staged, SLOT, Value::Closure(ObjectId::new(100)))
            .unwrap();
        runtime
            .declare_class_var(staged, SLOT, Value::Closure(ObjectId::new(101)), true)
            .unwrap();
        if committed {
            runtime.registry_mut().commit_declaration_group().unwrap();
        } else {
            runtime.registry_mut().roll_back_group();
        }
        let mut visits = 0;

        // When
        let reached = runtime.trace_with_edges([], |_value, _edges| visits += 1);

        // Then
        assert!(reached.is_empty());
        assert_eq!(visits, 0);
        assert!(
            runtime.commit_declaration_group().is_err(),
            "non-destructive tracing retains stale storage for validation"
        );
    }
}
