use iris_native_host::{NativeRegistry, PackageSource};
use iris_runtime::Value;
use std::rc::Rc;

#[test]
fn package_completes_when_startup_runs_inside_module() -> Result<(), String> {
    let registry = Rc::new(NativeRegistry::new());
    let sources = [PackageSource {
        package_id: "app".to_owned(),
        api_major: 1,
        version: "1.0.0".to_owned(),
        path: "main.ir".to_owned(),
        source: "module Main { public module fun run() -> Nil { nil } Main.run() }".to_owned(),
        allowed_imports: Default::default(),
    }];

    let program = iris_vm::compile_package_tree_with_natives(&sources, &registry)
        .map_err(|error| error.construct)?;
    iris_vm::verify(&program).map_err(|error| format!("{error:?}"))?;
    let result = iris_vm::run_with_natives(&program, registry);

    assert_eq!(result, Ok(Value::Nil));
    Ok(())
}

#[test]
fn finally_runs_when_catch_returns_from_startup() -> Result<(), String> {
    let source = r#"
        module Main {
            public module fun run(log) -> Nil {
                try {
                    try { raise :timeout } catch error { return nil }
                } finally {
                    log.append(:closed)
                }
                nil
            }
        }
        let log = []
        let result = Main.run(log)
        log
    "#;
    let program = iris_vm::compile(source).map_err(|error| error.construct)?;
    iris_vm::verify(&program).map_err(|error| format!("{error:?}"))?;

    let result = iris_vm::run(&program);

    assert_eq!(
        result,
        Ok(Value::Array(iris_runtime::ArrayRef::new(vec![
            Value::Symbol("closed".to_owned()),
        ])))
    );
    Ok(())
}

#[test]
fn return_unwinds_scopes_in_order_when_cleanup_nests() -> Result<(), String> {
    for (body, expected) in [
        (
            "try { try { return :answer } finally { log.append(:inner) } } finally { log.append(:outer) }",
            vec!["inner", "outer", "answer"],
        ),
        (
            "try { try { raise :timeout } catch error { return :answer } finally { log.append(:inner) } } finally { log.append(:outer) }",
            vec!["inner", "outer", "answer"],
        ),
        (
            "try { try { return :answer } finally { log.append(:inner); return :override } } finally { log.append(:outer) }",
            vec!["inner", "outer", "override"],
        ),
        (
            "try { try { return :answer } catch error { log.append(:wrong) } finally { raise :cleanup } } catch error { log.append(error); return :handled }",
            vec!["cleanup", "handled"],
        ),
        (
            "let name = :outer; try { let name = :inner; return :answer } finally { log.append(name) }",
            vec!["outer", "answer"],
        ),
        (
            "try { for item in [1] { try { return :answer } finally { log.append(:inner) } } } finally { log.append(:outer) }",
            vec!["inner", "outer", "answer"],
        ),
    ] {
        let source = format!(
            "module Main {{ public module fun run(log) {{ {body} }} }} \
             let log = []; let result = Main.run(log); let recorded = log.append(result); log"
        );
        let program = iris_vm::compile(&source).map_err(|error| error.construct)?;
        iris_vm::verify(&program).map_err(|error| format!("{error:?}: {body}"))?;

        let result = iris_vm::run(&program);

        assert_eq!(
            result,
            Ok(Value::Array(iris_runtime::ArrayRef::new(
                expected
                    .into_iter()
                    .map(|name| Value::Symbol(name.to_owned()))
                    .collect(),
            ))),
            "{body}"
        );
    }
    Ok(())
}

#[test]
fn package_errors_propagate_when_module_initialization_fails() -> Result<(), String> {
    for source in [
        "module Main { raise :startup_failed }",
        "module Main { return nil }",
    ] {
        let registry = Rc::new(NativeRegistry::new());
        let program = iris_vm::compile_packages_with_natives(
            &[("app".to_owned(), source.to_owned())],
            &registry,
        )
        .map_err(|error| error.construct)?;

        let result = iris_vm::run_with_natives(&program, registry);

        assert!(result.is_err(), "{source}: {result:?}");
    }
    Ok(())
}

#[test]
fn script_completion_is_preserved_when_only_declarations_are_present() -> Result<(), String> {
    let program = iris_vm::compile("module Main { nil }").map_err(|error| error.construct)?;

    let result = iris_vm::run(&program);

    assert_eq!(result, Err(iris_vm::MachineError::UnsupportedConstruct));
    Ok(())
}
