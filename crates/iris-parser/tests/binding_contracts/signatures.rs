use super::method;
use iris_syntax::TypeExpression;

#[test]
fn compares_declaration_variance_when_replacing_method() {
    for (implementation, promise, expected) in [
        ("(x: Object) -> Integer", "(x: Integer) -> Object", true),
        ("(x: Integer) -> String", "(x: Integer) -> Integer", false),
        ("(x: Integer) -> Object", "(x: Object) -> Object", false),
        ("(x) -> Integer", "(x: Integer) -> Integer", true),
        ("(x: Integer)", "(x: Integer) -> Integer", false),
        ("(x: Dog) -> Dog", "(x: Dog) -> Animal", true),
        (
            "(x: Closure<(Object) -> Integer>) -> Integer",
            "(x: Closure<(Integer) -> Object>) -> Integer",
            false,
        ),
    ] {
        let given = (method(implementation), method(promise));
        let when =
            iris_syntax::method_signature_compatible(&given.0, &given.1, |source, target| {
                source == "Dog" && target == "Animal"
            });
        assert_eq!(when, expected, "{implementation} replacing {promise}");
    }
}

#[test]
fn preserves_call_shapes_when_replacing_method() {
    for (implementation, promise, expected) in [
        (
            "(x: Integer = 1) -> Integer",
            "(x: Integer) -> Integer",
            true,
        ),
        (
            "(x: Integer) -> Integer",
            "(x: Integer = 1) -> Integer",
            false,
        ),
        (
            "(x: Integer, y: Integer = 1) -> Integer",
            "(x: Integer) -> Integer",
            true,
        ),
        (
            "(*values: Object) -> Integer",
            "(x: Integer) -> Integer",
            true,
        ),
        (
            "(x: Integer) -> Integer",
            "(*values: Integer) -> Integer",
            false,
        ),
        (
            "(key name: Object = nil) -> Integer",
            "(key name: String) -> Integer",
            true,
        ),
        (
            "(key renamed: String) -> Integer",
            "(key name: String) -> Integer",
            false,
        ),
        (
            "(**values: Object) -> Integer",
            "(key name: String) -> Integer",
            true,
        ),
        (
            "(key name: String = \"x\") -> Integer",
            "(**values: String) -> Integer",
            false,
        ),
        (
            "(&callback: Block<(Integer) -> Object>) -> Integer",
            "(&callback: Block<(Object) -> Integer>) -> Object",
            false,
        ),
        (
            "() -> Integer",
            "(&callback: Block<() -> Integer>) -> Integer",
            false,
        ),
        (
            "(&callback: Block<() -> Integer>) -> Integer",
            "() -> Integer",
            true,
        ),
    ] {
        let given = (method(implementation), method(promise));
        let when = iris_syntax::method_signature_compatible(&given.0, &given.1, |_, _| false);
        assert_eq!(when, expected, "{implementation} replacing {promise}");
    }
}

#[test]
fn compares_recursive_function_types_when_ast_contains_function_position() {
    let function = |parameter: &str, result: &str| TypeExpression::Function {
        parameters: vec![TypeExpression::Name(parameter.into())],
        result: Box::new(TypeExpression::Name(result.into())),
    };
    let mut given = (method("(value) -> Integer"), method("(value) -> Object"));
    given.0.parameters[0].annotation = Some(function("Integer", "Object"));
    given.1.parameters[0].annotation = Some(function("Object", "Integer"));
    let when = iris_syntax::method_signature_compatible(&given.0, &given.1, |_, _| false);
    assert!(when);
}
