use iris_builtins::{Surface, members, service_names};
use std::collections::BTreeSet;

const INSTANCE_ROWS: &[(&str, &str)] = &[
    (
        "Object",
        "to_bool hash == != < <= > >= <=> to_string inspect same?",
    ),
    (
        "Integer",
        "+ - * / ** div mod << >> & | ^ ~ negate mul_add == != < <= > >= <=> hash to_bool to_string",
    ),
    (
        "Float32",
        "+ - * / ** negate mul_add is_nan is_signaling_nan is_infinite is_finite is_normal is_subnormal is_zero sign_bit to_bits == != < <= > >= <=> hash to_bool to_string",
    ),
    (
        "Float64",
        "+ - * / ** negate mul_add is_nan is_signaling_nan is_infinite is_finite is_normal is_subnormal is_zero sign_bit to_bits == != < <= > >= <=> hash to_bool to_string",
    ),
    ("Nil", "== != < <= > >= <=> hash to_bool to_string same?"),
    ("Bool", "== != < <= > >= <=> hash to_bool to_string same?"),
    ("Symbol", "== != hash to_string"),
    (
        "Array",
        "length count empty? to_array reverse sort uniq flatten map select reject each each_with_index find reduce sum min max first last pop push append delete insert clear join include? index_of concat slice take drop all? any? at to_string iterator same?",
    ),
    (
        "Hash",
        "length empty? keys values to_array include? has_key? fetch delete rehash each each_with_iterator map select merge iterator same?",
    ),
    ("Tuple", "to_array == != hash"),
    (
        "Range",
        "by start end inclusive_end? to_array iterator hash == !=",
    ),
    ("ReadonlyArray", "length empty? iterator"),
    (
        "String",
        "length byte_length empty? to_string inspect split trim downcase upcase replace starts_with? ends_with? contains? chars to_array to_symbol to_bytes bytes + casefold nfc nfd graphemes =~ !~ == != hash to_bool",
    ),
    (
        "MutableString",
        "length to_string to_bytes to_array upcase downcase upcase! downcase! clear append replace << + bytes casefold nfc nfd graphemes =~ !~ iterator == != same?",
    ),
    (
        "Bytes",
        "length empty? to_string to_bytes to_array + iterator == != hash",
    ),
    (
        "ByteArray",
        "length empty? to_string to_bytes to_array + append << iterator == != same?",
    ),
    (
        "Match",
        "text to_string regex start end byte_start byte_end capture",
    ),
    ("Regex", "== != hash"),
    ("ArrayIterator", "next close == != hash same?"),
    ("HashIterator", "next close remove_current == != hash same?"),
    ("ByteIterator", "next close == != hash same?"),
    ("Generator", "iterator next close == != hash same?"),
    ("Iteration", "yield? done? value <=> == != hash same?"),
    ("Closure", "call class_name == != hash same?"),
    ("BoundMethod", "call class_name == != same?"),
    (
        "Method",
        "bind selector owner visibility parameters return_type source class_name == != same?",
    ),
    ("Task", "class_name == != hash same?"),
    ("Gate", "== != hash same?"),
    (
        "ExceptionContext",
        "value cause suppressed re_raise_sites original_stack raise_location class_name decoder offset expected operation caller_package target_scope denial_origin == != hash same?",
    ),
    ("SourceLocation", "path line column"),
    ("StackFrame", "callable_name location"),
    ("RaiseSite", "location"),
    ("ExternalResource", "close closed? == same?"),
    ("Class", "== != same?"),
    ("Module", "same?"),
    (
        "Type",
        "type kind package hash arguments members subtype? assignable? same?",
    ),
    ("ComposedType", "type kind members same?"),
    ("Contract", "parents view hash == != same?"),
    ("ContractView", "respond_to_contract? hash"),
    ("Transformation", "empty kind add_method"),
    (
        "FFI::Library",
        "bind signature bound? class_name == != same?",
    ),
];

#[test]
fn every_instance_family_matches_independent_inventory_when_loaded() {
    for (owner, selectors) in INSTANCE_ROWS {
        let actual: BTreeSet<_> = members()
            .iter()
            .filter(|row| row.owner == *owner && row.surface == Surface::Instance)
            .map(|row| row.selector)
            .collect();
        let mut expected: BTreeSet<_> = selectors.split_whitespace().collect();
        if *owner != "Module" {
            expected.insert("respond_to?");
        }
        if !matches!(
            *owner,
            "Class" | "Module" | "ContractView" | "ExternalResource"
        ) {
            expected.insert("to_bool");
        }
        assert_eq!(actual, expected, "{owner}");
    }
    let expected: BTreeSet<_> = INSTANCE_ROWS.iter().map(|(owner, _)| *owner).collect();
    let actual: BTreeSet<_> = members()
        .iter()
        .filter(|row| row.surface == Surface::Instance)
        .map(|row| row.owner)
        .collect();
    assert_eq!(actual, expected);
}

const SERVICE_ROWS: &[(&str, &str)] = &[
    ("Iteration", "yield"),
    ("Unicode", "version"),
    ("Encoding::UTF_8", "decode"),
    ("Encoding::UTF_16LE", "decode"),
    ("Encoding::UTF_16BE", "decode"),
    ("Encoding::Latin_1", "decode"),
    ("JSON", "encode decode"),
    ("IrisValue", "encode decode"),
    ("FFI", "open"),
    ("Host", "run"),
    ("Gate", "new complete"),
    ("Diagnostics", "discarded_contexts unobserved_failures"),
    ("Revision", "subscribe flush shutdown event_errors"),
    ("RevisionHistory", "events recover configure_sink"),
    (
        "Reflection::Object",
        "list_ivars get_ivar set_ivar remove_ivar",
    ),
    (
        "Reflection::Class",
        "method invoke remove_module remove_contract set_superclass properties revision ancestors define_method define_property",
    ),
    ("Reflection::Module", "method invoke"),
    ("Reflection::Contract", "requirement"),
    (
        "Reflection::Package",
        "identity version dependencies module_status reload load upgrade",
    ),
    ("Package", "validate"),
];

#[test]
fn services_match_independent_inventory_when_names_are_resolved() {
    for (owner, selectors) in SERVICE_ROWS {
        let actual: BTreeSet<_> = members()
            .iter()
            .filter(|row| row.owner == *owner && row.surface == Surface::Service)
            .map(|row| row.selector)
            .collect();
        assert_eq!(actual, selectors.split_whitespace().collect(), "{owner}");
    }
    assert_eq!(
        service_names().iter().copied().collect::<BTreeSet<_>>(),
        SERVICE_ROWS.iter().map(|(name, _)| *name).collect()
    );
}

#[test]
fn property_and_class_surfaces_match_when_calls_differ_from_reads() {
    for (owner, surface, selectors) in [
        ("Float32", Surface::Class, "from_bits nan infinity"),
        ("Float64", Surface::Class, "from_bits nan infinity"),
        ("Object", Surface::Class, "new"),
        (
            "Class",
            Surface::Class,
            "new open define_method define_property alias_method remove_method undef_method add_module remove_module remove_contract set_superclass method invoke properties active_revision contracts name methods modules static_spine denied_capabilities decorators decorator_arguments decorator_phases type package runtime_superclass mro ancestors meta_capabilities",
        ),
        ("Module", Surface::Class, "invoke method"),
        (
            "Class",
            Surface::Property,
            "properties active_revision contracts name methods modules static_spine denied_capabilities decorators decorator_arguments decorator_phases type package runtime_superclass mro ancestors meta_capabilities",
        ),
        ("Module", Surface::Property, "modules"),
        ("Float32", Surface::Property, "nan infinity"),
        ("Float64", Surface::Property, "nan infinity"),
        ("Range", Surface::Property, "start end inclusive_end?"),
        ("String", Surface::Property, "bytes"),
        ("MutableString", Surface::Property, "bytes"),
        (
            "Match",
            Surface::Property,
            "text to_string regex start end byte_start byte_end",
        ),
        ("Iteration", Surface::Property, "done yield? done? value"),
        (
            "Method",
            Surface::Property,
            "selector owner visibility parameters return_type source class_name",
        ),
        ("Closure", Surface::Property, "class_name"),
        ("BoundMethod", Surface::Property, "class_name"),
        ("Task", Surface::Property, "class_name"),
        (
            "ExceptionContext",
            Surface::Property,
            "value cause suppressed re_raise_sites original_stack raise_location class_name decoder offset expected operation caller_package target_scope denial_origin",
        ),
        ("SourceLocation", Surface::Property, "path line column"),
        ("StackFrame", Surface::Property, "callable_name location"),
        ("RaiseSite", Surface::Property, "location"),
        (
            "Type",
            Surface::Property,
            "type kind package hash arguments members",
        ),
        ("ComposedType", Surface::Property, "type kind members"),
        ("Contract", Surface::Property, "parents"),
        ("Transformation", Surface::Property, "empty kind"),
        ("", Surface::Global, "print using Integer Float64"),
    ] {
        let actual: BTreeSet<_> = members()
            .iter()
            .filter(|row| row.owner == owner && row.surface == surface)
            .map(|row| row.selector)
            .collect();
        assert_eq!(
            actual,
            selectors.split_whitespace().collect(),
            "{owner} {surface:?}"
        );
    }
}
