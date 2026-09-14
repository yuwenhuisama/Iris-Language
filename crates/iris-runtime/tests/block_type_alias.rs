use iris_runtime::{
    CallableKind, CallableSignature, ClassError, ClassId, ClassRegistry, ComposedType, CoreClass,
    NominalType, Runtime, SignatureType, StaticSpine, TypeAtom, Value,
};

fn nominal(identity: u64) -> SignatureType {
    NominalType::new(ClassId::new(identity), vec![]).into()
}

fn atom(value: Value) -> TypeAtom {
    let Value::Type(identity, arguments) = value else {
        unreachable!()
    };
    TypeAtom::Nominal(identity, arguments)
}

#[test]
fn block_equals_explicit_union_when_members_are_interned_in_either_order()
-> Result<(), Box<dyn std::error::Error>> {
    for kinds in [
        [CallableKind::BoundMethod, CallableKind::Closure],
        [CallableKind::Closure, CallableKind::BoundMethod],
    ] {
        let mut given = ClassRegistry::new();
        let signature = CallableSignature::new(vec![nominal(1)], nominal(2));
        let mut members = kinds
            .into_iter()
            .map(|kind| {
                given
                    .intern_callable_type(kind, signature.clone())
                    .map(atom)
            })
            .collect::<Result<Vec<_>, _>>()?;
        members.sort();
        let expected = Value::ComposedType(ComposedType::Union(members));

        let when = given.intern_block_alias(signature)?;

        assert_eq!(when, expected);
    }
    Ok(())
}

#[test]
fn block_contains_both_canonical_kinds_when_alias_is_interned_first()
-> Result<(), Box<dyn std::error::Error>> {
    let mut given = ClassRegistry::new();
    let signature = CallableSignature::new(vec![nominal(1)], nominal(2));

    let when = given.intern_block_alias(signature.clone())?;

    let Value::ComposedType(ComposedType::Union(members)) = &when else {
        unreachable!()
    };
    assert_eq!(members.len(), 2);
    for kind in [CallableKind::BoundMethod, CallableKind::Closure] {
        let member = atom(given.intern_callable_type(kind, signature.clone())?);
        assert!(members.contains(&member));
        let TypeAtom::Nominal(identity, arguments) = member else {
            unreachable!()
        };
        assert!(arguments.is_empty());
        let descriptor = given.callable_type(identity).ok_or("missing callable")?;
        assert_eq!(descriptor.kind(), kind);
        assert_eq!(descriptor.signature(), &signature);
    }
    assert_eq!(given.intern_block_alias(signature)?, when);
    Ok(())
}

#[test]
fn block_excludes_mismatched_signatures_when_parameter_or_result_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut given = ClassRegistry::new();
    let parent = given.define_class(StaticSpine::new(1), None)?;
    let child = given.define_class(StaticSpine::new(2), Some(parent))?;
    let signature = CallableSignature::new(
        vec![nominal(parent.raw()), nominal(child.raw())],
        nominal(parent.raw()),
    );
    let variants = [
        CallableSignature::new(
            vec![nominal(child.raw()), nominal(parent.raw())],
            nominal(parent.raw()),
        ),
        CallableSignature::new(vec![nominal(parent.raw())], nominal(parent.raw())),
        CallableSignature::new(
            vec![nominal(parent.raw()), nominal(child.raw())],
            nominal(child.raw()),
        ),
        CallableSignature::new(
            vec![nominal(child.raw()), nominal(child.raw())],
            nominal(parent.raw()),
        ),
    ];

    let when = given.intern_block_alias(signature)?;

    let Value::ComposedType(ComposedType::Union(members)) = &when else {
        unreachable!()
    };
    for variant in variants {
        for kind in [CallableKind::BoundMethod, CallableKind::Closure] {
            assert!(!members.contains(&atom(given.intern_callable_type(kind, variant.clone())?)));
        }
        assert_ne!(given.intern_block_alias(variant)?, when);
    }
    Ok(())
}

#[test]
fn block_members_are_not_classes_when_allocation_is_attempted()
-> Result<(), Box<dyn std::error::Error>> {
    let mut given = Runtime::new();
    let Value::ComposedType(ComposedType::Union(members)) = given
        .registry_mut()
        .intern_block_alias(CallableSignature::new(vec![], ComposedType::Never.into()))?
    else {
        unreachable!()
    };

    for member in members {
        let TypeAtom::Nominal(identity, _) = member else {
            unreachable!()
        };
        let when = given.allocate(identity);

        assert!(when.is_err());
        assert_eq!(
            given.registry().class(identity),
            Err(ClassError::UnknownClassId(identity))
        );
        assert!(given.registry().active(identity).is_err());
    }
    Ok(())
}

#[test]
fn user_publication_sequences_are_preserved_when_block_alias_is_interned()
-> Result<(), Box<dyn std::error::Error>> {
    let mut given = ClassRegistry::new();
    let mut control = ClassRegistry::new();
    let signature = CallableSignature::new(vec![], nominal(CoreClass::Task.id().raw()));

    let _when = given.intern_block_alias(signature)?;

    for ordinal in [1, 2] {
        let actual = given.define_class(StaticSpine::new(ordinal), None)?;
        let expected = control.define_class(StaticSpine::new(ordinal), None)?;
        assert_eq!(actual, expected);
        assert_eq!(given.active(actual)?, control.active(expected)?);
        assert_eq!(
            given.publish(given.open(actual)?)?,
            control.publish(control.open(expected)?)?
        );
    }
    assert!(given.callable_type(CoreClass::Task.id()).is_none());
    Ok(())
}

#[test]
fn nested_task_result_is_invariant_when_block_alias_result_argument_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut given = ClassRegistry::new();
    let signatures = [1, 2].map(|identity| {
        CallableSignature::new(
            vec![],
            NominalType::new(
                CoreClass::Task.id(),
                vec![NominalType::new(
                    CoreClass::Task.id(),
                    vec![NominalType::new(ClassId::new(identity), vec![])],
                )],
            )
            .into(),
        )
    });
    let first = given.intern_block_alias(signatures[0].clone())?;

    let when = given.intern_block_alias(signatures[1].clone())?;

    assert_ne!(first, when);
    for signature in signatures {
        for kind in [CallableKind::BoundMethod, CallableKind::Closure] {
            let Value::Type(identity, _) = given.intern_callable_type(kind, signature.clone())?
            else {
                unreachable!()
            };
            assert_eq!(
                given
                    .callable_type(identity)
                    .ok_or("missing callable")?
                    .signature(),
                &signature
            );
        }
    }
    Ok(())
}

#[test]
fn nested_alias_signature_reuses_identity_when_result_is_expanded_union()
-> Result<(), Box<dyn std::error::Error>> {
    let mut given = ClassRegistry::new();
    let signature = CallableSignature::new(vec![], ComposedType::Never.into());
    let Value::ComposedType(alias) = given.intern_block_alias(signature.clone())? else {
        unreachable!()
    };
    let mut members = [CallableKind::Closure, CallableKind::BoundMethod]
        .into_iter()
        .map(|kind| {
            given
                .intern_callable_type(kind, signature.clone())
                .map(atom)
        })
        .collect::<Result<Vec<_>, _>>()?;
    members.sort();
    let expected = given.intern_callable_type(
        CallableKind::BoundMethod,
        CallableSignature::new(vec![], ComposedType::Union(members).into()),
    )?;

    let when = given.intern_callable_type(
        CallableKind::BoundMethod,
        CallableSignature::new(vec![], alias.into()),
    )?;

    assert_eq!(when, expected);
    Ok(())
}
