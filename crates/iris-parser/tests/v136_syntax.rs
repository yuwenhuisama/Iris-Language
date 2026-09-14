use iris_parser::{analyze, parse};
use iris_syntax::{Declaration, Statement};

fn parse_codes(source: &str) -> Vec<&'static str> {
    parse(source)
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn analysis_codes(source: &str) -> Vec<&'static str> {
    let parsed = parse(source);
    assert!(
        parsed.program_accepted,
        "source must parse: {source}: {parsed:#?}"
    );
    analyze(&parsed.program)
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn parses_percent_bracket_arrays_when_prefix_is_present() {
    // Given
    let source = "%[1, value,]";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
    assert_eq!(
        format!("{:?}", result.program.statements),
        "[Expression(Array([Literal(\"1\"), Name(\"value\")]))]"
    );
}

#[test]
fn rejects_bare_bracket_array_expressions_with_targeted_diagnostic() {
    // Given / When
    let codes = parse_codes("[1, 2]");

    // Then
    assert_eq!(codes, ["PARSE_ARRAY_PREFIX_REQUIRED"]);
}

#[test]
fn keeps_bare_brackets_for_patterns_and_indexes() {
    // Given
    let source = "let values = %[1]; values[0]; for [head] in values { head }";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
}

#[test]
fn parses_contextual_is_question_and_preserves_same_question() {
    // Given
    let sources = ["value is? String", "left same? right"];

    // When / Then
    for source in sources {
        let result = parse(source);
        assert!(result.is_clean(), "{source}: {result:#?}");
    }
}

#[test]
fn rejects_legacy_is_with_targeted_diagnostic() {
    // Given / When
    let codes = parse_codes("value is String");

    // Then
    assert_eq!(codes, ["PARSE_IS_QUESTION_REQUIRED"]);
}

#[test]
fn postfix_nonnull_has_highest_precedence_and_chains() {
    // Given
    let sources = ["x!.member", "x![0]", "(foo)!()", "value! + other"];

    // When / Then
    for source in sources {
        let result = parse(source);
        assert!(result.is_clean(), "{source}: {result:#?}");
        assert!(format!("{:?}", result.program.statements).contains("NonNull"));
    }
}

#[test]
fn bang_selector_call_is_not_rewritten_as_nonnull() {
    // Given
    let source = "foo!()";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
    let shape = format!("{:?}", result.program.statements);
    assert!(shape.contains("Name(\"foo!\")"), "{shape}");
    assert!(!shape.contains("NonNull"), "{shape}");
}

#[test]
fn class_origin_retains_immutable_and_mutable_instance_fields() {
    // Given
    let source = "class Pair { let @left: Integer = 1; mut @right = 2 }";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
    let [Declaration::Class(class)] = result.program.declarations.as_slice() else {
        panic!("expected one class origin")
    };
    assert!(matches!(
        class.body.as_slice(),
        [
            Statement::InstanceField {
                mutable: false,
                name: left,
                ..
            },
            Statement::InstanceField {
                mutable: true,
                name: right,
                ..
            }
        ] if left == "left" && right == "right"
    ));
}

#[test]
fn nominal_origins_require_declaration_only_bodies() {
    // Given / When
    let class_binding = parse_codes("class Bad { let local = 1 }");
    let class_expression = parse_codes("class Bad { run() }");
    let module_binding = parse_codes("module Bad { let local = 1 }");
    let module_mutable = parse_codes("module Bad { mut local = 1 }");
    let module_deferred = parse_codes("module Bad { const local }");
    let module_expression = parse_codes("module Bad { run() }");
    let module_constants = parse(
        "module Bad { const value = 1; shared let @@state = 2; public class fun read() { 3 } }",
    );

    // Then
    assert_eq!(class_binding, ["PARSE_ORIGIN_BODY_REQUIRES_DECLARATION"]);
    assert_eq!(class_expression, ["PARSE_ORIGIN_BODY_REQUIRES_DECLARATION"]);
    assert_eq!(module_binding, ["PARSE_ORIGIN_BODY_REQUIRES_DECLARATION"]);
    assert_eq!(module_mutable, ["PARSE_ORIGIN_BODY_REQUIRES_DECLARATION"]);
    assert_eq!(module_deferred, ["PARSE_ORIGIN_BODY_REQUIRES_DECLARATION"]);
    assert_eq!(
        module_expression,
        ["PARSE_ORIGIN_BODY_REQUIRES_DECLARATION"]
    );
    assert!(module_constants.is_clean(), "{module_constants:#?}");
}

#[test]
fn open_nominal_bodies_remain_executable() {
    // Given
    let source =
        "class C {}; open class C { let local = 1; run() }; module M {}; open module M { run() }";

    // When
    let result = parse(source);

    // Then
    assert!(result.is_clean(), "{result:#?}");
}

#[test]
fn parses_top_level_impl_before_its_targets_and_exports_it() {
    // Given
    let sources = [
        "impl Box<T> for Show where T: Object { public fun show() -> String { \"box\" } }",
        "export impl Box for Show { fun show() -> String { \"box\" } }",
    ];

    // When / Then
    for source in sources {
        let result = parse(source);
        assert!(result.is_clean(), "{source}: {result:#?}");
        assert!(format!("{:?}", result.program.declarations).contains("Impl"));
    }
}

#[test]
fn top_level_impl_accepts_one_contract_and_methods_without_impl_modifier() {
    // Given / When
    let clean = parse("impl Box for Show { fun show() -> String { \"box\" } }");
    let multiple = parse_codes("impl Box for Show, Hashable { fun show() {} }");
    let marked = parse_codes("impl Box for Show { impl fun show() {} }");
    let field = parse_codes("impl Box for Show { let @value = 1 }");

    // Then
    assert!(clean.is_clean(), "{clean:#?}");
    assert_eq!(multiple, ["PARSE_IMPL_REQUIRES_ONE_CONTRACT"]);
    assert_eq!(marked, ["PARSE_LEGACY_METHOD_IMPL"]);
    assert_eq!(field, ["PARSE_IMPL_METHODS_ONLY"]);
}

#[test]
fn rejects_removed_class_for_and_method_impl_forms() {
    // Given / When
    let class_for = parse_codes("class Box for Show {}");
    let method_impl = parse_codes("class Box { impl fun show() {} }");

    // Then
    assert_eq!(class_for, ["PARSE_LEGACY_CLASS_FOR"]);
    assert_eq!(method_impl, ["PARSE_LEGACY_METHOD_IMPL"]);
}

#[test]
fn reports_duplicate_class_and_module_origins_during_analysis() {
    // Given / When
    let class_codes = analysis_codes("class Same {} class Same {}");
    let module_codes = analysis_codes("module Same {} module Same {}");

    // Then
    assert_eq!(class_codes, ["QUALIFIED_NAMESPACE_COLLISION"]);
    assert_eq!(module_codes, ["QUALIFIED_NAMESPACE_COLLISION"]);
}
