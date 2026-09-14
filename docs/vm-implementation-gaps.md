# VM Implementation Gap Audit

Status: investigation complete for the probes below; fixes not implemented by this audit.
Date: 2026-09-08.

This is a targeted audit, not an exhaustive language-conformance claim. It separates
explicit backend refusals from accepted programs that omit required behavior. Agreement
between the VM and reference evaluator is not proof of conformance.

## Build Provenance

The probes used a freshly built CLI from commit
`5683e7cfbaae2caf24b02a1e4142b75f7eb1cf5e` plus preserved, pre-existing parser
worktree changes. This was not a pristine-commit build. The isolated build completed
with exit 0 using Rust/Cargo 1.97.1 on macOS arm64.

Binary SHA-256: `0338c8e384b7646307036eb8b124a2f9ed2dbd740dc95ab61a8f56c1aa0b1849`.

To repeat a source probe, save the complete block as a temporary `.iris` file and run:

```sh
IRIS=/absolute/path/to/iris
"$IRIS" probe.iris
"$IRIS" --vm probe.iris
```

Caught failures below exit successfully; that does not mean the invalid operation
was accepted correctly. Compare the body-entry markers and retained values.

## Confirmed Findings

| ID | Finding | Classification |
| --- | --- | --- |
| VM-DECORATOR-01 | Class decorator transformation never executes | Accepted program omits runtime semantics |
| VM-DECORATOR-02 | Decorated methods cannot compile | Explicit VM coverage refusal; reference also omits transform execution |
| VM-CONTRACT-01 | Invalid named Contract argument enters the method body | Missing boundary guard |
| VM-PROPERTY-01 | Typed stored property accepts an incompatible value | Missing write guard and invalid stored state |
| VM-INDEX-01 | Chained indexing of decorator metadata is refused | Explicit VM coverage limitation |
| SHARED-JSON-01 | Fractional JSON decoding fails in both engines | Shared implementation limitation, not VM-specific |

### VM-DECORATOR-01: Metadata Is Not Phase Execution

```iris
contract ClassDecorator {
  fun plan(declaration, arguments) -> Plan
  fun transform(declaration, arguments, context) -> Transformation
}
class Tag for ClassDecorator {
  public impl fun plan(declaration, arguments) -> Plan { Plan.empty }
  public impl fun transform(declaration, arguments, context) -> Transformation {
    print("TRANSFORM")
    Transformation.empty.add_method(:stamped) { 7 }
  }
}
@Tag(:baseline)
class Box {}
let names = Box.decorators
let applications = Box.decorator_arguments
let arguments = applications[0]
let phases = Box.decorator_phases
print(names[0])
print(arguments[0])
print(phases[0])
print(try { Box.new().stamped() } catch error { error.to_string() })
```

Reference stdout: `TRANSFORM\nTag\nbaseline\nstatic-and-runtime\n7\n`.
VM stdout: `Tag\nbaseline\nstatic-and-runtime\nMessageNotFound\n`.
Both exit 0 with empty stderr.

The VM's `compile/declarations.rs` stores decorator identity and rendered arguments;
`machine/runtime.rs` stages `DecoratorTransform::metadata` during registration.
This does not invoke source `plan` or `transform`. The reported
`static-and-runtime` describes declared members, not measured phase participation.
Nonliteral argument expressions also become empty strings in `decorator_literal`;
the consequence for arbitrary arguments needs separate runtime probes.

### VM-DECORATOR-02: Method Decorators

```iris
contract MethodDecorator {
  fun plan(declaration, arguments) -> Plan
  fun transform(declaration, arguments, context) -> Transformation
}
class Tag for MethodDecorator {
  public impl fun plan(declaration, arguments) -> Plan { Plan.empty }
  public impl fun transform(declaration, arguments, context) -> Transformation {
    print("METHOD_TRANSFORM")
    Transformation.empty
  }
}
class Box {
  @Tag()
  public fun value() -> Integer { 7 }
}
print(Box.new().value())
```

Reference: exit 0, stdout `7\n`, without the transform marker.
VM: exit 1, empty stdout, stderr
`iris: probe.iris: the machine does not cover method decorator` (origin varies).
The rejection is in `compile/declarations.rs::method_error`. Reference acceptance
must not be used as evidence that method decorators are implemented there.

### VM-CONTRACT-01: Rejection Happens Too Late

```iris
class Request {}
contract HandlerContract { fun handle(request: Request) -> String }
class Router for HandlerContract {
  public impl fun handle(request: Request) -> String { "OK" }
}
module Pipeline {
  public module fun run(handler: HandlerContract, request: Request) -> String {
    print("ENTER")
    handler.handle(request)
  }
}
print(Pipeline.run(Router.new(), Request.new()))
print(try { Pipeline.run("not a handler", Request.new()) } catch error { error.to_string() })
```

Reference stdout: `ENTER\nOK\nTypeContractError\n`.
VM stdout: `ENTER\nOK\nENTER\nMessageNotFound\n`.
The invalid call must be rejected before its body runs. In
`machine/composed_types.rs::annotation_admits`, an annotation name absent from
built-in and program Class lookup returns `Ok(true)`; this path does not resolve
the named Contract. A later missing-method error does not repair that boundary.

### VM-PROPERTY-01: Failed Write Must Preserve the Old Value

```iris
class Box { public property value: Integer = 0 }
let box = Box.new()
box.value = 7
print(box.value)
print(try { box.value = "wrong"; "ACCEPTED" } catch error { error.to_string() })
print(box.value)
```

Reference stdout: `7\nTypeContractError\n7\n`.
VM stdout: `7\nACCEPTED\nwrong\n`.
The automatic stored-property setter in `machine/stdlib.rs` reaches
`assign_raw_ivar` without enforcing the property's declared type. This finding
is about the guard, not whether setter-result semantics exist.

### VM-INDEX-01: Equivalent Access Shapes Differ in Coverage

Using the declarations in VM-DECORATOR-01, append:

```iris
print(Box.decorator_arguments[0][0].to_string())
```

The reference prints the additional `baseline` line. The VM rejects the whole
program before execution with `the machine does not cover index receiver`.
The intermediate-variable metadata access already present in VM-DECORATOR-01
is supported. This does not imply that all indexing or index assignment is absent.

### SHARED-JSON-01: Fractional Values

```iris
print(JSON.decode("15"))
print(try { JSON.decode("1.5") } catch error { error.to_string() })
```

Both engines print `15\nJSONSyntaxError\n` and exit 0. The VM scalar decoder in
`machine/stdlib/json.rs` handles integers but not fractional JSON numbers.
Existing agreement tests encode this limitation; agreement is not full JSON support.

## Decorator Implementation Contract and Pending Decisions

[Chapter 08](../spec/iris-v1/08-modules-metaprogramming.md), C085-C094 and
C122-C125, requires five named Decorator Contracts, both phase members,
top-to-bottom application, pure deterministic static planning, runtime transforms
inside candidate transactions, capability checks, and atomic failure. C124 fixes
`transform(declaration, arguments, context)`; C125 defines the minimal surfaces
`Plan.empty`, `Transformation.empty`, `add_method(selector, body)`, and `kind`.

Implementation scope is awaiting owner confirmation. The specification does not
fully define kind construction across all five targets, `add_method` applicability
to Method/property/Contract targets, decorator instance lifetime across phases,
or public context members. Body-wrapping and other transformation builders are
explicitly left for later additions by C125. This audit does not invent those APIs.

The owner was asked to choose between implementing the specified minimum with
explicit rejection of undefined operations, or defining the remaining semantics
before implementing the complete five-target system. No answer has been assumed.

Code inspection also identified decorator prerequisites requiring tests: Class
origin publication currently precedes executable class-body evaluation, Module
tables lack the Class revision machinery, and cross-package VM source execution
is refused. Upgrade/rollback and closed-generic decorator replay need explicit
coverage. These are integration concerns, not additional measured failures in
the matrix above.

No production code, frozen specification, conformance vector, or demo was changed
by this audit. The nondecorator findings are a repair backlog, not fixes completed
as part of the requested decorator work.
