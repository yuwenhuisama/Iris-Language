# SCRIPT INTEGRATION TESTS

## OVERVIEW

This console project is the repository's only general test harness. It compiles and runs one Iris source file supplied on the command line; scripts demonstrate behavior through output rather than assertions.

## STRUCTURE

```text
Iris Library Test/
├── main.cpp                 # public-API lifecycle runner
└── test script/
    ├── expression/          # expression parsing, validation, execution
    ├── statement/           # declarations and control-flow behavior
    ├── main.ir              # empty; not an aggregate suite
    └── main.irc             # checked-in binary virtual-code artifact
```

## WHERE TO LOOK

| Behavior | Directory / examples |
|----------|----------------------|
| Operators and assignment | `test script/expression/test_binary_expression.ir` |
| Calls and closures | `test script/expression/test_functioncall_expression.ir` |
| Containers/indexing | `test script/expression/test_array_expression.ir`, `test script/expression/test_index_exxpression.ir` |
| Classes/inheritance | `test script/statement/test_class_statement.ir` |
| Functions/variadics | `test script/statement/test_function_statement.ir` |
| Modules/interfaces | `test script/statement/test_module_statement.ir`, `test script/statement/test_inteface_intefacefunction_statement.ir` |
| Control flow | `test script/statement/test_conditionif_statement.ir`, `test script/statement/test_for_statement.ir`, `test script/statement/test_switch_statement.ir` |
| Irregular handling | `test script/statement/test_order_statement.ir`, `test script/statement/test_order_statetement.ir` |

## RUNNER CONTRACT

`main.cpp` requires exactly one script path, then executes:

1. `IR_Initialize`
2. `IR_LoadScriptFromPath`
3. `IR_Run`
4. `IR_ShutDown`

Build on Windows first, then invoke the generated executable:

```bat
"<path-to-Iris Library Test.exe>" "Iris Library Test\test script\statement\test_class_statement.ir"
```

## CONVENTIONS

- Keep expression and statement cases in their existing semantic subdirectories.
- Use `.ir` for editable source. `.irc` is generated binary virtual code and is not reviewable text.
- Match the existing script surface: `;` introduces executable statements; definitions use `fun`, `class`, `module`, and `interface` blocks.
- A useful case must visibly distinguish success from failure through output or runtime errors; the harness has no assertion API.
- Run files individually. `test script/main.ir` is empty and no discovery/aggregate runner exists.
- Preserve established misspelled filenames in references unless deliberately renaming every dependent reference.

## ANTI-PATTERNS

- Do not claim these are unit tests; each case traverses parser, compiler, bytecode, and interpreter together.
- Do not edit `main.irc` as source.
- Do not add a silent behavior demo that cannot reveal whether the expected branch/value occurred.
- Do not assume all scripts run automatically; validation requires explicitly invoking each relevant file.

## COVERAGE GAPS

- No assertion framework, expected-output fixtures, negative-test runner, or aggregate command.
- Pointer extension has a separate smoke script under `../Iris Pointer Extension/test script.ir`.
- File extension has no runnable integration case because its plugin ABI is incomplete.
