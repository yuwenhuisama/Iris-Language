use iris_parser::{parse_editor, parse_with_source};

fn main() {
    let text = "/// Converts an input.\nfun convert(_: Integer = seed(), key format: String = 'text') -> String { format }";
    let parsed = parse_with_source(text);
    assert!(parsed.parse.program_accepted);
    assert_eq!(parsed.source.documentation.len(), 1);
    assert_eq!(parsed.source.parameter_slots.len(), 2);
    assert_eq!(parsed.source.parameter_slots[0].declaration, None);
    println!("documentation: {}", parsed.source.documentation[0].text);
    for slot in &parsed.source.parameter_slots {
        println!(
            "parameter {} {:?}: {}",
            slot.name.text,
            slot.category,
            &text[slot.span.start..slot.span.end]
        );
    }
    let incomplete = parse_editor("convert(nested(1), format: unfinished + )");
    assert!(!incomplete.parse.program_accepted);
    assert_eq!(incomplete.source.calls.len(), 2);
    for call in &incomplete.source.calls {
        println!(
            "call {:?}: open={:?} close={:?} commas={} slots={} incomplete={}",
            call.call,
            call.open,
            call.close,
            call.commas.len(),
            call.arguments.len(),
            call.incomplete
        );
    }
    assert!(incomplete.source.calls[1].arguments[1].incomplete);
    assert_eq!(incomplete.source.calls[1].arguments[1].expression, None);
}
