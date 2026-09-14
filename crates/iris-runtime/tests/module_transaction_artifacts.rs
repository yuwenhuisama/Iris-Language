use iris_runtime::{
    ClassRegistry, CompositionEdge, MetaCapabilities, MethodBody, ModuleMethodDefinition, Selector,
    StaticSpine, Visibility,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn public_declaration_addition_is_checked_without_granting_decorator_authority() -> TestResult {
    let mut given = ClassRegistry::new();
    let module = given.stage_module_origin(&[], MetaCapabilities::all())?;
    given.commit_structural_group()?;
    given.begin_module_transaction(module)?;
    let definition =
        ModuleMethodDefinition::new(Selector::new(1), MethodBody::new(1), Visibility::Public);
    assert!(matches!(
        given.stage_module_method(module, definition),
        Err(iris_runtime::ModuleCandidateError::PublicAdditionDenied { .. })
    ));

    let when = given.stage_module_declaration_method(module, definition)?;
    given.commit_structural_group()?;

    assert_eq!(given.module_method(module, Selector::new(1)), Some(when));
    let denied = given.stage_module_origin(
        &[],
        MetaCapabilities::denying(&[iris_runtime::Capability::MethodSet]),
    )?;
    given.commit_structural_group()?;
    given.begin_module_transaction(denied)?;
    assert!(matches!(
        given.stage_module_declaration_method(denied, definition),
        Err(iris_runtime::ModuleCandidateError::CapabilityDenied { .. })
    ));
    Ok(())
}

#[test]
fn provisional_artifacts_are_removed_when_group_validation_fails() -> TestResult {
    let mut given = ClassRegistry::new();
    let class = given.define_class(StaticSpine::new(1), None)?;
    let retained = given.publish_origin_method(
        class,
        Selector::new(1),
        MethodBody::new(1),
        Visibility::Public,
    )?;
    given.begin_transaction(class)?;
    let provisional = given.publish_origin_method(
        class,
        Selector::new(2),
        MethodBody::new(2),
        Visibility::Public,
    )?;
    let module = given.stage_module_origin(&[], MetaCapabilities::all())?;
    let first = given.stage_module_origin_method(
        module,
        ModuleMethodDefinition::new(Selector::new(1), MethodBody::new(3), Visibility::Public),
    )?;
    let replacement =
        given.stage_module_method_body(module, Selector::new(1), MethodBody::new(4))?;
    given.stage_module_composition(module, &[CompositionEdge::new(module, false)])?;

    let when = given.commit_structural_group();

    assert!(when.is_err());
    assert_eq!(given.method_by_id(retained.id()), Some(retained));
    for method in [provisional, first, replacement] {
        assert_eq!(given.method_by_id(method.id()), None);
    }
    Ok(())
}

#[test]
fn active_method_is_retained_when_open_candidate_is_aborted() -> TestResult {
    let mut given = ClassRegistry::new();
    let module = given.stage_module_origin(&[], MetaCapabilities::all())?;
    let old = given.stage_module_origin_method(
        module,
        ModuleMethodDefinition::new(Selector::new(1), MethodBody::new(1), Visibility::Public),
    )?;
    given.commit_structural_group()?;
    given.begin_module_transaction(module)?;
    let new = given.stage_module_method_body(module, Selector::new(1), MethodBody::new(2))?;

    given.roll_back_group();

    assert_eq!(given.module_method(module, Selector::new(1)), Some(old));
    assert_eq!(given.method_by_id(old.id()), Some(old));
    assert_eq!(given.method_by_id(new.id()), None);
    Ok(())
}
