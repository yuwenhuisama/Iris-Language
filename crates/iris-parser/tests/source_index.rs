use iris_parser::source::{ExpressionFact, SourceKind};
use iris_parser::{parse, parse_with_source};

#[test]
fn records_actual_children_when_indexing_nested_expressions() {
    let text = "let a=\"ffff\".split(\"\");let b=a[0]; a[indices[1]][0 ..< 2]";
    let given = parse(text);

    let when = parse_with_source(text);

    assert!(given.program_accepted, "{:?}", given.diagnostics);
    assert_eq!(given.program, when.parse.program);
    let indexes: Vec<_> = when
        .source
        .nodes
        .iter()
        .filter_map(|node| {
            let SourceKind::Expression(ExpressionFact::Index { receiver, index }) = node.kind
            else {
                return None;
            };
            assert_eq!(node.children, [receiver, index]);
            let receiver = when.source.node(receiver).span;
            let index = when.source.node(index).span;
            Some((
                &text[receiver.start..receiver.end],
                &text[index.start..index.end],
            ))
        })
        .collect();
    assert_eq!(
        indexes,
        [
            ("a", "0"),
            ("indices", "1"),
            ("a", "indices[1]"),
            ("a[indices[1]]", "0 ..< 2")
        ]
    );
}
