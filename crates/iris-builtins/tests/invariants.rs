use iris_builtins::{
    Availability, BuiltinType, ParameterKind, ReturnFact, Surface, class_names, members,
    service_names,
};
use std::collections::HashSet;

#[test]
fn descriptors_are_unique_when_catalog_is_loaded() {
    let catalog = members();
    let keys: HashSet<_> = catalog
        .iter()
        .map(|row| (row.owner, row.receiver, row.surface, row.selector))
        .collect();
    assert!(!catalog.is_empty());
    assert_eq!(keys.len(), catalog.len());
    for row in catalog {
        assert!(!row.documentation.is_empty());
        assert!(!row.evidence.is_empty());
        if let Some(receiver) = row.receiver {
            assert_eq!(BuiltinType::from_name(receiver.name()), Some(receiver));
        }
        for shape in row.shapes {
            for parameter in shape.parameters {
                match parameter.kind {
                    ParameterKind::Positional | ParameterKind::Rest => assert!(matches!(
                        parameter.label,
                        "arg1" | "arg2" | "arg3" | "arg4" | "callback"
                    )),
                    ParameterKind::Keyword => assert!(matches!(
                        parameter.label,
                        "step"
                            | "errors"
                            | "canonical"
                            | "depth"
                            | "element_limit"
                            | "declarations"
                            | "core_abi"
                            | "replaces_core_regex_literals"
                    )),
                    ParameterKind::Block => assert_eq!(parameter.label, "callback"),
                }
            }
        }
    }
}

#[test]
fn name_routes_are_separate_when_receiver_families_exist() {
    let classes: HashSet<_> = class_names().iter().copied().collect();
    assert_eq!(
        classes,
        HashSet::from([
            "Object", "Nil", "Bool", "Integer", "Float32", "Float64", "String"
        ])
    );
    assert!(service_names().contains(&"JSON"));
    assert!(service_names().contains(&"Reflection::Package"));
    assert!(!service_names().contains(&"File"));
    assert_eq!(BuiltinType::from_name("Text"), None);
    assert_eq!(BuiltinType::from_name("Array"), Some(BuiltinType::Array));
}

#[test]
fn alternatives_stay_distinct_when_arity_or_backend_changes() {
    for (selector, counts) in [
        ("count", vec![0, 1]),
        ("reduce", vec![1, 2]),
        ("slice", vec![2, 1]),
    ] {
        let rows: Vec<_> = members()
            .iter()
            .filter(|row| {
                row.receiver == Some(BuiltinType::Array)
                    && row.surface == Surface::Instance
                    && row.selector == selector
            })
            .collect();
        assert_eq!(rows.len(), 1, "{selector}");
        assert_eq!(
            rows[0]
                .shapes
                .iter()
                .map(|shape| shape.parameters.len())
                .collect::<Vec<_>>(),
            counts
        );
        if selector == "slice" {
            assert_eq!(rows[0].shapes[1].availability, Availability::Reference);
        }
    }
}

#[test]
fn callable_signature_is_unknown_when_callable_is_arbitrary() {
    for family in [BuiltinType::Closure, BuiltinType::BoundMethod] {
        let rows: Vec<_> = members()
            .iter()
            .filter(|row| row.receiver == Some(family) && row.selector == "call")
            .collect();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].shapes.is_empty());
        assert_eq!(rows[0].result, ReturnFact::Unknown);
    }
}

#[test]
fn refusals_are_absent_when_public_surface_is_requested() {
    for row in members() {
        assert!(!matches!(
            row.selector,
            "share_count" | "prune" | "rollback" | "reactivate" | "[]" | "[]="
        ));
        assert!(!matches!(
            row.owner,
            "NativeFixture" | "NativeResource" | "File" | "Encoding"
        ));
        assert!(!(row.owner == "FFI::Library" && row.selector == "call"));
        assert!(!(row.owner == "Method" && row.selector == "call"));
        assert!(
            !(matches!(
                row.owner,
                "Array" | "MutableString" | "ByteArray" | "Hash" | "Match" | "ReadonlyArray"
            ) && row.selector == "hash")
        );
        assert!(
            !(matches!(
                row.owner,
                "Array" | "Hash" | "Bytes" | "ByteArray" | "Tuple" | "Regex" | "Task"
            ) && row.selector == "new")
        );
    }
}
