use super::*;

fn name(value: &str) -> TypeExpression {
    TypeExpression::Name(value.into())
}

fn union(members: Vec<TypeExpression>) -> TypeExpression {
    TypeExpression::Union(members)
}

fn intersection(members: Vec<TypeExpression>) -> TypeExpression {
    TypeExpression::Intersection(members)
}

fn callable(kind: &str, parameters: Vec<TypeExpression>, result: TypeExpression) -> TypeExpression {
    TypeExpression::Generic {
        name: kind.into(),
        arguments: vec![TypeExpression::Function {
            parameters,
            result: Box::new(result),
        }],
    }
}

#[test]
fn normalization_preserves_identity_when_nil_constraints_are_simplified() {
    for (given, expected) in [
        (
            intersection(vec![name("NonNil"), name("NonNil")]),
            name("NonNil"),
        ),
        (
            intersection(vec![name("Nil"), name("NonNil")]),
            name("Never"),
        ),
        (
            intersection(vec![
                union(vec![
                    intersection(vec![name("Nil"), name("Unknown")]),
                    name("Nil"),
                ]),
                name("NonNil"),
            ]),
            name("Never"),
        ),
        (
            intersection(vec![
                union(vec![name("String"), name("Nil")]),
                name("NonNil"),
            ]),
            name("String"),
        ),
        (
            intersection(vec![
                union(vec![name("Unknown"), name("Nil")]),
                name("NonNil"),
            ]),
            intersection(vec![name("Unknown"), name("NonNil")]),
        ),
        (
            intersection(vec![name("Object"), name("NonNil")]),
            intersection(vec![name("Object"), name("NonNil")]),
        ),
        (
            intersection(vec![name("NonNil"), name("Object")]),
            intersection(vec![name("NonNil"), name("Object")]),
        ),
        (
            intersection(vec![
                union(vec![name("String"), name("Integer"), name("Nil")]),
                name("NonNil"),
            ]),
            union(vec![name("String"), name("Integer")]),
        ),
        (
            intersection(vec![
                union(vec![
                    intersection(vec![name("String"), name("Integer")]),
                    name("Nil"),
                ]),
                name("NonNil"),
            ]),
            intersection(vec![name("String"), name("Integer")]),
        ),
    ] {
        let admission = Admission::new(&[], &[]);

        let when = admission.normalize_signature_type(&given);

        assert_eq!(when, expected, "{given:?}");
    }
}

#[test]
fn normalization_preserves_structure_when_boolean_operators_are_mixed() {
    for given in [
        union(vec![
            intersection(vec![name("String"), name("Integer")]),
            name("Nil"),
        ]),
        intersection(vec![
            name("String"),
            union(vec![name("Integer"), name("Nil")]),
        ]),
    ] {
        let admission = Admission::new(&[], &[]);

        let when = admission.normalize_signature_type(&given);

        assert_eq!(when, given);
    }
}

#[test]
fn nonnil_proof_stays_conservative_when_members_are_composed() {
    for (given, expected) in [
        (name("Unknown"), false),
        (name("Object"), false),
        (name("Nil"), false),
        (name("NonNil"), true),
        (name("String"), true),
        (union(vec![name("String"), name("Unknown")]), false),
        (union(vec![name("String"), name("Integer")]), true),
        (intersection(vec![name("String"), name("Unknown")]), true),
        (intersection(vec![name("Object"), name("Unknown")]), false),
    ] {
        let admission = Admission::new(&[], &[]);

        let when = admission.provably_non_nil(&given);

        assert_eq!(when, expected, "{given:?}");
    }
}

#[test]
fn absorption_rejects_object_when_nonnil_is_nested_in_the_bound() {
    let admission = Admission::new(&[], &[]);
    let given = intersection(vec![name("Object"), name("NonNil")]);

    let when = admission.signature_subtype(&name("Object"), &given);

    assert!(!when);
}

#[test]
fn callable_equivalence_preserves_invariance_when_signatures_differ() {
    let admission = Admission::new(&[], &[]);
    let bound = callable("Closure", vec![name("String")], name("Integer"));
    for given in [
        callable("BoundMethod", vec![name("String")], name("Integer")),
        callable("Closure", Vec::new(), name("Integer")),
        callable("Closure", vec![name("Object")], name("Integer")),
        callable("Closure", vec![name("String")], name("Object")),
    ] {
        let when = equivalent(&admission, &given, &bound);

        assert!(!when, "{given:?}");
    }
}

#[test]
fn callable_equivalence_accepts_when_nested_signatures_normalize() {
    let admission = Admission::new(&[], &[]);
    let given = callable(
        "Closure",
        vec![intersection(vec![name("NonNil"), name("NonNil")])],
        name("Integer"),
    );
    let expected = callable("Closure", vec![name("NonNil")], name("Integer"));
    let given = callable("Closure", vec![given], name("String"));
    let expected = callable("Closure", vec![expected], name("String"));

    let when = equivalent(&admission, &given, &expected);

    assert!(when);
}
