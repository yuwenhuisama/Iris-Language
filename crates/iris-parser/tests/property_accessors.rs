use iris_parser::{parse, parse_with_source, prepare_bindings};
use iris_syntax::{Declaration, Expression, Program, ProgramEntry, Statement, Visibility};

fn property(program: &Program) -> &Statement {
    let Declaration::Class(class) = &program.declarations[0] else {
        panic!("expected Class");
    };
    &class.body[0]
}

#[test]
fn retains_accessor_presence_and_visibility_when_explicit() {
    // Given
    let cases = [
        ("get;", vec![("get", Visibility::Private)]),
        ("set;", vec![("set", Visibility::Private)]),
        (
            "public get; private set;",
            vec![("get", Visibility::Public), ("set", Visibility::Private)],
        ),
        (
            "protected set; public get;",
            vec![("set", Visibility::Protected), ("get", Visibility::Public)],
        ),
        (
            "get; get;",
            vec![("get", Visibility::Private), ("get", Visibility::Private)],
        ),
        ("", vec![]),
    ];
    for (members, expected) in cases {
        // When
        let parsed = parse(&format!(
            "class C {{ public property x: Integer = 7 {{ {members} }} }}"
        ));
        // Then
        assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
        let Statement::StoredProperty {
            visibility,
            accessors,
            ..
        } = property(&parsed.program)
        else {
            panic!("expected stored property");
        };
        assert_eq!(*visibility, Visibility::Public);
        let actual = accessors
            .as_ref()
            .expect("explicit block")
            .members
            .iter()
            .map(|accessor| {
                let kind = match accessor.kind {
                    iris_syntax::PropertyAccessorKind::Get => "get",
                    iris_syntax::PropertyAccessorKind::Set => "set",
                };
                (kind, accessor.visibility)
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
}

#[test]
fn preserves_omitted_block_when_shorthand_is_written() {
    // Given
    let source = "class C { property x: Integer = 7\npublic fun next() { 8 } }";
    // When
    let parsed = parse(source);
    // Then
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    assert!(matches!(
        property(&parsed.program),
        Statement::StoredProperty {
            visibility: Visibility::Private,
            accessors: None,
            ..
        }
    ));
    let Declaration::Class(class) = &parsed.program.declarations[0] else {
        panic!("expected Class")
    };
    assert_eq!(class.body.len(), 2);
}

#[test]
fn retains_initializer_when_accessor_block_follows_call() {
    // Given
    let source = "class C { public property x: Integer = make<Integer>(7) { get; private set; }\nfun next() { 8 } }";
    // When
    let parsed = parse(source);
    // Then
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    let Statement::StoredProperty {
        initializer: Expression::Call { arguments, .. },
        accessors,
        ..
    } = property(&parsed.program)
    else {
        panic!("expected call initializer");
    };
    assert_eq!(arguments, &[Expression::Literal("7".into())]);
    assert_eq!(accessors.as_ref().expect("explicit block").members.len(), 2);
    let Declaration::Class(class) = &parsed.program.declarations[0] else {
        panic!("expected Class")
    };
    assert_eq!(class.body.len(), 2);
}

#[test]
fn retains_trailing_closure_when_initializer_has_real_block() {
    // Given
    let source = "class C { property x: Integer = make() { ||; 7 } { get; } }";
    // When
    let parsed = parse(source);
    // Then
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    let Statement::StoredProperty {
        initializer: Expression::Call { arguments, .. },
        accessors,
        ..
    } = property(&parsed.program)
    else {
        panic!("expected call initializer");
    };
    assert!(matches!(
        arguments.as_slice(),
        [Expression::BlockArgument { value }]
            if matches!(value.as_ref(), Expression::Closure { has_header: true, .. })
    ));
    assert_eq!(accessors.as_ref().expect("explicit block").members.len(), 1);
}

#[test]
fn preserves_metadata_when_program_is_normalized_and_cloned() {
    // Given
    let parsed = parse("class C { protected property x: Integer = 7 { public get; } }");
    // When
    let prepared = prepare_bindings(&parsed.program).expect("valid preparation");
    // Then
    assert!(parsed.program_accepted);
    assert_eq!(property(&prepared), property(&parsed.program));
    let ProgramEntry::Declaration(Declaration::Class(class)) = &prepared.entries[0] else {
        panic!("expected Class entry")
    };
    assert_eq!(&class.body[0], property(&parsed.program));
}

#[test]
fn preserves_source_initializer_when_accessors_are_consumed() {
    // Given
    let source =
        "class C { public property x: Integer = make(7) { get; private set; }\nfun next() { 8 } }";
    // When
    let parsed = parse_with_source(source);
    // Then
    assert!(
        parsed.parse.program_accepted,
        "{:?}",
        parsed.parse.diagnostics
    );
    let declaration = parsed
        .source
        .nodes
        .iter()
        .find_map(|node| match &node.kind {
            iris_parser::source::SourceKind::Declaration(declaration)
                if declaration.kind == iris_parser::source::DeclarationKind::Property =>
            {
                Some(declaration)
            }
            _ => None,
        })
        .expect("property source fact");
    let initializer = parsed
        .source
        .node(declaration.initializer.expect("initializer source fact"));
    assert_eq!(
        &source[initializer.span.start..initializer.span.end],
        "make(7)"
    );
    assert_eq!(declaration.visibility, Visibility::Public);
    assert_eq!(parsed.source.calls.len(), 1);
    assert_eq!(parsed.source.calls[0].arguments.len(), 1);
}

#[test]
fn rejects_accessor_member_when_semicolon_is_missing() {
    // Given
    let source = "class C { property x: Integer = make() { get\nset; } }";
    // When
    let parsed = parse(source);
    // Then
    assert!(!parsed.program_accepted);
}

#[test]
fn retains_accessors_when_initializer_is_omitted_and_layout_changes() {
    // Given
    let sources = [
        "class C { protected shared class property x: Integer { public get; }\nfun next() {} }",
        "class C {\nprotected shared class property x: Integer\n{\npublic get;\n}\nfun next() {}\n}",
    ];
    for source in sources {
        // When
        let parsed = parse(source);
        // Then
        assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
        let Statement::StoredProperty {
            visibility,
            shared,
            class_level,
            initializer,
            accessors,
            ..
        } = property(&parsed.program)
        else {
            panic!("expected stored property")
        };
        assert_eq!(
            (*visibility, *shared, *class_level),
            (Visibility::Protected, true, true)
        );
        assert_eq!(initializer, &Expression::Literal("nil".into()));
        assert_eq!(accessors.as_ref().expect("explicit block").members.len(), 1);
        let Declaration::Class(class) = &parsed.program.declarations[0] else {
            panic!("expected Class")
        };
        assert_eq!(class.body.len(), 2);
    }
}

#[test]
fn retains_nested_call_block_when_property_initializer_is_delimited() {
    // Given
    let source = "class C { property x: Integer = outer(inner() {}) { set; } }";
    // When
    let parsed = parse(source);
    // Then
    assert!(parsed.program_accepted, "{:?}", parsed.diagnostics);
    let Statement::StoredProperty {
        initializer: Expression::Call { arguments, .. },
        accessors,
        ..
    } = property(&parsed.program)
    else {
        panic!("expected call initializer")
    };
    let Expression::Call { arguments, .. } = &arguments[0] else {
        panic!("expected nested call")
    };
    assert!(
        matches!(arguments.as_slice(), [Expression::BlockArgument { value }]
        if matches!(value.as_ref(), Expression::Closure { .. }))
    );
    assert_eq!(
        accessors.as_ref().expect("explicit block").members[0].kind,
        iris_syntax::PropertyAccessorKind::Set
    );
}
