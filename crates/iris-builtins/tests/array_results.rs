use iris_builtins::{BuiltinType, ReturnFact, members};

#[test]
fn retains_element_family_when_text_produces_string_arrays() -> Result<(), &'static str> {
    for (owner, selector) in [
        ("String", "split"),
        ("String", "chars"),
        ("String", "to_array"),
        ("String", "graphemes"),
        ("MutableString", "to_array"),
        ("MutableString", "graphemes"),
    ] {
        let given = members();

        let when = given
            .iter()
            .find(|row| row.owner == owner && row.selector == selector)
            .ok_or("missing text array producer")?;

        assert_eq!(when.result, ReturnFact::ArrayOf(BuiltinType::String));
        assert_eq!(when.return_label, Some("Array<String>"));
    }
    Ok(())
}
