use iris_runtime::{
    BuiltinClass, CallableKind, CallableSignature, ClassRegistry, CoreClass, Kernel, KernelError,
    NominalType, Runtime, StaticSpine, Value,
};

#[test]
fn task_identity_is_stable_when_bootstrap_is_repeated() -> Result<(), KernelError> {
    let mut given = ClassRegistry::new();
    let mut kernel = Kernel::new(&mut given)?;
    kernel.register_decorator_classes(&mut given)?;
    let first = given.active(CoreClass::Task.id())?.clone();

    kernel.register_decorator_classes(&mut given)?;

    assert_eq!(kernel.core_class("Task"), Some(CoreClass::Task.id()));
    assert_eq!(CoreClass::from_name("Task"), Some(CoreClass::Task));
    assert_eq!(CoreClass::Task.id().raw(), (1_u64 << 63) | 15);
    assert_eq!(given.active(CoreClass::Task.id())?, &first);
    assert_eq!(first.id().raw(), CoreClass::Task.id().raw());
    assert_eq!(first.commit_id(), 0);
    Ok(())
}

#[test]
fn existing_core_ids_are_preserved_when_task_is_appended() {
    let given = [
        CoreClass::Invocation,
        CoreClass::InvocationSignature,
        CoreClass::InvocationParameter,
        CoreClass::ArgumentChanges,
        CoreClass::DecoratorContext,
        CoreClass::Plan,
        CoreClass::Transformation,
        CoreClass::DecoratorProtocolError,
        CoreClass::Array,
        CoreClass::Hash,
        CoreClass::Symbol,
        CoreClass::Type,
        CoreClass::Tuple,
        CoreClass::TypeError,
        CoreClass::ArgumentError,
        CoreClass::Task,
    ];

    let when: Vec<_> = given.into_iter().map(|core| core.id().raw()).collect();

    assert_eq!(
        when,
        ((1_u64 << 63)..((1_u64 << 63) + 16)).collect::<Vec<_>>()
    );
}

#[test]
fn user_sequences_are_preserved_when_task_registration_precedes_user_declaration()
-> Result<(), KernelError> {
    let mut eager = ClassRegistry::new();
    let mut eager_kernel = Kernel::new(&mut eager)?;
    let mut lazy = ClassRegistry::new();
    let mut lazy_kernel = Kernel::new(&mut lazy)?;

    eager_kernel.register_decorator_classes(&mut eager)?;
    let eager_user = eager.define_class(StaticSpine::new(1), None)?;
    let lazy_user = lazy.define_class(StaticSpine::new(1), None)?;
    lazy_kernel.register_decorator_classes(&mut lazy)?;

    assert_eq!(eager_kernel.core_class("Task"), Some(CoreClass::Task.id()));
    assert_eq!(lazy_kernel.core_class("Task"), Some(CoreClass::Task.id()));
    assert_eq!(eager_user, lazy_user);
    assert_eq!(eager.active(eager_user)?, lazy.active(lazy_user)?);
    assert_eq!(
        eager.publish(eager.open(eager_user)?)?,
        lazy.publish(lazy.open(lazy_user)?)?
    );
    Ok(())
}

#[test]
fn nested_task_signature_matches_when_result_uses_canonical_nominal_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let mut given = ClassRegistry::new();
    let mut kernel = Kernel::new(&mut given)?;
    kernel.register_decorator_classes(&mut given)?;
    let object = NominalType::new(kernel.class(BuiltinClass::Object)?, vec![]);
    let task = kernel.core_class("Task").ok_or(KernelError::Type)?;
    let expected = CallableSignature::new(
        vec![],
        NominalType::new(CoreClass::Task.id(), vec![object.clone()]).into(),
    );
    let first = given.intern_callable_type(CallableKind::Closure, expected.clone())?;

    let when = given.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(vec![], NominalType::new(task, vec![object]).into()),
    )?;

    assert_eq!(when, first);
    let Value::Type(identity, arguments) = when else {
        unreachable!()
    };
    assert!(arguments.is_empty());
    assert_eq!(
        given
            .callable_type(identity)
            .ok_or(KernelError::Type)?
            .signature(),
        &expected
    );
    Ok(())
}

#[test]
fn task_results_remain_invariant_when_result_class_is_a_subclass()
-> Result<(), Box<dyn std::error::Error>> {
    let mut given = ClassRegistry::new();
    let mut kernel = Kernel::new(&mut given)?;
    kernel.register_decorator_classes(&mut given)?;
    let object = kernel.class(BuiltinClass::Object)?;
    let child = given.define_class(StaticSpine::new(1), Some(object))?;
    let synthetic_task = given.define_class(StaticSpine::new(1), Some(object))?;
    let canonical = given.intern_callable_type(
        CallableKind::Closure,
        CallableSignature::new(
            vec![],
            NominalType::new(CoreClass::Task.id(), vec![NominalType::new(object, vec![])]).into(),
        ),
    )?;

    for (task, result) in [(CoreClass::Task.id(), child), (synthetic_task, object)] {
        let when = given.intern_callable_type(
            CallableKind::Closure,
            CallableSignature::new(
                vec![],
                NominalType::new(task, vec![NominalType::new(result, vec![])]).into(),
            ),
        )?;

        assert_ne!(when, canonical);
    }
    Ok(())
}

#[test]
fn task_value_classification_is_unchanged_when_only_identity_is_registered()
-> Result<(), Box<dyn std::error::Error>> {
    let mut given = Runtime::new();
    let mut kernel = Kernel::new(given.registry_mut())?;
    let object = kernel.class(BuiltinClass::Object)?;
    let task_without_metadata = Value::Task(given.allocate(object)?);

    kernel.register_decorator_classes(given.registry_mut())?;

    assert_eq!(kernel.core_class("Task"), Some(CoreClass::Task.id()));
    assert_eq!(kernel.class_of(&task_without_metadata)?, object);
    Ok(())
}

#[test]
fn native_construction_is_rejected_when_task_identity_is_registered() -> Result<(), KernelError> {
    let mut given = ClassRegistry::new();
    let mut kernel = Kernel::new(&mut given)?;
    kernel.register_decorator_classes(&mut given)?;

    let when = kernel.construct(CoreClass::Task.id(), &[]);

    assert_eq!(when, Err(KernelError::Type));
    Ok(())
}
