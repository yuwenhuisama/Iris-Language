use iris_runtime::{
    ClassId, ConstructionError, ExecutionError, MethodBody, Runtime, Selector, StaticSpine, Value,
    Visibility,
};

const INITIALIZE: Selector = Selector::new(1);
const LATER_SEND: Selector = Selector::new(2);
const PROPERTY: Selector = Selector::new(3);
const PROPERTY_SETTER: Selector = Selector::new(4);
const IVAR: Selector = Selector::new(5);
const CLASS_VAR: Selector = Selector::new(6);

fn define_class(
    runtime: &mut Runtime,
    superclass: Option<ClassId>,
) -> Result<ClassId, ConstructionError> {
    runtime
        .registry_mut()
        .define_class(StaticSpine::new(1), superclass)
        .map_err(ConstructionError::Class)
}

#[test]
fn construction_snapshots_revision_then_later_dispatch_uses_current_active_revision()
-> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = define_class(&mut runtime, None)?;
    runtime
        .registry_mut()
        .publish_stored_property(class, Selector::new(9), MethodBody::new(5))?;
    let captured_initialize = runtime.registry_mut().publish_method(
        class,
        INITIALIZE,
        MethodBody::new(10),
        Visibility::Public,
    )?;
    let old_send = runtime.registry_mut().publish_method(
        class,
        LATER_SEND,
        MethodBody::new(20),
        Visibility::Public,
    )?;
    let mut observed_initialize = None;

    // When
    let instance = runtime.construct(class, &[], |runtime, method, _receiver, _arguments| {
        if method.body() == MethodBody::new(5) {
            runtime.registry_mut().publish_method(
                class,
                INITIALIZE,
                MethodBody::new(15),
                Visibility::Public,
            )?;
            runtime.registry_mut().publish_method(
                class,
                LATER_SEND,
                MethodBody::new(30),
                Visibility::Public,
            )?;
        }
        if method == captured_initialize {
            observed_initialize = Some(method);
        }
        Ok(Value::Nil)
    })?;
    let later = runtime.dispatch_instance(instance, LATER_SEND)?;

    // Then
    assert_eq!(observed_initialize, Some(captured_initialize));
    assert_eq!(old_send.body(), MethodBody::new(20));
    assert_eq!(later.body(), MethodBody::new(30));
    Ok(())
}

#[test]
fn stored_initializers_run_superclass_to_subclass_before_initialize()
-> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let parent = define_class(&mut runtime, None)?;
    let child = define_class(&mut runtime, Some(parent))?;
    runtime.registry_mut().publish_stored_property(
        parent,
        Selector::new(11),
        MethodBody::new(1),
    )?;
    runtime.registry_mut().publish_stored_property(
        parent,
        Selector::new(10),
        MethodBody::new(2),
    )?;
    runtime
        .registry_mut()
        .publish_stored_property(child, Selector::new(9), MethodBody::new(3))?;
    runtime.registry_mut().publish_method(
        child,
        INITIALIZE,
        MethodBody::new(4),
        Visibility::Public,
    )?;
    let mut log = Vec::new();

    // When
    runtime.construct(child, &[], |_runtime, method, _receiver, _arguments| {
        log.push(method.body());
        Ok(Value::Nil)
    })?;

    // Then
    assert_eq!(
        log,
        vec![
            MethodBody::new(1),
            MethodBody::new(2),
            MethodBody::new(3),
            MethodBody::new(4),
        ]
    );
    Ok(())
}

#[test]
fn raising_initializer_returns_no_instance_but_escaped_object_remains_ordinary()
-> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = define_class(&mut runtime, None)?;
    runtime.registry_mut().publish_method(
        class,
        INITIALIZE,
        MethodBody::new(1),
        Visibility::Public,
    )?;
    let mut escaped = None;

    // When
    let result = runtime.construct(class, &[], |_runtime, _method, receiver, _arguments| {
        escaped = Some(receiver);
        Err(ExecutionError::Raised(Value::Integer(7_u8.into())))
    });

    // Then
    assert_eq!(
        result,
        Err(ConstructionError::Runtime(ExecutionError::Raised(
            Value::Integer(7_u8.into())
        )))
    );
    let escaped = escaped.ok_or(ConstructionError::EscapedObjectMissing)?;
    assert_eq!(runtime.class_of(escaped)?, class);
    assert_eq!(runtime.raw_ivar(escaped, IVAR)?, Value::Nil);
    Ok(())
}

#[test]
fn raw_ivar_identity_is_receiver_and_name_across_parent_subclass_and_module_code()
-> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let parent = define_class(&mut runtime, None)?;
    let child = define_class(&mut runtime, Some(parent))?;
    let instance = runtime.allocate(child)?;

    // When
    runtime.assign_raw_ivar(instance, IVAR, Value::Integer(1_u8.into()))?;
    let parent_read = runtime.raw_ivar(instance, IVAR)?;
    runtime.assign_raw_ivar(instance, IVAR, Value::Integer(2_u8.into()))?;
    let module_read = runtime.raw_ivar(instance, IVAR)?;

    // Then
    assert_eq!(parent_read, Value::Integer(1_u8.into()));
    assert_eq!(module_read, Value::Integer(2_u8.into()));
    assert_eq!(runtime.class_of(instance)?, child);
    Ok(())
}

#[test]
fn undeclared_raw_ivar_read_returns_nil_without_creating_storage() -> Result<(), ConstructionError>
{
    // Given
    let mut runtime = Runtime::new();
    let class = define_class(&mut runtime, None)?;
    let instance = runtime.allocate(class)?;

    // When
    let value = runtime.raw_ivar(instance, IVAR)?;

    // Then
    assert_eq!(value, Value::Nil);
    assert_eq!(runtime.raw_ivar_count(instance)?, 0);
    Ok(())
}

#[test]
fn property_assignment_returns_setter_result_while_raw_ivar_assignment_returns_stored_value()
-> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = define_class(&mut runtime, None)?;
    runtime.registry_mut().publish_method(
        class,
        PROPERTY_SETTER,
        MethodBody::new(1),
        Visibility::Public,
    )?;
    let instance = runtime.allocate(class)?;
    let stored = Value::Integer(4_u8.into());
    let setter_result = Value::Integer(9_u8.into());

    // When
    let property_result = runtime.assign_property(
        instance,
        PROPERTY_SETTER,
        stored.clone(),
        |_runtime, _method, _receiver, _arguments| Ok(setter_result.clone()),
    )?;
    let ivar_result = runtime.assign_raw_ivar(instance, IVAR, stored.clone())?;

    // Then
    assert_eq!(property_result, setter_result);
    assert_eq!(ivar_result, stored);
    Ok(())
}

#[test]
fn properties_are_explicit_selector_dispatch_without_implicit_backing_storage()
-> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = define_class(&mut runtime, None)?;
    runtime.registry_mut().publish_method(
        class,
        PROPERTY,
        MethodBody::new(1),
        Visibility::Public,
    )?;
    runtime.registry_mut().publish_method(
        class,
        PROPERTY_SETTER,
        MethodBody::new(2),
        Visibility::Public,
    )?;
    let instance = runtime.allocate(class)?;

    // When
    let getter = runtime.dispatch_instance(instance, PROPERTY)?;
    let setter = runtime.dispatch_instance(instance, PROPERTY_SETTER)?;

    // Then
    assert_eq!(getter.body(), MethodBody::new(1));
    assert_eq!(setter.body(), MethodBody::new(2));
    assert_eq!(runtime.raw_ivar_count(instance)?, 0);
    Ok(())
}

#[test]
fn declared_class_variable_assignment_returns_the_stored_value() -> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = define_class(&mut runtime, None)?;
    let stored = Value::Integer(3_u8.into());

    // When
    runtime.declare_class_var(class, CLASS_VAR, Value::Nil, true)?;
    let result = runtime.assign_class_var(class, CLASS_VAR, stored.clone())?;

    // Then
    assert_eq!(result, stored);
    assert_eq!(runtime.class_var(class, CLASS_VAR)?, Some(stored));
    Ok(())
}

#[test]
fn class_variable_assignment_to_absent_storage_fails_without_creating_a_cell()
-> Result<(), ConstructionError> {
    // Given
    let mut runtime = Runtime::new();
    let class = define_class(&mut runtime, None)?;
    let stored = Value::Integer(3_u8.into());

    // When
    let result = runtime.assign_class_var(class, CLASS_VAR, stored);

    // Then
    assert!(matches!(
        result,
        Err(ConstructionError::MissingDeclaredClassVariable { .. })
    ));
    assert!(matches!(
        runtime.class_var(class, CLASS_VAR),
        Err(ConstructionError::MissingDeclaredClassVariable { .. })
    ));
    Ok(())
}
