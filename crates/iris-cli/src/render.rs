//! Renders a runtime value for a human reader.
//!
//! This is DISPLAY, not serialization: it exists so a person running a script
//! or a REPL line can see what came back. It deliberately does not promise a
//! round-trippable form, and it never dispatches into Iris code, so rendering a
//! value can neither run user code nor fail.

use iris_runtime::Value;

/// Renders `value` for display.
#[must_use]
pub fn render(value: &Value) -> String {
    match value {
        Value::Nil => "nil".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::Integer(number) => number.decimal_text(),
        Value::Float32(number) => number.to_string(),
        Value::Float64(number) => number.to_string(),
        // A String renders with quotes so an empty String is visible and so a
        // String is distinguishable from a Symbol or a bare identifier.
        Value::Text(text) => format!("{text:?}"),
        Value::MutableString(text) => format!("m{:?}", text.text()),
        Value::Symbol(name) => format!(":{name}"),
        Value::Array(values) => {
            let rendered: Vec<String> = values.elements().iter().map(render).collect();
            format!("[{}]", rendered.join(", "))
        }
        Value::ReadonlyArray(values) => {
            let rendered: Vec<String> = values.iter().map(render).collect();
            format!("[{}]", rendered.join(", "))
        }
        Value::Tuple(values) => {
            let rendered: Vec<String> = values.iter().map(render).collect();
            format!("({})", rendered.join(", "))
        }
        Value::Hash(entries) => {
            let rendered: Vec<String> = entries
                .entries()
                .iter()
                .map(|(key, entry)| format!("{}: {}", render(key), render(entry)))
                .collect();
            format!("{{{}}}", rendered.join(", "))
        }
        Value::Bytes(bytes) => format!("bytes[{}]", bytes.len()),
        Value::ByteArray(bytes) => format!("byte_array[{}]", bytes.bytes().len()),
        Value::IterationDone => "Iteration.done".to_owned(),
        Value::IterationYield(payload) => format!("Iteration.yield({})", render(payload)),
        // The remaining kinds have no meaningful literal form. Naming the KIND
        // is more useful to a reader than a debug dump of runtime identity, and
        // it keeps output stable across runs.
        Value::Object(_) => "<object>".to_owned(),
        Value::Class(_) | Value::ClosedClass(..) => "<class>".to_owned(),
        Value::Closure(_) => "<closure>".to_owned(),
        Value::Method(_) | Value::BoundMethod(_) => "<method>".to_owned(),
        Value::Contract(..) | Value::ContractView(..) => "<contract>".to_owned(),
        Value::Type(..) | Value::ComposedType(_) => "<type>".to_owned(),
        Value::Range(_) => "<range>".to_owned(),
        Value::Regex(_) => "<regex>".to_owned(),
        Value::Match(_) => "<match>".to_owned(),
        Value::Task(_) => "<task>".to_owned(),
        Value::Generator(_) => "<generator>".to_owned(),
        Value::Gate(_) => "<gate>".to_owned(),
        Value::Library(_) => "<library>".to_owned(),
        Value::ArrayIterator(_) | Value::HashIterator(_) | Value::ByteIterator(_) => {
            "<iterator>".to_owned()
        }
        Value::NativeResource(_) | Value::ExternalResource(_) => "<native-resource>".to_owned(),
        Value::Transformation { .. } => "<transformation>".to_owned(),
        Value::ExceptionContext(..) => "<exception-context>".to_owned(),
        Value::StackFrame(..) | Value::RaiseSite(_) | Value::SourceLocation(..) => {
            "<trace>".to_owned()
        }
        Value::KeywordArgument(name, inner) => format!("{name}: {}", render(inner)),
    }
}
