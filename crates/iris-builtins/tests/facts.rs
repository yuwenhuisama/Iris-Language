use iris_builtins::{Availability, BuiltinType, ParameterKind, ReturnFact, Surface, members};

#[test]
fn backend_facts_remain_conservative_when_results_or_syntax_diverge() {
    for row in members() {
        match (row.owner, row.surface, row.selector) {
            ("Hash", Surface::Instance, "each" | "each_with_iterator")
            | ("SourceLocation", Surface::Property, "path")
            | ("Integer", Surface::Instance, "mul_add" | "/" | "**") => {
                assert_eq!(row.result, ReturnFact::Unknown)
            }
            ("Array", Surface::Instance, "append") => {
                assert_eq!(row.result, ReturnFact::Known(BuiltinType::Nil))
            }
            ("Array", Surface::Instance, "push") => assert_eq!(row.result, ReturnFact::Receiver),
            ("MutableString", Surface::Instance, "casefold" | "nfc" | "nfd") => {
                assert_eq!(row.result, ReturnFact::Known(BuiltinType::String))
            }
            ("ExceptionContext", Surface::Instance, "decoder" | "offset" | "expected")
            | ("Class", Surface::Class, "name" | "methods" | "type")
            | ("MutableString", Surface::Property, "bytes") => {
                assert_eq!(row.shapes[0].availability, Availability::Reference)
            }
            ("ExceptionContext", Surface::Property, "decoder" | "offset" | "expected")
            | ("Class", Surface::Property, "name" | "methods" | "type") => {
                assert_eq!(row.shapes[0].availability, Availability::Both)
            }
            _ => {}
        }
    }
}

#[test]
fn keyword_alternatives_are_explicit_when_options_are_supplied() {
    for (owner, selector, keyword) in [
        ("JSON", "encode", "canonical"),
        ("JSON", "decode", "depth"),
        ("IrisValue", "decode", "element_limit"),
        ("FFI", "open", "declarations"),
        ("Encoding::UTF_8", "decode", "errors"),
    ] {
        let rows: Vec<_> = members()
            .iter()
            .filter(|row| row.owner == owner && row.selector == selector)
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].shapes[0].parameters.len(), 1);
        let option = &rows[0].shapes[1].parameters[1];
        assert_eq!(
            (option.label, option.kind, option.optional),
            (keyword, ParameterKind::Keyword, false)
        );
    }
}
