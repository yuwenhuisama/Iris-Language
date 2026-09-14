# AST Decorator Phase Slice

Implemented here: source Class applications and directly declared Class Method
applications run `plan` before runtime statements, on a fresh AST evaluator with
no native registry or shared runtime heap. Both core Contract members are
required. Each phase constructs its own zero-argument instance after activating
the private phase stack. Planning gates cover transitive AST expression and send
execution, constructors, raw class/global state, service dispatch and native or
async bodies. Unsupported planning effects fail closed with
`IRIS-DECORATOR-NONDETERMINISTIC` at `static`.

Runtime transformation uses the existing candidate transaction, immutable
declaration snapshots and the core `DecoratorContext`. Empty factories and
persistent operation append use the active phase kind. Errors unwind the phase
stack and restore candidate context. Closure records retain canonical full
parameters, async mode and return annotations.

## Remaining Protocol Work

- Wrapper descriptors are recorded but installation rejects nonempty Method
  transformations with `UnsupportedConstruct`. No Invocation/next execution,
  admission adapter, async wrapper bridge or published wrapper chain is claimed.
- `add_method` cannot install captured Closure environments yet; these are
  explicitly rejected rather than silently rebinding a Closure to the target.
- Origin identity publication is still the existing registry transaction model;
  atomic unpublished origin identity and complete group rollback remain pending.
- Planning reconstructs direct Class declaration artifacts, not arbitrary Module
  import/package graphs. Module/main, nested executable Method applications,
  Property applications, upgrade, rollback and closed materialization replay
  need their own canonical artifact lifecycle.
- Declaration snapshots expose read-only data, not executable `open`, `bind` or
  `call` handles. The complete C097 reflection adapter and exact generic Method
  signature metadata remain incomplete. No public AST API is introduced.
- Planning's pure built-in send set is deliberately limited. Unsupported pure
  helpers must gain explicit support, never a VM fallback.

Focused evidence is in `tests/decorator_phases.rs` and
`src/source_runtime/decorator_phase_tests.rs`. Existing tests using unresolved
decorator names or locally weakened pre-v1.35 Contracts are not compatibility
requirements and have not been modified or ignored.
