#[test]
fn strict_result_when_metadata_is_disabled() {
    let source = "class Box { fun make(value = (A<B>>C)) { { ||; value } } }";
    let strict = iris_parser::parse(source);
    let recorded = iris_parser::parse_with_source(source);
    assert_eq!(strict, recorded.parse);
    assert!(strict.program_accepted);
}
