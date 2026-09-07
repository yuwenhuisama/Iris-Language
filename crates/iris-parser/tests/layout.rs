use iris_parser::parse;

#[test]
fn preserves_decorator_order_when_declarations_start_on_newlines() {
    let compact = parse("@first() @second() class C { @trace() fun f() { 1 } }");
    let laid_out =
        parse("@first()\n@second()\nclass C {\n/// method\n@trace()\nfun f() {\n1\n}\n}");
    assert!(compact.program_accepted);
    assert!(laid_out.program_accepted, "{laid_out:?}");
    assert_eq!(compact.program, laid_out.program);
    assert!(!parse("@first(); class C {}").program_accepted);
}

#[test]
fn equivalent_ast_when_branches_continue_on_new_lines() {
    let pairs = [
        ("if true { 1 } else { 2 }", "if true { 1 }\nelse { 2 }"),
        (
            "let x = if true { 1 } else if false { 2 } else { 3 }",
            "let x = if true { 1 }\nelse if false { 2 }\nelse { 3 }",
        ),
        (
            "try { 1 } catch error { 2 } finally { 3 }",
            "try { 1 }\ncatch error { 2 }\nfinally { 3 }",
        ),
        (
            "let x = try { 1 } catch { 2 }",
            "let x = try { 1 }\ncatch { 2 }",
        ),
    ];
    for (compact, laid_out) in pairs {
        let expected = parse(compact);
        let actual = parse(laid_out);
        assert!(expected.program_accepted, "{compact}: {expected:?}");
        assert!(actual.program_accepted, "{laid_out}: {actual:?}");
        assert_eq!(actual.program, expected.program);
    }
}

#[test]
fn equivalent_ast_when_delimited_lists_span_lines() {
    let pairs = [
        ("f(1, option: 2,)", "f(\n1,\noption:\n2,\n)"),
        ("[1, 2,]", "[\n1,\n2,\n]"),
        ("%{a: 1, b: 2,}", "%{\na:\n1,\nb: 2,\n}"),
        ("(1, 2,)", "(\n1,\n2,\n)"),
        ("(1 + 2)", "(1\n+ 2)"),
        ("fun f(a: A, b: B,) { a }", "fun f(\na: A,\nb: B,\n) { a }"),
        (
            "class Pair<T, U> where T: A & B, U: C {}",
            "class Pair<\nT,\nU\n> where\nT:\nA &\nB,\nU: C\n{}",
        ),
        ("let x: Pair<A, B> = nil", "let x: Pair<\nA,\nB\n> = nil"),
        ("choose<A, B>(1)", "choose<\nA,\nB\n>(\n1\n)"),
        ("Box<A>.new()", "Box<\nA\n>.new(\n)"),
        ("let x = 1 + 2", "let x =\n1 +\n2"),
        (
            "fun f(a: A = 1 + 2) { a }",
            "fun f(\na: A = 1\n+ 2\n)\n{ a }",
        ),
        ("class A {}", "class A\n{}"),
        (
            "let x: Closure<(A, B) -> C> = nil",
            "let x: Closure<\n(\nA,\nB\n) -> C\n> = nil",
        ),
        ("{ |x|; x }", "{\n|x|\nx\n}"),
    ];
    for (compact, laid_out) in pairs {
        let expected = parse(compact);
        let actual = parse(laid_out);
        assert!(expected.program_accepted, "{compact}: {expected:?}");
        assert!(actual.program_accepted, "{laid_out}: {actual:?}");
        assert_eq!(actual.program, expected.program);
    }
}

#[test]
fn preserves_complete_statement_boundaries_when_newlines_are_present() {
    let pairs = [
        ("1; +2", "1\n+2"),
        ("f(); [1]", "f()\n[1]"),
        ("return; 1", "return\n1"),
        ("if true { 1 }; [2]", "if true { 1 }\n[2]"),
        ("f({ 1; +2 }, 3)", "f({ 1\n+2 },\n3)"),
        (
            "contract C { fun first() -> A; fun second() -> B }",
            "contract C {\nfun first() -> A\nfun second() -> B\n}",
        ),
    ];
    for (compact, laid_out) in pairs {
        let expected = parse(compact);
        let actual = parse(laid_out);
        assert!(expected.program_accepted, "{compact}: {expected:?}");
        assert!(actual.program_accepted, "{laid_out}: {actual:?}");
        assert_eq!(actual.program, expected.program);
    }
}

#[test]
fn rejects_semicolons_when_they_are_not_statement_terminators() {
    let sources = [
        ";",
        "1;;2",
        "[1;2]",
        "f(;)",
        "{ |x|;; x }",
        "{ ||;; 1 }",
        "{ ; 1 }",
        "class C<> {}",
        "class C<T,> {}",
        "let x: Box<A,> = nil",
    ];
    let results = sources.map(parse);
    for (source, result) in sources.into_iter().zip(results) {
        assert!(!result.program_accepted, "{source}: {result:?}");
        assert!(!result.diagnostics.is_empty());
    }
}

#[test]
fn preserves_closure_ast_when_header_terminator_is_omitted() {
    let pairs = [
        ("{ |t|; 1 }", "{ |t| 1 }"),
        ("{ ||; 9 }", "{ || 9 }"),
        (
            "{ |x: Integer| -> Integer; x }",
            "{ |x: Integer| -> Integer x }",
        ),
        ("{ || -> Integer; 9 }", "{ || -> Integer 9 }"),
    ];
    for (explicit, compatible) in pairs {
        let expected = parse(explicit);
        let actual = parse(compatible);
        assert!(expected.program_accepted, "{explicit}: {expected:?}");
        assert!(actual.program_accepted, "{compatible}: {actual:?}");
        assert_eq!(actual.program, expected.program);
    }
}

#[test]
fn preserves_separate_numeric_tokens_when_spelling_is_compact() {
    let expected = parse("1 + 2");
    let actual = parse("1+2");
    assert!(actual.program_accepted, "{actual:?}");
    assert_eq!(actual.program, expected.program);
    assert!(!parse("1 2").program_accepted);
}

#[test]
fn retains_raise_location_when_only_layout_changes() {
    let compact = "raise problem from cause";
    let laid_out = "\nraise problem\n  from cause";
    let expected = parse(compact);
    let actual = parse(laid_out);
    assert!(expected.program_accepted);
    assert!(actual.program_accepted, "{actual:?}");
    let [iris_syntax::Statement::Raise(Some(expected_raise))] =
        expected.program.statements.as_slice()
    else {
        unreachable!()
    };
    let [iris_syntax::Statement::Raise(Some(actual_raise))] = actual.program.statements.as_slice()
    else {
        unreachable!()
    };
    assert_eq!(actual_raise.value, expected_raise.value);
    assert_eq!(actual_raise.cause, expected_raise.cause);
    assert_eq!(expected_raise.offset, 0);
    assert_eq!(actual_raise.offset, 1);
}

#[test]
fn rejects_newline_only_property_accessors() {
    let result = parse("class C { property value: Integer { get\nset\n} }");
    assert!(!result.program_accepted, "{result:?}");
}

#[test]
fn accepts_property_accessors_when_semicolons_are_preserved() {
    let compact = parse("class C { property value: Integer { get; set; } }");
    let laid_out = parse("class C {\nproperty value: Integer {\nget;\nset;\n}\n}");
    assert!(compact.program_accepted);
    assert!(laid_out.program_accepted, "{laid_out:?}");
    assert_eq!(compact.program, laid_out.program);
}
