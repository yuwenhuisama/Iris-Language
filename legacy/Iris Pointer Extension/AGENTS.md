# POINTER NATIVE EXTENSION

## OVERVIEW

This DLL is the complete in-repo example of extending Iris: export the loader hooks, register an `IIrisClass` wrapper, and store native state in a payload tag.

## WHERE TO LOOK

| Concern | Location | Notes |
|---------|----------|-------|
| Plugin ABI | `Iris Pointer Extension.h`, `Iris Pointer Extension.cpp` | Exports `IR_Initialize`, `IR_Release` |
| Language-visible class | `IrisPointer.h`, `IrisPointer.cpp` | `IIrisClass` wrapper and native methods |
| Native byte buffer | `IrisPointerTag.h`, `IrisPointerTag.cpp` | Allocation, bounds, payload size |
| DLL entry stub | `dllmain.cpp` | No custom attach/detach work |
| Smoke script | `test script.ir` | Imports DLL and exercises Pointer methods |
| Build coupling | `Iris Pointer Extension.vcxproj` | References `IrisLangLibrary` |

## EXTENSION CONTRACT

- The host resolves an export named exactly `IR_Initialize`; shutdown looks for `IR_Release`.
- `IR_Initialize()` registers `Pointer` through `IrisDev::RegistClass`.
- `IrisPointer::NativeClassDefine()` registers `__format`, `get_data`, `set_data`, and `@length`.
- `IrisPointerTag` owns the buffer; wrapper allocation/free/trusttee-size methods connect it to runtime lifetime accounting.
- Native callback signatures must keep all five runtime arguments, including context and thread information.

## CONVENTIONS

- Keep plugin exports in the project umbrella header and C++ implementation.
- Put language method wiring in `NativeClassDefine`, behavior in static wrapper callbacks, and raw storage operations in the tag.
- Validate all Iris arguments before extracting native pointers or primitive values.
- Return Iris values through `IrisDev` factories and report language-level failures through runtime irregular mechanisms.
- Update `test script.ir` whenever the public Pointer method surface changes.

## ANTI-PATTERNS

- Do not rename `IR_Initialize`, `IR_Release`, `RegistClass`, or other legacy public symbols.
- Do not register a class before all allocation/free/mark/trusttee-size hooks are coherent.
- Do not bypass tag bounds checks from wrapper methods.
- Do not copy the incomplete `Iris File Extension` as an implementation template.

## KNOWN RISKS

- `IrisPointerTag::Get` currently builds the result from `m_pBuffer` rather than `m_pBuffer + nPointer`; offset behavior deserves a regression case before modification.
- `Initialize` allocates an uninitialized buffer and assumes one initialization per payload.
- Debug/Win32 links directly to `IrisLangLibrary\Debug\IrisValue.obj`; preserve or deliberately replace this project-specific coupling.
