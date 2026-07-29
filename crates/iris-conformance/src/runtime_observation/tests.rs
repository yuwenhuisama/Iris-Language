use super::compare_runtime;
use crate::model::Record;

fn record(source: &str, independent_sources: Vec<&str>, expect: &str) -> Record {
    Record {
        id: "test".into(),
        source: source.into(),
        independent_sources: independent_sources.into_iter().map(str::to_owned).collect(),
        expect: expect.into(),
        tags: vec![],
    }
}

#[test]
fn expected_division_by_zero_is_accepted() {
    // Given
    let record = record(
        "1 div 0",
        vec![],
        "{\"error\":{\"code\":\"DivisionByZeroError\"}}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}

#[test]
fn canonical_numeric_bytes_is_reported_as_message_not_found() {
    // Given
    let record = record(
        "Integer(1).canonical_numeric_bytes()",
        vec![],
        "{\"error\":{\"code\":\"MessageNotFoundError\"}}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}

#[test]
fn absent_class_selectors_are_reported_as_message_not_found() {
    // Given
    let record = record(
        "A.new_current(); A.new_checked()",
        vec![],
        "{\"error\":{\"code\":\"MessageNotFoundError\"}}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}

#[test]
fn runtime_v016_fixture_uses_a_fresh_bound_method_read() {
    // Given
    let mut record = record(
        "obj.method same? obj.method",
        vec![],
        "{\"value\":{\"bool\":false}}",
    );
    record.id = "IRIS-V1-RUNTIME-V016".into();

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}

#[test]
fn runtime_v093_fixture_observes_typed_no_super_method_error() {
    // Given
    let mut record = record(
        "bare super; explicit super() with no successor",
        vec![],
        "{\"error\":{\"code\":\"NoSuperMethodError\"}}",
    );
    record.id = "IRIS-V1-RUNTIME-V093".into();

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}

#[test]
fn v041_sources_each_map_to_domain_error() {
    // Given
    let record = record(
        "",
        vec![
            "0 ** 0",
            "0.0f32 ** 0.0f32",
            "Float64.from_bits(0x8000000000000000) ** -0.0f64",
        ],
        "{\"independent_expectations\":[{\"error\":{\"code\":\"DomainError\"}},{\"error\":{\"code\":\"DomainError\"}},{\"error\":{\"code\":\"DomainError\"}}]}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}

#[test]
fn independent_source_mismatch_is_reported() {
    // Given
    let record = record(
        "",
        vec!["0 ** 0", "0.0f32 ** 0.0f32"],
        "{\"independent_expectations\":[{\"error\":{\"code\":\"DomainError\"}},{\"error\":{\"code\":\"IdentityError\"}}]}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert!(result.is_err());
}

#[test]
fn value_and_side_effects_are_accepted_when_both_match() {
    // Given
    let record = record(
        "class Probe { class fun observe() { log.append(:observed); [true, log] } }; let mut log = []; Probe.observe()",
        vec![],
        "{\"value\":{\"bool\":true},\"side_effects\":{\"array\":[{\"symbol\":\"observed\"}]}}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}

#[test]
fn side_effect_mismatch_is_reported_when_value_matches() {
    // Given
    let record = record(
        "class Probe { class fun observe() { log.append(:observed); [true, log] } }; let mut log = []; Probe.observe()",
        vec![],
        "{\"value\":{\"bool\":true},\"side_effects\":{\"array\":[{\"symbol\":\"missing\"}]}}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert!(result.is_err());
}

#[test]
fn side_effects_without_value_are_accepted_from_a_single_item_observation() {
    // Given
    let record = record(
        "class Probe { class fun observe() { log.append(:observed); [log] } }; let mut log = []; Probe.observe()",
        vec![],
        "{\"side_effects\":{\"array\":[{\"symbol\":\"observed\"}]}}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}

#[test]
fn visibility_denial_is_reported_as_method_visibility_error() {
    // Given
    let record = record(
        "class A { private fun secret() -> Integer { 1 } }; let a = A.new(); a.secret()",
        vec![],
        "{\"error\":{\"code\":\"MethodVisibilityError\"}}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}

#[test]
fn error_and_side_effects_assert_a_failed_declaration_is_not_published() {
    // Given
    let record = record(
        "class Base meta deny subclass { }; class Child extends Base { }",
        vec![],
        "{\"error\":{\"code\":\"MetaOperationError\"},\"side_effects\":\"Child is not published.\"}",
    );

    // When
    let result = compare_runtime(&record);

    // Then
    assert_eq!(result, Ok(()));
}
