use iris_runtime::decorator_protocol::*;
use iris_runtime::{ObjectId, Value};

#[test]
fn operation_admission_matches_the_complete_kind_table() -> Result<(), ConstructionError> {
    for kind in [
        DecoratorKind::Class,
        DecoratorKind::Module,
        DecoratorKind::Contract,
        DecoratorKind::Method,
        DecoratorKind::Property,
    ] {
        let phase = DecoratorPhase::new(kind, DecoratorReason::ClosedMaterialization);
        let empty = Transformation::empty(Some(&phase))?;
        for (operation, allowed) in [
            (
                Operation::AddMethod {
                    selector: "new_slot".into(),
                    body: Value::Closure(ObjectId::new(1)),
                },
                matches!(kind, DecoratorKind::Class | DecoratorKind::Module),
            ),
            (
                Operation::WrapMethod(Value::Closure(ObjectId::new(2))),
                kind == DecoratorKind::Method,
            ),
            (
                Operation::WrapGetter(Value::Closure(ObjectId::new(3))),
                kind == DecoratorKind::Property,
            ),
            (
                Operation::WrapSetter(Value::Closure(ObjectId::new(4))),
                kind == DecoratorKind::Property,
            ),
        ] {
            assert_eq!(empty.append(Some(&phase), operation).is_ok(), allowed);
        }
        assert_eq!(Plan::empty(Some(&phase))?.kind(), kind);
    }
    Ok(())
}

#[test]
fn public_symbol_vocabularies_are_exact() {
    assert_eq!(
        [
            DecoratorKind::Class,
            DecoratorKind::Module,
            DecoratorKind::Contract,
            DecoratorKind::Method,
            DecoratorKind::Property
        ]
        .map(DecoratorKind::symbol),
        ["class", "module", "contract", "method", "property"]
    );
    assert_eq!(
        [
            DecoratorReason::Origin,
            DecoratorReason::Open,
            DecoratorReason::Upgrade,
            DecoratorReason::Rollback,
            DecoratorReason::ClosedMaterialization
        ]
        .map(DecoratorReason::symbol),
        [
            "origin",
            "open",
            "upgrade",
            "rollback",
            "closed_materialization"
        ]
    );
    assert_eq!(
        [
            ProtocolCategory::OutsidePhase,
            ProtocolCategory::Expired,
            ProtocolCategory::ForeignTask,
            ProtocolCategory::Overlap,
            ProtocolCategory::UnfinishedInner
        ]
        .map(ProtocolCategory::symbol),
        [
            "outside_phase",
            "expired",
            "foreign_task",
            "overlap",
            "unfinished_inner"
        ]
    );
}

#[test]
fn transformations_are_persistent_and_phase_bound() -> Result<(), ConstructionError> {
    let mut phase = DecoratorPhase::new(DecoratorKind::Method, DecoratorReason::Origin);
    let empty = Transformation::empty(Some(&phase))?;
    let first = empty.append(
        Some(&phase),
        Operation::WrapMethod(Value::Closure(ObjectId::new(1))),
    )?;
    let second = first.append(
        Some(&phase),
        Operation::WrapMethod(Value::Closure(ObjectId::new(2))),
    )?;
    assert!(empty.operations().is_empty());
    assert_eq!(first.operations().len(), 1);
    assert_eq!(second.operations().len(), 2);
    let context = phase.context();
    phase.finish();
    assert_eq!(context.kind(), DecoratorKind::Method);
    assert!(
        matches!(Transformation::empty(Some(&phase)), Err(ConstructionError::Protocol(error)) if error.category() == ProtocolCategory::OutsidePhase)
    );
    Ok(())
}

#[test]
fn kinds_reject_inadmissible_operations_and_retained_tags() -> Result<(), ConstructionError> {
    let class = DecoratorPhase::new(DecoratorKind::Class, DecoratorReason::Open);
    let method = DecoratorPhase::new(DecoratorKind::Method, DecoratorReason::Upgrade);
    let empty = Transformation::empty(Some(&class))?;
    assert!(matches!(
        empty.append(Some(&class), Operation::WrapMethod(Value::Nil)),
        Err(ConstructionError::Kind(_))
    ));
    assert!(matches!(
        empty.append(Some(&method), Operation::WrapMethod(Value::Nil)),
        Err(ConstructionError::Kind(_))
    ));
    assert!(matches!(
        Plan::empty(None),
        Err(ConstructionError::Protocol(_))
    ));
    Ok(())
}
