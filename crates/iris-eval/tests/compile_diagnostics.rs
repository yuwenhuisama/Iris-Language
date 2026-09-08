use iris_eval::backend::{
    Agreement, Backend, Bytecode, Interpreter, Observation, Support, compare_backends,
};

#[test]
fn bytecode_reports_type_error_when_fixed_local_changes_type() {
    let given = "mut port = 8080; port = \"9090\"; port";

    let when = Bytecode.execute(given);

    assert_eq!(
        when,
        Support::Ran(Observation::Error("TypeContractError".to_owned()))
    );
}

#[test]
fn bytecode_reports_type_error_when_uncalled_method_has_invalid_assignment() {
    let given = "class Uncalled { public fun check() { mut port: Integer = 8080; port = \"9090\"; port } }; nil";

    let when = Bytecode.execute(given);

    assert_eq!(
        when,
        Support::Ran(Observation::Error("TypeContractError".to_owned()))
    );
}

#[test]
fn backends_agree_when_fixed_local_assignment_is_statically_rejected() {
    let given = "mut port: Integer | String = 8080; port = true; port";

    let when = compare_backends(given, &[&Interpreter, &Bytecode]);

    let Agreement::Agreed {
        backends,
        observation,
    } = when
    else {
        unreachable!("static rejection must be an ordinary agreement: {when:?}")
    };
    assert_eq!(backends, ["interpreter", "bytecode"]);
    assert_eq!(
        observation,
        Observation::Error("TypeContractError".to_owned())
    );
}

#[test]
fn bytecode_preserves_refusal_when_construct_is_uncovered() {
    let given = "NativeFixture.unknown_thing()";

    let when = Bytecode.execute(given);

    assert_eq!(
        when,
        Support::Unsupported("call unbound receiver".to_owned())
    );
}

#[test]
fn bytecode_preserves_parse_error_when_source_is_invalid() {
    let given = "let =";

    let when = Bytecode.execute(given);

    assert_eq!(
        when,
        Support::Ran(Observation::Error("ParseDiagnostic".to_owned()))
    );
}

#[test]
fn bytecode_runs_when_explicit_union_permits_assignment() {
    let given = "module Main { public module fun run() { mut port: Integer | String = 8080; port = \"9090\"; port } }; Main.run()";

    let when = Bytecode.execute(given);

    assert_eq!(
        when,
        Support::Ran(Observation::Value("\"9090\"".to_owned()))
    );
}

#[test]
fn bytecode_preserves_parse_error_when_generic_arguments_are_invariant() {
    let given = "class Box<T> {} let target: Box<Object> = Box<String>.new()";

    let when = Bytecode.execute(given);

    assert_eq!(
        when,
        Support::Ran(Observation::Error("ParseDiagnostic".to_owned()))
    );
}
