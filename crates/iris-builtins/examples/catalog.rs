use iris_builtins::{BuiltinType, Surface, class_names, members, service_names};

fn main() {
    println!(
        "{} descriptors; {} class names; {} service names",
        members().len(),
        class_names().len(),
        service_names().len()
    );
    for row in members().iter().filter(|row| {
        row.receiver == Some(BuiltinType::Array)
            && row.surface == Surface::Instance
            && matches!(row.selector, "count" | "reduce" | "append" | "push")
    }) {
        println!(
            "{}.{}: {:?}; shapes {:?}",
            row.owner,
            row.selector,
            row.result,
            row.shapes
                .iter()
                .map(|shape| shape.parameters.len())
                .collect::<Vec<_>>()
        );
    }
    println!("Text family lookup: {:?}", BuiltinType::from_name("Text"));
}
