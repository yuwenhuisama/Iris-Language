use iris_parser::{parse, prepare_bindings};
use iris_syntax::{Expression, ProgramEntry, Statement, TypeExpression, render_parse_shape};

#[test]
fn source_modes_preserve_order_when_block_channels_are_written() {
    let given = "target.run(first, &callback, label: value, &nil) { ||; 9 }";

    let when = parse(given);
    let sourced = iris_parser::parse_with_source(given);
    let editor = iris_parser::parse_editor(given);

    assert!(when.program_accepted, "{:?}", when.diagnostics);
    assert_eq!(when.program, sourced.parse.program);
    assert_eq!(when.program, editor.parse.program);
    let Statement::Expression(Expression::Call { arguments, .. }) = &when.program.statements[0]
    else {
        panic!("call required")
    };
    assert_eq!(arguments.len(), 5);
    assert_eq!(arguments[0], Expression::Name("first".into()));
    assert_eq!(
        arguments[1],
        Expression::BlockArgument {
            value: Box::new(Expression::Name("callback".into()))
        }
    );
    assert!(matches!(&arguments[2], Expression::KeywordArgument { name, .. } if name == "label"));
    assert_eq!(
        arguments[3],
        Expression::BlockArgument {
            value: Box::new(Expression::Name("nil".into()))
        }
    );
    assert!(
        matches!(&arguments[4], Expression::BlockArgument { value } if matches!(value.as_ref(), Expression::Closure { .. }))
    );
}

#[test]
fn trailing_closure_is_canonical_when_source_metadata_is_absent() {
    let given = "target.run() { ||; 9 }";

    let when = parse(given);

    assert!(when.program_accepted);
    assert!(
        matches!(&when.program.statements[0], Statement::Expression(Expression::Call { arguments, .. })
        if matches!(&arguments[0], Expression::BlockArgument { value } if matches!(value.as_ref(), Expression::Closure { .. })))
    );
}

#[test]
fn block_metadata_links_operand_when_ampersand_is_written() {
    use iris_parser::source::{ArgumentKind, ExpressionFact, SourceKind};
    let given = "target.run(&callback, &nil) { ||; 9 }";

    let when = iris_parser::parse_with_source(given);

    assert!(when.parse.program_accepted);
    let call = when.source.calls.last().expect("call site");
    assert_eq!(call.arguments[0].kind, ArgumentKind::Block);
    assert_eq!(call.arguments[1].kind, ArgumentKind::Block);
    assert_eq!(call.arguments[2].kind, ArgumentKind::TrailingBlock);
    let slot = &call.arguments[0];
    let node = when.source.node(slot.expression.expect("argument node"));
    assert_eq!(&given[node.span.start..node.span.end], "&callback");
    assert_eq!(node.span, slot.span);
    let SourceKind::Expression(ExpressionFact::BlockArgument { value }) = node.kind else {
        panic!("block source fact required")
    };
    assert_eq!(node.children, vec![value]);
    let operand = when.source.node(value);
    assert_eq!(&given[operand.span.start..operand.span.end], "callback");
}

#[test]
fn block_slots_recover_when_operand_is_incomplete() {
    use iris_parser::source::ArgumentKind;
    for given in ["target.run(&", "target.run(&value + )"] {
        let when = iris_parser::parse_editor(given);

        assert!(!when.parse.program_accepted);
        let call = when.source.calls.last().expect("retained call");
        assert_eq!(call.arguments[0].kind, ArgumentKind::Block);
        assert!(call.arguments[0].incomplete);
        assert_eq!(call.arguments[0].expression, None);
        assert_eq!(iris_parser::parse_with_source(given).parse, parse(given));
    }
}

#[test]
fn block_argument_clone_preserves_structural_shape() {
    let given = Expression::BlockArgument {
        value: Box::new(Expression::Name("callback".into())),
    };

    let when = given.clone();

    assert_eq!(when, given);
    assert_eq!(render_parse_shape(&when), "block_argument(callback)");
}

#[test]
fn block_argument_preparation_visits_binding_and_alias_types() {
    let parsed = parse("type Number = Integer; { ||; let value: Number = 1; let inferred = 2 }");
    assert!(parsed.program_accepted, "{parsed:?}");
    let mut given = parsed.program;
    let Some(ProgramEntry::Statement(Statement::Expression(expression))) = given.entries.last_mut()
    else {
        unreachable!("closure fixture required")
    };
    *expression = Expression::BlockArgument {
        value: Box::new(expression.clone()),
    };

    let when = prepare_bindings(&given).expect("valid binding types");

    let Some(ProgramEntry::Statement(Statement::Expression(Expression::BlockArgument { value }))) =
        when.entries.last()
    else {
        unreachable!("preparation must preserve transport")
    };
    let Expression::Closure { body, .. } = value.as_ref() else {
        unreachable!("closure operand must survive preparation")
    };
    for statement in body {
        let Statement::Binding { annotation, .. } = statement else {
            unreachable!("binding fixture required")
        };
        assert_eq!(annotation, &Some(TypeExpression::Name("Integer".into())));
    }
}
