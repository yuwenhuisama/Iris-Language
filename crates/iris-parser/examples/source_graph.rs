use iris_parser::source::{ExpressionFact, SourceKind};
use iris_parser::{parse_editor, parse_with_source};

fn main() {
    let headers = parse_with_source(
        "import org.dep::Core; class Box extends Base { fun get(value = call(1)) { value } }",
    );
    assert!(headers.parse.program_accepted);
    for node in &headers.source.nodes {
        match &node.kind {
            SourceKind::Declaration(value) => {
                if let Some(header) = &value.header {
                    println!(
                        "header {} complete={} extends={}",
                        value.name.text, header.complete, header.has_extends
                    );
                }
                if let Some(offset) = value.return_hint_offset {
                    println!("return hint {} at byte {}", value.name.text, offset);
                }
            }
            SourceKind::Import(value) => println!("import separators {:?}", value.separators),
            _ => {}
        }
    }
    let result = parse_with_source("module Main { let value: Integer = 1; value.to_string() }");
    assert!(result.parse.program_accepted);
    for node in &result.source.nodes {
        match &node.kind {
            SourceKind::Declaration(value) => println!(
                "{:?} {} {:?} scope={:?} visible={}",
                value.kind, value.name.text, value.name.span, node.scope, value.visible_from
            ),
            SourceKind::Expression(ExpressionFact::Member { receiver, name, .. }) => {
                println!("member {} receiver={:?}", name.text, receiver)
            }
            _ => {}
        }
    }
    let partial = parse_editor("module Main { let value = 1; value.");
    assert!(!partial.parse.program_accepted);
    assert!(partial.source.nodes.iter().any(|node| matches!(
        node.kind,
        SourceKind::Expression(ExpressionFact::IncompleteMember { .. })
    )));
    println!(
        "partial source: {} recovery barriers",
        partial.source.recovery.len()
    );
}
