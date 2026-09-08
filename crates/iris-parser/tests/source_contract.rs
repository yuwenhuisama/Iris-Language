use iris_parser::source::{ArgumentKind, DeclarationKind, SourceKind};
use iris_parser::{parse, parse_editor, parse_with_source};

#[test]
fn documentation_when_standalone_run_precedes_declaration() {
    let text = "/// First\n/// second\nfun same() {}\n\n/// Other\nfun same() {}";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert_eq!(result.source.documentation.len(), 2);
    assert_eq!(result.source.documentation[0].text, "First\nsecond");
    assert_eq!(result.source.documentation[1].text, "Other");
    assert_ne!(
        result.source.documentation[0].declaration,
        result.source.documentation[1].declaration
    );
}

#[test]
fn parameter_slots_when_discard_has_default() -> Result<(), &'static str> {
    let text = "fun use(_: Integer = seed(), key value: String = 'text') {}";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    let slots = &result.source.parameter_slots;
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].name.text, "_");
    assert_eq!(slots[0].declaration, None);
    assert!(slots[0].default.is_some());
    assert!(slots[0].annotation.is_some());
    assert_eq!(slots[0].owner, slots[1].owner);
    assert_eq!(result.source.signatures.len(), 1);
    let SourceKind::Declaration(method) = &result.source.node(slots[0].owner).kind else {
        return Err("method");
    };
    assert_eq!(method.kind, DeclarationKind::Method);
    assert_eq!(method.parameters.len(), 1);
    Ok(())
}

#[test]
fn call_slots_when_editor_argument_is_unfinished() {
    let result = parse_editor("outer(inner(1, key: value +");
    assert!(!result.parse.program_accepted);
    assert_eq!(result.source.calls.len(), 2);
    let inner = &result.source.calls[0];
    assert_eq!(inner.commas.len(), 1);
    assert_eq!(inner.close, None);
    assert_eq!(inner.arguments.len(), 2);
    assert!(matches!(&inner.arguments[1].kind, ArgumentKind::Keyword(name) if name.text == "key"));
    assert!(inner.arguments[1].incomplete);
    assert_eq!(inner.arguments[1].expression, None);
}

#[test]
fn categories_when_all_parameter_channels_are_written() {
    use iris_syntax::ParameterCategory;
    let text = "fun target(_, value=1, *rest, key option, **options, &block) {}";
    let result = parse_with_source(text);
    assert_eq!(result.parse, parse(text));
    assert!(result.parse.program_accepted);
    assert_eq!(
        result
            .source
            .parameter_slots
            .iter()
            .map(|slot| slot.category)
            .collect::<Vec<_>>(),
        [
            ParameterCategory::Positional,
            ParameterCategory::Positional,
            ParameterCategory::Rest,
            ParameterCategory::Keyword,
            ParameterCategory::KeywordRest,
            ParameterCategory::Block
        ]
    );
    assert_eq!(result.source.parameter_slots[0].declaration, None);
    assert_eq!(
        &text[result.source.signatures[0].span.start..result.source.signatures[0].span.end],
        "fun target(_, value=1, *rest, key option, **options, &block)"
    );
}
