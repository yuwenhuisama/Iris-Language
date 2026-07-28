# IRIS LANGUAGE ENGINE

## OVERVIEW

This project owns the complete `.ir` to `.irc` to execution pipeline and the ABI consumed by hosts and native extensions. Public declarations mirror implementation directories under `include/` and `src/`.

## STRUCTURE

```text
IrisLangLibrary/
├── include/                 # public and internal declarations
│   ├── IrisComponents/      # AST nodes and generated parser interface
│   ├── IrisInterpreter/     # built-ins, payload tags, runtime structures
│   ├── IrisInterfaces/      # thin ABI-facing contracts
│   ├── IrisThread/          # thread and synchronization declarations
│   ├── IrisUnil/            # values, environments, containers, memory pools
│   └── IrisValidator/       # statement/expression visitor contracts
└── src/                     # matching implementations plus engine cores
```

## WHERE TO LOOK

| Change | Start here | Follow through |
|--------|------------|----------------|
| Parse/compile orchestration | `src/IrisCompiler.cpp` | AST validators, `IrisInstructorMaker` |
| Statement semantics | `src/IrisComponents/IrisStatements/` | Matching header and script test |
| Expression semantics | `src/IrisComponents/IrisExpressions/` | Matching header and script test |
| Opcode shape | `src/IrisInstructorMaker.cpp` | `IrisVirtualCodeStructures.h`, interpreter handler |
| Opcode behavior | `src/IrisInterpreter.cpp` | Thread registers, environment stack, GC roots |
| Class/module/interface semantics | `src/IrisInterpreter/IrisStructure/` | Base wrappers and `IrisDevelopUtil.cpp` |
| Built-in native type | `include/IrisInterpreter/IrisNativeClasses/` | Wrapper, `Tag`, both implementations |
| GC/object lifetime | `src/IrisInterpreter/IrisNativeModules/IrisGC.cpp` | `IrisObject`, environment heaps, thread state |
| Host/extension API | `include/IrisExportAPIs/` | matching `src/IrisExportAPIs/`, `IrisDevelopUtil.cpp` |

## PIPELINE

1. `IrisCompiler::LoadScript*` feeds source text to `yyparse()`.
2. Generated parser actions construct `IrisStatement`, `IrisExpression`, and `IrisParts` nodes.
3. `IrisCompiler::Generate()` validates nodes, invokes node `Generate()`, and asks `IrisInstructorMaker` to finalize virtual code.
4. `IrisInterpreter::Run()` creates the main context and calls `RunCode()` over compiler output.
5. User methods and closures temporarily push environments, recursively call `RunCode()`, then restore thread state.

## CONVENTIONS

- Add/change AST behavior in the matching `.h`/`.cpp` pair; validation belongs in visitors/node `Validate`, emission in node `Generate`.
- Native wrappers implement `NativeClassNameDefine`, `NativeSuperClassDefine`, `NativeAlloc`, `NativeFree`, `GetTrustteeSize`, `Mark`, and `NativeClassDefine`.
- A wrapper's `*Tag` owns native payload; mark every contained `IrisValue` reachable from a container tag.
- Register built-in methods/getters/setters inside `NativeClassDefine()` via `IrisDevUtil`.
- Preserve `IrisValue` pointer-identity semantics and existing raw-pointer ownership conventions unless the whole lifecycle is updated.
- Compile switches live in `include/IrisCompileConfigure.h`; defaults currently disable memory pools and C exports, and enable debug printing.

## ANTI-PATTERNS

- Never edit generated `LexYacc` outputs as the source of truth. The referenced `iris.l` and `iris.y` inputs are not checked in.
- Never add an opcode on only one side: emission, opcode declaration/serialization, dispatch, and handler behavior must remain synchronized.
- Never alter environment push/set/pop order in methods or closures without restoring file index and auditing GC reachability.
- Never add a native payload pointer without defining allocation, freeing, trusttee size, and marking behavior.
- Avoid broad cleanup of `IrisInterpreter.cpp`; its shared dispatch state gives local-looking changes wide blast radius.

## HOTSPOTS

- `src/IrisInterpreter.cpp` (~3K lines authored): lifecycle, registries, loader, dispatch, handlers.
- `src/IrisCompiler.cpp` (~846 lines): source buffers, parse state, validation, output.
- `src/IrisInstructorMaker.cpp` (~724 lines): instruction and label generation.
- `src/IrisInterpreter/IrisStructure/IrisClass.cpp` (~596 lines): inheritance/composition/method lookup.
- `src/IrisDevelopUtil.cpp` (~521 lines): common bridge used by exports and built-ins.
- `src/IrisComponents/LexYacc/{lex.yy.cpp,y.tab.cpp}` (~7K lines combined): generated, not authored complexity.

## VALIDATION

There are no isolated C++ unit tests. For a language feature, build the solution on Windows and run the closest script in `../Iris Library Test/test script/`; add a focused `.ir` case when existing scripts do not exercise the boundary.
