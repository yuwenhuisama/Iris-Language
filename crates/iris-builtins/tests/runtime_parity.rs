use std::process::Command;

#[test]
#[ignore = "Requires IRIS_BUILTINS_CLI pointing to an existing iris executable"]
fn selected_descriptors_match_when_existing_cli_executes_both_backends()
-> Result<(), Box<dyn std::error::Error>> {
    let executable = std::env::var("IRIS_BUILTINS_CLI")?;
    let source = "print([1, 2].append(3))\nprint([1, 2].count())\nprint((1).mul_add(2, 3).to_bits())\nprint(JSON.encode([1], canonical: true))\nprint(m\"x\".bytes().to_string())\nprint((1 ..= 3).by(step: 2).to_array().to_string())";
    for backend in [None, Some("--vm")] {
        let mut command = Command::new(&executable);
        command.args(backend);
        let output = command.args(["-e", source]).output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout)?,
            "nil\n2\n1084227584\n[1]\nx\n[1, 3]\n"
        );
    }
    Ok(())
}

#[test]
#[ignore = "Requires IRIS_BUILTINS_CLI pointing to an existing iris executable"]
fn unsupported_surfaces_refuse_when_existing_cli_is_used() -> Result<(), Box<dyn std::error::Error>>
{
    let executable = std::env::var("IRIS_BUILTINS_CLI")?;
    for (backend, source) in [
        (None, "[1].join()"),
        (Some("--vm"), "[1].join()"),
        (Some("--vm"), "m\"x\".bytes"),
        (Some("--vm"), "\"x\".upcase()"),
        (None, "Encoding.default()"),
        (Some("--vm"), "Encoding.default()"),
        (None, "FFI.open(\"unused\").call(:missing)"),
        (Some("--vm"), "FFI.open(\"unused\").call(:missing)"),
    ] {
        let mut command = Command::new(&executable);
        command.args(backend);
        let output = command.args(["-e", source]).output()?;
        assert!(!output.status.success(), "{backend:?}: {source}");
        assert!(!output.stderr.is_empty());
    }
    Ok(())
}
