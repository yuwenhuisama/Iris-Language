use iris_parser::source::{DeclarationKind, SourceKind};
use iris_parser::{parse, parse_with_source, prepare_bindings};
use iris_syntax::{Declaration, Decorator, ExportDeclaration, Expression, ProgramEntry};

fn wrapped(declaration: &Declaration) -> &Declaration {
    let Declaration::Export(export) = declaration else {
        panic!("expected export: {declaration:?}");
    };
    let ExportDeclaration::Declaration(inner) = export.as_ref() else {
        panic!("expected wrapped declaration: {export:?}");
    };
    inner
}

fn decorators(declaration: &Declaration) -> &[Decorator] {
    match declaration {
        Declaration::Class(value) => &value.decorators,
        Declaration::Module(value) => &value.decorators,
        Declaration::Contract(value) => &value.decorators,
        Declaration::Impl(_)
        | Declaration::Export(_)
        | Declaration::Import(_)
        | Declaration::TypeAlias(_) => {
            panic!("expected nominal declaration: {declaration:?}");
        }
    }
}

#[test]
fn retains_decorators_when_export_wraps_a_nominal_declaration() {
    for nominal in ["class", "module", "contract", "open class", "open module"] {
        for applications in ["@D()", "@D() @Tools::Stamp(1, :tag)"] {
            let given = format!("export {applications} {nominal} Target {{}}");
            let reference = parse(&format!("{applications} {nominal} Target {{}}"));
            assert!(reference.program_accepted);

            let when = parse(&given);

            assert!(when.program_accepted, "{given}: {:?}", when.diagnostics);
            assert_eq!(when.program.declarations.len(), 1);
            let inner = wrapped(&when.program.declarations[0]);
            assert_eq!(inner, &reference.program.declarations[0]);
            assert_eq!(
                decorators(inner)[0],
                Decorator {
                    name: "D".into(),
                    arguments: vec![]
                }
            );
            if applications.contains("Stamp") {
                assert_eq!(
                    decorators(inner)[1],
                    Decorator {
                        name: "Tools::Stamp".into(),
                        arguments: vec![
                            Expression::Literal("1".into()),
                            Expression::Symbol("tag".into())
                        ],
                    }
                );
            }
            assert_eq!(
                when.program.entries,
                vec![ProgramEntry::Declaration(
                    when.program.declarations[0].clone()
                )]
            );
            assert!(when.program.statements.is_empty());
        }
    }
}

#[test]
fn preserves_order_when_exported_decorators_are_normalized() {
    let given = parse(
        "type Alias = Integer; let before: Integer = 1; export @D((Alias).type) class C {}; let middle: Integer = 2; export @D((Alias).type) module M {}; export @D((Alias).type) contract K {}; let after: Integer = 3",
    );
    assert!(given.program_accepted, "{:?}", given.diagnostics);
    let expected = parse(
        "type Alias = Integer; let before: Integer = 1; export @D((Integer).type) class C {}; let middle: Integer = 2; export @D((Integer).type) module M {}; export @D((Integer).type) contract K {}; let after: Integer = 3",
    );
    assert!(expected.program_accepted, "{:?}", expected.diagnostics);

    let when = prepare_bindings(&given.program).expect("valid alias normalization");

    assert_eq!(when, expected.program);
    for declaration in &when.declarations[1..] {
        assert_eq!(decorators(wrapped(declaration)).len(), 1);
    }
    assert_eq!(given.program.entries.len(), 7);
}

#[test]
fn records_exported_nominal_metadata_when_decorators_are_child_nodes() {
    for (nominal, kind, reopen) in [
        ("class", DeclarationKind::Class, false),
        ("module", DeclarationKind::Module, false),
        ("contract", DeclarationKind::Contract, false),
        ("open class", DeclarationKind::Class, true),
        ("open module", DeclarationKind::Module, true),
    ] {
        let given = format!("export @D()\n@Tools::Stamp(1)\n{nominal} Target {{}}");

        let when = parse_with_source(&given);

        assert!(
            when.parse.program_accepted,
            "{given}: {:?}",
            when.parse.diagnostics
        );
        assert_eq!(when.parse, parse(&given));
        assert_eq!(when.source.roots.len(), 1);
        let export = when.source.node(when.source.roots[0]);
        assert_eq!(export.kind, SourceKind::Export);
        assert_eq!(&given[export.span.start..export.span.end], given);
        assert_eq!(export.children.len(), 3);
        for (child, expected) in export.children[..2]
            .iter()
            .zip(["@D()", "@Tools::Stamp(1)"])
        {
            let node = when.source.node(*child);
            assert_eq!(node.kind, SourceKind::Decorator);
            assert_eq!(&given[node.span.start..node.span.end], expected);
        }
        let node = when.source.node(export.children[2]);
        let SourceKind::Declaration(declaration) = &node.kind else {
            panic!("expected nominal metadata: {node:?}");
        };
        assert_eq!(declaration.kind, kind);
        assert_eq!(declaration.name.text, "Target");
        assert_eq!(
            &given[declaration.name.span.start..declaration.name.span.end],
            "Target"
        );
        assert!(declaration.modifiers.exported);
        assert_eq!(declaration.modifiers.reopen, reopen);
        let header = declaration.header.as_ref().expect("nominal header");
        assert!(header.has_decorators);
        assert!(header.complete);
        assert_eq!(
            &given[node.span.start..node.span.end],
            format!("{nominal} Target {{}}")
        );
        assert_eq!(
            when.source
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, SourceKind::Declaration(_)))
                .count(),
            1
        );
        assert!(when.source.signatures.is_empty());
        assert!(when.source.recovery.is_empty());
    }
}

#[test]
fn preserves_existing_exports_when_decorators_are_absent() {
    for given in [
        "export class C {}",
        "export module M {}",
        "export contract K {}",
        "export open class C {}",
        "export open module M {}",
        "export import pkg::M as Local",
        "export from pkg::M import Name as Local, Other",
    ] {
        let reference = parse(given.strip_prefix("export ").expect("export prefix"));
        assert!(reference.program_accepted);

        let when = parse(given);

        assert!(when.program_accepted, "{given}: {:?}", when.diagnostics);
        assert_eq!(
            wrapped(&when.program.declarations[0]),
            &reference.program.declarations[0]
        );
    }
}

#[test]
fn preserves_name_lists_when_export_has_no_declaration() {
    let given = "export first, second,\n";

    let when = parse(given);

    assert!(when.program_accepted, "{:?}", when.diagnostics);
    assert_eq!(
        when.program.declarations,
        vec![Declaration::Export(Box::new(ExportDeclaration::Names(
            vec!["first".into(), "second".into()]
        )))]
    );
}

#[test]
fn rejects_decorators_when_their_position_or_target_is_outside_the_grammar() {
    for given in [
        "@D() export class C {}",
        "@D() export module M {}",
        "@D() export contract K {}",
        "export @D() name",
        "export @D() import pkg::M",
        "export @D() from pkg::M import Name",
        "export open @D() class C {}",
        "export open @D() module M {}",
        "export @D()",
    ] {
        let when = parse(given);

        assert!(!when.program_accepted, "{given}");
        assert!(!when.diagnostics.is_empty(), "{given}");
    }
}
