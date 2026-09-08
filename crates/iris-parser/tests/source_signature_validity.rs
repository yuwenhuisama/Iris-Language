use iris_parser::source::{DeclarationKind, SourceKind};
use iris_parser::{parse, parse_editor, parse_with_source};

#[test]
fn rejects_header_when_singleton_parameter_channels_repeat() {
    for parameters in [
        "value, *first, *second",
        "**first, **second",
        "&first, &second",
    ] {
        let given = format!("fun read({parameters}) {{}}");

        let when = parse(&given);

        assert!(!when.program_accepted, "{given}");
        assert!(
            when.diagnostics
                .iter()
                .any(|item| item.code == "PARSE_BAD_PARAMETER_ORDER")
        );
        assert_eq!(parse_with_source(&given).parse, when);
        assert!(!parse_editor(&given).parse.program_accepted);
    }
}

#[test]
fn signature_is_invalid_when_header_damage_is_outside_written_span() {
    for header in [
        "read(value, key option, second)",
        "read(value, key option, second) -> String",
        "read(*first, *second)",
        "read(**first, **second)",
        "read(&first, &second)",
        "read<>(value)",
    ] {
        for body in [" {}", "\n{}", "", "\n"] {
            let given = format!("fun {header}{body}");

            let when = parse_with_source(&given);

            assert_eq!(when.parse, parse(&given));
            assert!(!when.parse.program_accepted, "{given}");
            assert_eq!(when.source.signatures.len(), 1, "{given}");
            assert!(!when.source.signatures[0].valid, "{given}");
            assert_eq!(
                parse_editor(&given).source.signatures,
                when.source.signatures
            );
        }
    }
}

#[test]
fn signature_is_invalid_when_header_error_code_was_already_reported() {
    let given = "fun first(key option, value) {} fun second(key option, value) {}";

    let when = parse_with_source(given);

    assert_eq!(when.parse.diagnostics.len(), 1);
    assert_eq!(when.source.recovery.len(), 2);
    assert_eq!(when.source.signatures.len(), 2);
    assert!(when.source.signatures.iter().all(|site| !site.valid));
    assert!(
        when.source
            .recovery
            .iter()
            .zip(&when.source.signatures)
            .all(|(damage, site)| damage.span.start >= site.span.end)
    );
}

#[test]
fn signature_stays_valid_when_only_body_or_call_requires_recovery() {
    for given in [
        "fun read(value, second) { read(1,",
        "fun read(value, second) {} read(1,",
        "fun read(value, second) {",
        "fun read(value, second) { let broken = ; }",
    ] {
        let when = parse_editor(given);

        assert!(!when.parse.program_accepted);
        assert_eq!(when.source.signatures.len(), 1, "{given}");
        assert!(when.source.signatures[0].valid, "{given}");
    }
}

#[test]
fn signature_is_absent_or_invalid_when_default_expression_is_missing() {
    for given in ["fun read(value = ) {}", "fun read(value = , second) {}"] {
        let when = parse_with_source(given);

        assert!(!when.parse.program_accepted);
        assert!(when.source.signatures.iter().all(|site| {
            match &when.source.node(site.owner).kind {
                SourceKind::Declaration(declaration)
                    if declaration.kind == DeclarationKind::Method =>
                {
                    !site.valid
                }
                _ => true,
            }
        }));
    }
}

#[test]
fn signature_stays_valid_when_discards_and_speculative_defaults_are_used() {
    for given in [
        "fun read(_, _) {}",
        "fun read(_, _, *rest, key option, **kwargs, &block) {}",
        "fun read(value = (A<B>>C)) {}",
        "fun read(value = (1 + 2)) {}",
        "fun read(value = { ||; 1 }) {}",
        "fun read(value, second)",
    ] {
        let when = parse_with_source(given);

        assert_eq!(when.parse, parse(given));
        assert!(when.parse.program_accepted, "{given}");
        assert!(!when.source.signatures.is_empty());
        assert!(
            when.source.signatures.iter().all(|site| site.valid),
            "{given}"
        );
        assert_eq!(
            parse_editor(given).source.signatures,
            when.source.signatures
        );
    }
}

#[test]
fn closure_signature_validity_respects_existing_grammar_and_body_boundary() {
    for given in [
        "let closure = { |_, _|; 1 }",
        "let closure = { ||; 1 }",
        "let closure = { |value| value }",
        "let closure = { 1 }",
    ] {
        let when = parse_with_source(given);

        assert!(when.parse.program_accepted, "{given}");
        assert_eq!(when.source.signatures.len(), 1);
        assert!(when.source.signatures[0].valid);
    }
    let given = "let closure = { |value|;";

    let when = parse_editor(given);

    assert!(!when.parse.program_accepted);
    assert_eq!(when.source.signatures.len(), 1);
    assert!(when.source.signatures[0].valid);
}
