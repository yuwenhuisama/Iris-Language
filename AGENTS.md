# PROJECT KNOWLEDGE BASE

**Generated:** 2026-07-23
**Commit:** 40b6559
**Branch:** iris_dev

## OVERVIEW

Iris is a Windows-native dynamic, object-oriented scripting language. The Visual Studio 2017 solution builds a C++ DLL runtime/compiler, two extension DLL projects, and a script-driven integration runner.

The `legacy/` tree is archived, frozen, and not guaranteed to build. Iris v1 implementation work happens outside that tree.

## STRUCTURE

```text
./
├── legacy/IrisLangLibrary/          # compiler, bytecode VM, runtime, C/C++ APIs
├── legacy/Iris Library Test/        # console runner and .ir behavior corpus
├── legacy/Iris Pointer Extension/   # complete native extension example
├── legacy/Iris File Extension/      # incomplete extension stub; no plugin exports
├── legacy/Document/                 # language PDF and Notepad++ highlighting XML
└── legacy/Iris Programming Language.sln
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Build topology | `legacy/Iris Programming Language.sln` | Four projects; Debug/Release x Win32/x64 |
| Compiler pipeline | `legacy/IrisLangLibrary/src/IrisCompiler.cpp` | Loads/parses `.ir`, validates AST, emits `.irc` |
| Bytecode emission | `legacy/IrisLangLibrary/src/IrisInstructorMaker.cpp` | Instruction construction and label patching |
| VM execution | `legacy/IrisLangLibrary/src/IrisInterpreter.cpp` | Singleton state, opcode loop, registries, extension loader |
| Public host API | `legacy/IrisLangLibrary/include/IrisLangLibrary.h` | Selects C or C++ API using `IR_USE_C_EXPORT` |
| Native helper API | `legacy/IrisLangLibrary/include/IrisExportAPIs/` | `IR_*` lifecycle and `IrisDev*` extension surface |
| Script behavior tests | `legacy/Iris Library Test/test script/` | Split into expression and statement programs |
| Plugin pattern | `legacy/Iris Pointer Extension/` | Exports `IR_Initialize` and `IR_Release` |
| File extension status | `legacy/Iris File Extension/FileTag.h` | Stub only; does not satisfy full `IIrisClass` contract |

## CODE MAP

LSP symbol/reference indexing is unavailable for most MSVC sources in this environment; roles below come from source and project-file call paths.

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `IrisCompiler` | singleton class | `legacy/IrisLangLibrary/include/IrisCompiler.h` | Source ingestion, parse state, validation, code generation |
| `IrisInstructorMaker` | class | `legacy/IrisLangLibrary/include/IrisInstructorMaker.h` | Emits virtual-machine instructions |
| `IrisInterpreter` | singleton class | `legacy/IrisLangLibrary/include/IrisInterpreter.h` | Runtime registries, heap, dispatch, plugin loading |
| `IrisThreadManager` | singleton class | `legacy/IrisLangLibrary/include/IrisThread/IrisThreadManager.h` | Thread contexts and GC blocking coordination |
| `IrisGC` | singleton class | `legacy/IrisLangLibrary/include/IrisInterpreter/IrisNativeModules/IrisGC.h` | Marks interpreter and thread roots, sweeps runtime heap |
| `IR_Initialize` | exported function | `legacy/IrisLangLibrary/src/IrisExportAPIs/` | Host lifecycle entry; wires compiler and interpreter |
| `IR_LoadScriptFromPath` | exported function | `legacy/IrisLangLibrary/src/IrisExportAPIs/` | Compiles a source script before execution |
| `main` | executable entry | `legacy/Iris Library Test/main.cpp` | Runs one script path through the public API |

## CONVENTIONS

- MSVC project files are authoritative; there is no CMake, Makefile, CI, formatter, or linter configuration.
- Preserve paired public/private paths: declarations under `legacy/IrisLangLibrary/include/`, implementations under `legacy/IrisLangLibrary/src/`.
- Core headers usually use `_H_IRIS..._` guards; retain the local style in the file being changed.
- Runtime types use `Iris` prefixes, members use `m_`, singleton accessors use `CurrentX()`, and existing APIs deliberately spell `Regist` and `Extention`.
- Language-qualified names use `::`; public lifecycle exports use `IR_*`; native developer APIs use `IrisDev`/`IrisDevUtil`.
- `.ir` is source; `.irc` is generated binary virtual code. Script statements commonly begin with `;`.

## ANTI-PATTERNS (THIS PROJECT)

- Do not modernize established public misspellings; they are ABI/source contracts.
- Do not hand-edit `legacy/IrisLangLibrary/src/IrisComponents/LexYacc/lex.yy.cpp`, `y.tab.cpp`, or `include/IrisComponents/LexYacc/y.tab.h`; they are Flex/Bison outputs, and their `.l`/`.y` inputs are absent here.
- Do not treat `legacy/Iris File Extension` as a working plugin until it has the required `IIrisClass` methods and `IR_Initialize`/`IR_Release` exports.
- Do not change interpreter environment stacks or thread registers without auditing GC roots in `IrisGC.cpp` and nested execution in `IrisMethod.cpp`/`IrisClosureBlock.cpp`.
- Do not commit Visual Studio outputs (`Debug/`, `Release/`, `x64/`, `x86/`, `.vs/`, object/PDB files).

## UNIQUE STYLES

- AST nodes validate and generate bytecode; `IrisInterpreter::RunCode` executes the resulting opcode stream.
- Native built-ins pair an `IIrisClass` wrapper with a `*Tag` payload; wrappers define methods/allocation/marking, tags own C++ state.
- Runtime context is global-heavy: compiler, interpreter, GC, fatal handler, and thread manager are singleton services.
- Tests are executable examples, not assertions: each `.ir` file is passed individually to the console harness.

## COMMANDS

Run on Windows with Visual Studio 2017 Build Tools (`v141`) and Windows SDK `10.0.14393.0`:

```bat
msbuild "legacy/Iris Programming Language.sln" /p:Configuration=Debug /p:Platform=Win32
msbuild "legacy/Iris Programming Language.sln" /p:Configuration=Release /p:Platform=x64
"<path-to-Iris Library Test.exe>" "legacy/Iris Library Test\test script\expression\test_binary_expression.ir"
```

No repository-defined aggregate test, format, lint, or CI command exists.

## NOTES

- The checked-in build targets are Windows-only; this macOS workspace has no MSBuild, so solution builds cannot run here.
- `legacy/IrisLangLibrary/src/main.cpp` is an executable-style demo inside a DLL project and references missing `script/test.ir`; use the separate test project as the supported runner.
- `legacy/Iris Pointer Extension/Iris Pointer Extension.vcxproj` has a fragile Debug/Win32 dependency on `legacy/IrisLangLibrary\Debug\IrisValue.obj`.
- Some legacy comments display mojibake under UTF-8; avoid unrelated encoding churn.
