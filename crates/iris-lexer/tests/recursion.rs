use iris_lexer::{ByteOffset, Literal, TokenKind, convert_literals, lex, lex_type};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn nested_literal(depth: usize, delimiters: &[&str]) -> String {
    let mut source = String::from("let x=");
    for level in 0..depth {
        source.push_str(delimiters[level % delimiters.len()]);
        source.push_str("${");
    }
    source.push_str("\"x\"");
    for level in (0..depth).rev() {
        source.push('}');
        source.push_str(delimiters[level % delimiters.len()]);
    }
    source
}

#[test]
fn retains_nested_literals_when_within_the_shared_budget() {
    for delimiters in [&["\""][..], &["/"], &["\"", "/"], &["/", "\""]] {
        let source = nested_literal(31, delimiters);

        let result = lex(source.as_bytes());

        assert!(result.is_clean(), "{result:?}");
        assert_eq!(result.tokens().len(), 4);
        assert_eq!(result.tokens()[3].offset, ByteOffset(6));
        assert_eq!(result.tokens()[3].end, ByteOffset(source.len()));
        assert_eq!(
            result.tokens()[3].kind,
            if delimiters[0] == "/" {
                TokenKind::RegexLiteral
            } else {
                TokenKind::StringLiteral
            }
        );
    }
}

#[test]
fn preserves_interpolation_text_when_converting_nested_literals() {
    let source = nested_literal(10, &["\"", "/"]);

    let result = convert_literals(&source);

    assert!(result.diagnostics().is_empty());
    assert_eq!(
        result.values(),
        [Literal::String(source[7..source.len() - 1].to_owned())]
    );
}

#[test]
fn rejects_nested_literals_when_the_shared_budget_is_exceeded() {
    for delimiters in [&["\""][..], &["/"], &["\"", "/"], &["/", "\""]] {
        let source = nested_literal(32, delimiters);

        let result = lex(source.as_bytes());

        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(result.diagnostics()[0].code(), "LEX_RESOURCE_LIMIT");
        assert_eq!(result.diagnostics()[0].offset(), ByteOffset(6));
        assert!(result.tokens().is_empty());
    }
}

#[test]
fn allows_many_sibling_interpolations_when_each_is_shallow() {
    let source = format!("\"{}\"", "${\"x\"}".repeat(10000));

    let result = lex(source.as_bytes());

    assert!(result.is_clean(), "{result:?}");
    assert_eq!(result.tokens().len(), 1);
    assert_eq!(result.tokens()[0].end, ByteOffset(source.len()));
}

#[test]
fn ignores_interpolation_markers_when_the_literal_is_raw_or_single_quoted() {
    for (open, close) in [("r#\"", "\"#"), ("'", "'"), ("r/", "/")] {
        let source = format!("{open}{}{close}", "${".repeat(10000));

        let result = lex(source.as_bytes());

        assert!(result.is_clean(), "{result:?}");
        assert_eq!(result.tokens().len(), 1);
        assert_eq!(result.tokens()[0].end, ByteOffset(source.len()));
    }
}

#[test]
fn rejects_deep_literals_without_crashing_when_scanning_in_a_bounded_child() -> std::io::Result<()>
{
    if let Ok(case) = std::env::var("IRIS_LEXER_RECURSION_CASE") {
        let (operation, shape) = case
            .split_once(':')
            .ok_or(std::io::ErrorKind::InvalidInput)?;
        let delimiters: &[&str] = match shape {
            "string" => &["\""],
            "regex" => &["/"],
            "mixed" => &["\"", "/"],
            _ => unreachable!(),
        };
        let source = nested_literal(10000, delimiters);
        assert_eq!(source.len(), 50009);

        let codes: Vec<_> = match operation {
            "lex" => lex(source.as_bytes())
                .diagnostics()
                .iter()
                .map(|item| item.code())
                .collect(),
            "type" => lex_type(source.as_bytes())
                .diagnostics()
                .iter()
                .map(|item| item.code())
                .collect(),
            "convert" => convert_literals(&source).diagnostics().to_vec(),
            _ => unreachable!(),
        };

        assert_eq!(codes, ["LEX_RESOURCE_LIMIT"], "{case}");
        return Ok(());
    }
    for operation in ["lex", "type", "convert"] {
        for shape in ["string", "regex", "mixed"] {
            let case = format!("{operation}:{shape}");
            let mut child = Command::new(std::env::current_exe()?)
                .args([
                    "--exact",
                    "rejects_deep_literals_without_crashing_when_scanning_in_a_bounded_child",
                    "--nocapture",
                ])
                .env("IRIS_LEXER_RECURSION_CASE", &case)
                .env("RUST_MIN_STACK", "2097152")
                .stdout(Stdio::null())
                .spawn()?;
            let deadline = Instant::now() + Duration::from_secs(45);
            let status = loop {
                if let Some(status) = child.try_wait()? {
                    break status;
                }
                if Instant::now() >= deadline {
                    child.kill()?;
                    break child.wait()?;
                }
                std::thread::yield_now();
            };
            assert!(status.success(), "{case}: {status}");
        }
    }
    Ok(())
}
