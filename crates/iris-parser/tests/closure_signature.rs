use iris_parser::{parse, parse_editor, parse_with_source};
use iris_syntax::{Expression, Parameter, ParameterCategory, Statement, TypeExpression};

fn closure(source: &str) -> Expression {
    let parsed = parse(source);
    assert!(parsed.program_accepted, "{source}: {parsed:?}");
    let Statement::Binding { value, .. } = &parsed.program.statements[0] else {
        panic!("expected binding")
    };
    value.clone()
}

fn parameters(source: &str) -> Vec<Parameter> {
    let Expression::Closure {
        parameters,
        full_parameters,
        ..
    } = closure(source)
    else {
        panic!("expected closure")
    };
    let names: Vec<_> = full_parameters.iter().map(|value| &value.name).collect();
    assert_eq!(parameters.iter().collect::<Vec<_>>(), names);
    full_parameters
}

#[test]
fn retains_annotation_when_parameter_is_typed() {
    let source = "let callback = { |value: Integer| -> String; 'result' }";

    let actual = parameters(source);

    assert_eq!(actual[0].name, "value");
    assert_eq!(actual[0].category, ParameterCategory::Positional);
    assert_eq!(actual[0].default, None);
    assert_eq!(
        actual[0].annotation,
        Some(TypeExpression::Name("Integer".into()))
    );
}

#[test]
fn retains_every_channel_when_header_has_complete_signature() {
    let source = "let callback = { |value: Integer, optional: Integer = 1, *rest: Integer, key required: String, key option: String = 'yes', **keywords: Object, &block: Block<() -> Nil> = nil| -> Object; value }";

    let actual = parameters(source);

    let names: Vec<_> = actual.iter().map(|value| value.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "value", "optional", "rest", "required", "option", "keywords", "block"
        ]
    );
    let categories: Vec<_> = actual.iter().map(|value| value.category).collect();
    assert_eq!(
        categories,
        [
            ParameterCategory::Positional,
            ParameterCategory::Positional,
            ParameterCategory::Rest,
            ParameterCategory::Keyword,
            ParameterCategory::Keyword,
            ParameterCategory::KeywordRest,
            ParameterCategory::Block,
        ]
    );
    let method = parse(&format!(
        "fun callback({}) -> Object {{ value }}",
        source.split('|').nth(1).expect("header")
    ));
    let Statement::Method(method) = &method.program.statements[0] else {
        panic!("expected method")
    };
    assert_eq!(actual, method.parameters);
    assert_eq!(actual[1].default, Some(Expression::Literal("1".into())));
    assert_eq!(actual[6].default, Some(Expression::Name("nil".into())));
}

#[test]
fn retains_default_expression_when_operators_and_nested_delimiters_occur() {
    for default in [
        "1 + 2 * 3 < 9 && true || false",
        "(1 | 2) + 4",
        "combine(1 | 2, %[3 | 4])",
        "saved = 1 + 2",
        "{ |nested: Integer = (1 | 2)| -> Integer; nested | 4 }",
        "if true { 1 | 2 } else { 3 | 4 }",
        "(Object).type",
        "value as Integer",
        "-1 + 3",
        "!false",
        "await work()",
        "yield 1",
    ] {
        let source = format!("let callback = {{ |value: Object = {default}| -> Object; value }}");
        let expected = closure(&format!("let expected = {default}"));

        let actual = parameters(&source);

        assert_eq!(actual[0].default.as_ref(), Some(&expected), "{default}");
    }
}

#[test]
fn rejects_missing_defaults_when_closure_header_is_damaged() {
    for header in ["value =", "value = +", "value: = 1", "key", "&"] {
        let source = format!("let callback = {{ |{header}| -> Nil; nil }}");

        let parsed = parse_with_source(&source);

        assert!(!parsed.parse.program_accepted, "{header}");
        assert_eq!(parsed.parse, parse(&source));
    }
}

#[test]
fn retains_async_mode_and_generic_next_when_source_declares_them() {
    let source = "let callback = { async |next: Closure<(ArgumentChanges) -> Task<Object>>| -> Object; await next.call(ArgumentChanges.keep) }";

    let actual = parameters(source);

    assert_eq!(
        actual[0].annotation,
        Some(TypeExpression::Generic {
            name: "Closure".into(),
            arguments: vec![TypeExpression::Function {
                parameters: vec![TypeExpression::Name("ArgumentChanges".into())],
                result: Box::new(TypeExpression::Generic {
                    name: "Task".into(),
                    arguments: vec![TypeExpression::Name("Object".into())],
                }),
            }],
        })
    );
}

#[test]
fn preserves_header_and_mode_when_empty_async_or_absent() {
    for (source, expected_header, expected_async, expected_return) in [
        ("let callback = { 1 }", false, false, None),
        ("let callback = { || 1 }", true, false, None),
        (
            "let callback = { || -> Integer; 1 }",
            true,
            false,
            Some("Integer"),
        ),
        (
            "let callback = { async || -> Integer; 1 }",
            true,
            true,
            Some("Integer"),
        ),
        (
            "let callback = { async |value| -> Object; value }",
            true,
            true,
            Some("Object"),
        ),
    ] {
        let Expression::Closure {
            has_header,
            is_async,
            return_type,
            ..
        } = closure(source)
        else {
            panic!("expected closure")
        };

        assert_eq!((has_header, is_async), (expected_header, expected_async));
        assert_eq!(
            return_type,
            expected_return.map(|name| TypeExpression::Name(name.into()))
        );
    }
}

#[test]
fn rejects_bad_channel_order_when_parameters_repeat_or_move_backwards() {
    for header in [
        "key named, positional",
        "*first, *second",
        "**first, **second",
        "&first, &second",
    ] {
        let source = format!("let callback = {{ |{header}| -> Nil; nil }}");

        let recorded = parse_with_source(&source);

        assert!(!recorded.parse.program_accepted);
        assert!(
            recorded
                .parse
                .diagnostics
                .iter()
                .any(|value| value.code == "PARSE_BAD_PARAMETER_ORDER")
        );
        assert!(recorded.source.signatures.iter().all(|value| !value.valid));
    }
}

#[test]
fn preserves_source_graph_when_complete_header_and_editor_prefixes_are_parsed() {
    let source = "let callback = { |_: Integer, key option: Integer = (1 | 2), &block: Block<() -> Nil> = nil| -> Object; option }";

    let recorded = parse_with_source(source);

    assert!(recorded.parse.program_accepted, "{:?}", recorded.parse);
    assert_eq!(recorded.parse, parse(source));
    assert_eq!(recorded.source.parameter_slots.len(), 3);
    let option = &recorded.source.parameter_slots[1];
    assert_eq!(option.category, ParameterCategory::Keyword);
    assert!(option.default.is_some());
    assert!(recorded.source.parameter_slots[0].declaration.is_none());
    assert!(recorded.source.signatures.iter().all(|value| value.valid));
    for end in 0..source.len() {
        let prefix = &source[..end];
        let editor = parse_editor(prefix);
        assert_eq!(parse_with_source(prefix).parse, parse(prefix));
        for slot in &editor.source.parameter_slots {
            assert!(slot.owner.0 < editor.source.nodes.len());
            for id in [slot.annotation, slot.default, slot.declaration]
                .into_iter()
                .flatten()
            {
                assert!(id.0 < editor.source.nodes.len());
            }
        }
    }
}
