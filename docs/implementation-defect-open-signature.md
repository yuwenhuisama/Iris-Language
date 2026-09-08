# Conformance Defects: Incompatible Method Return Signature Replacement and Local Reassignment Bypasses

Status: Resolved for the four reported defects; frozen-vector conflicts documented below
Date: 2026-09-07 (Updated: 2026-09-08)  
Component: Metaprogramming / Type System Boundary Enforcement / Local Bindings  
Tested Binary: existing `./target/debug/iris` (version `iris 0.1.0`, macOS; fresh rebuild blocked, see provenance below)  
Source HEAD at verification: `6f368bf503365fab2588354b7e37076bcfe9cb92`  

## Local Identifier Index

The identifiers below are document-local tracking tags for repair agents, not normative specification identifiers:

1. `OPEN-SIGNATURE-01`: Incompatible method return signature replacement permitted in open class transaction.
2. `LOCAL-INFERRED-01`: Inferred mutable local binding permits reassignment to incompatible type.
3. `LOCAL-ANNOTATED-01`: Explicitly typed mutable local binding permits reassignment to incompatible type.
4. `LOCAL-UNION-01`: Explicit union-typed mutable local binding permits reassignment to type outside union.

## Executive Summary

This handoff tracks four observed conformance defects where the implementation accepts type-incompatible operations without error:

1. Under `OPEN-SIGNATURE-01`, when an existing class method is reopened via `open class` and overridden with an incompatible return signature (`Integer -> String` replacing `Integer -> Integer`), both the register bytecode virtual machine (`--vm`) and the tree-walking reference evaluator execute the replacement without error and publish the revision. The published method returns a `String` value to existing callers holding references across the mutation boundary.
2. Under `LOCAL-INFERRED-01`, `LOCAL-ANNOTATED-01`, and `LOCAL-UNION-01`, mutable local bindings (`mut`) permit reassignment to incompatible values without static diagnosis or runtime boundary rejection. In all three cases, both `--vm` and the reference evaluator execute the reassignment, store the incompatible value, and exit with code `0`.

Under Iris v1 specifications [`IRIS-V1-IDENTITY-C008`](../spec/iris-v1/01-language-identity.md) and [`IRIS-V1-TYPES-C038`](../spec/iris-v1/05-types-contracts-generics.md), dynamic mutation may change implementation details, but it must preserve static promises before atomic publication. Under [`IRIS-V1-CONTROL-C004`](../spec/iris-v1/04-bindings-callables-control-flow.md), [`IRIS-V1-CONTROL-C005`](../spec/iris-v1/04-bindings-callables-control-flow.md), [`IRIS-V1-TYPES-C004`](../spec/iris-v1/05-types-contracts-generics.md), and [`IRIS-V1-TYPES-C007`](../spec/iris-v1/05-types-contracts-generics.md), a local binding possesses a fixed type that cannot be widened implicitly. Incompatible assignments provable at compile time must be diagnosed before execution, and crossing guards must reject incompatible values before storing them into binding cells.

The method replacement finding comes from article 01 and its subsequent defect-report verification. The local assignment findings and callable controls come from article 02 (`/Users/huisama/Project/articles/iris-lang/02-types-and-boundaries.md`). All are historical observations on the tested binary; no fresh rebuild or rerun was conducted during this edit. The filename is retained so existing report links remain valid.

## Normative Basis

The observed behaviors conflict with the following normative requirements:

1. [`IRIS-V1-IDENTITY-C008`](../spec/iris-v1/01-language-identity.md):
   "Iris v1 uses static promises to bound dynamic behavior. Static facts such as declared superclass, declared Contracts, visible member names and signatures, typed properties, generic constraints, native layout obligations, package identity, and API major identity MUST be preserved across dynamic mutation, open transactions, package upgrade, reflection, native binding, and optimization."
2. [`IRIS-V1-TYPES-C038`](../spec/iris-v1/05-types-contracts-generics.md):
   "Method implementation compatibility for superclass, Contract, Module, intersection, open, package upgrade, native metadata, and dynamic replacement uses the callable subtyping rule in IRIS-V1-TYPES-C037. A compatible body replacement may change implementation details but MUST preserve the static promise before atomic publication."
3. [`IRIS-V1-TYPES-C037`](../spec/iris-v1/05-types-contracts-generics.md):
   Callable subtyping requires parameter contravariance and return covariance. Because `String` is not a subtype of `Integer`, replacing `(Integer) -> Integer` with `(Integer) -> String` is incompatible.
4. [`IRIS-V1-TYPES-C003`](../spec/iris-v1/05-types-contracts-generics.md):
   "Type annotations are optional to write and mandatory to obey once written. An omitted Method parameter or return annotation has static and runtime Contract Dynamic<Object>."
5. [`IRIS-V1-TYPES-C004`](../spec/iris-v1/05-types-contracts-generics.md):
   "A written type annotation on a binding, property, parameter, return, generic argument, Contract requirement, native metadata entry, or Type alias target is both a static Contract and a runtime boundary guard. A provable violation MUST be diagnosed before execution. A not-proven boundary MUST check at runtime before publishing or passing the value across that boundary."
6. [`IRIS-V1-TYPES-C006`](../spec/iris-v1/05-types-contracts-generics.md):
   Boundary contract failures must raise or report `TypeError`, `TypeContractError`, or a more specific ordinary Iris diagnostic named by the owning chapter.
7. [`IRIS-V1-TYPES-C007`](../spec/iris-v1/05-types-contracts-generics.md):
   The gradual boundary table specifies for binding annotations (`let x: T = expr` and mutable equivalents) that statically `expr` must be assignable to `T`, and runtime guards require that stored values satisfy `T`.
8. [`IRIS-V1-CONTROL-C004`](../spec/iris-v1/04-bindings-callables-control-flow.md):
   "After a binding has a declared or inferred local type, later assignments MUST satisfy that fixed type and MUST NOT widen it implicitly."
9. [`IRIS-V1-CONTROL-C005`](../spec/iris-v1/04-bindings-callables-control-flow.md):
   "A binding annotation is a static and runtime contract for the binding cell. Without an annotation, the initializer's precise static type becomes the binding's fixed local type. Programs that need a wider target type MUST write it explicitly, such as mut value: String | Integer = \"ready\" or mut value: Dynamic<Object> = source."
10. [`IRIS-V1-META-C034`](../spec/iris-v1/08-modules-metaprogramming.md):
   "An open transaction body receives stable logical identity for its target while structural operations mutate one implicit candidate for each target in the current transaction group. Normal completion validates and commits. Exception, invalid control transfer, validation error, capability denial, permission denial, or conflict rolls back every candidate in the group."

### Distinct Concepts and Exclusions

This report isolates specific boundary verification gaps from unrelated language rules:

- **Absence of declared Contract**: The method replacement reproducer defines no nominal `contract` and declares no `for` conformance. Existing defect entry `IRIS-V1-TYPES-V203` in `docs/spec-defects-v1.md` resolved return type validation during open transactions specifically for contract-visible requirements (`IRIS-V1-TYPES-C045`). This defect must not be conflated with `V203` or treated as a regression of contract checking.
- **Generic callable invariance**: `IRIS-V1-TYPES-C096` establishes that callable generic type arguments are invariant (for example, `Closure<S>`) to eliminate subtyping between first-class callable values, verifying compatibility at call sites instead. That rule governs callable objects and does not govern declaration-level method replacement compatibility, which remains explicitly governed by `IRIS-V1-TYPES-C038`. We do not propose new variance rules.
- **Transaction rollback scope**: `IRIS-V1-META-C034` mandates candidate rollback upon validation failure. In `OPEN-SIGNATURE-01`, the candidate transaction was accepted and published rather than rejected. Because no rejected transaction occurred, this observation does not demonstrate an engine rollback failure.
- **Scope of type enforcement in current binary**: These four observations do not mean that all runtime type checking is absent, nor do they claim that immutable types or parameter guards are broken. Parameter guards and return guards continue to operate correctly as shown in the baseline verification controls below.

## Measured Repro Matrix

### Case 1: Method Return Replacement (`OPEN-SIGNATURE-01`)

#### Incompatible Replacement Probe

The test script declares `ShippingRule.total` returning `Integer`, invokes it on an instance, reopens `ShippingRule` with `override fun total` returning `String`, and invokes it again on the same receiver.

Reproducible `-e` shell commands:

```bash
# Register bytecode VM backend
./target/debug/iris --vm -e 'class ShippingRule { public fun total(subtotal: Integer) -> Integer { subtotal + 10 } }; let rule = ShippingRule.new(); print(rule.total(90)); open class ShippingRule { public override fun total(subtotal: Integer) -> String { "free shipping" } }; print(rule.total(90));'

# Reference evaluator backend
./target/debug/iris -e 'class ShippingRule { public fun total(subtotal: Integer) -> Integer { subtotal + 10 } }; let rule = ShippingRule.new(); print(rule.total(90)); open class ShippingRule { public override fun total(subtotal: Integer) -> String { "free shipping" } }; print(rule.total(90));'
```

#### Compatible Control Probe

The control script reopens `ShippingRule` with a compatible signature (`Integer -> Integer`):

```bash
# Register bytecode VM backend
./target/debug/iris --vm -e 'class ShippingRule { public fun total(subtotal: Integer) -> Integer { subtotal + 10 } }; let rule = ShippingRule.new(); print(rule.total(90)); open class ShippingRule { public override fun total(subtotal: Integer) -> Integer { subtotal } }; print(rule.total(90));'

# Reference evaluator backend
./target/debug/iris -e 'class ShippingRule { public fun total(subtotal: Integer) -> Integer { subtotal + 10 } }; let rule = ShippingRule.new(); print(rule.total(90)); open class ShippingRule { public override fun total(subtotal: Integer) -> Integer { subtotal } }; print(rule.total(90));'
```

### Case 2: Inferred Mutable Local Reassignment (`LOCAL-INFERRED-01`)

Standalone reproducer script:

```iris
mut port = 8080
port = "9090"
print(port is String)
```

Execution commands:

```bash
# Register bytecode VM backend
./target/debug/iris --vm -e 'mut port = 8080; port = "9090"; print(port is String)'

# Reference evaluator backend
./target/debug/iris -e 'mut port = 8080; port = "9090"; print(port is String)'
```

- **Observed Actual**: Exit code `0`, stdout `true\n`, stderr `""`. Both backends update `port` to a string.
- **Expected Conformance**: Under `IRIS-V1-CONTROL-C004` and `IRIS-V1-CONTROL-C005`, `port` acquires the fixed type `Integer`. Provable assignments of `String` must be rejected before execution with a static diagnostic, or dynamic crossing must be rejected before storing. Exit code must be non-zero and standard error must describe the contract violation.

### Case 3: Annotated Mutable Local Reassignment (`LOCAL-ANNOTATED-01`)

Standalone reproducer script:

```iris
mut port: Integer = 8080
port = "9090"
print(port is String)
```

Execution commands:

```bash
# Register bytecode VM backend
./target/debug/iris --vm -e 'mut port: Integer = 8080; port = "9090"; print(port is String)'

# Reference evaluator backend
./target/debug/iris -e 'mut port: Integer = 8080; port = "9090"; print(port is String)'
```

- **Observed Actual**: Exit code `0`, stdout `true\n`, stderr `""`. Both backends update `port` to a string.
- **Expected Conformance**: Under `IRIS-V1-TYPES-C004`, `IRIS-V1-TYPES-C007`, and `IRIS-V1-CONTROL-C004`, `port` is explicitly bounded to `Integer`. Assigning `"9090"` must be diagnosed statically or rejected at the storage boundary before assignment completes. Exit code must be non-zero.

### Case 4: Union-Annotated Mutable Local Reassignment (`LOCAL-UNION-01`)

Standalone reproducer script:

```iris
mut port: Integer | String = 8080
port = true
print(port is Bool)
```

Execution commands:

```bash
# Register bytecode VM backend
./target/debug/iris --vm -e 'mut port: Integer | String = 8080; port = true; print(port is Bool)'

# Reference evaluator backend
./target/debug/iris -e 'mut port: Integer | String = 8080; port = true; print(port is Bool)'
```

- **Observed Actual**: Exit code `0`, stdout `true\n`, stderr `""`. Both backends update `port` to a boolean.
- **Expected Conformance**: Under `IRIS-V1-CONTROL-C004`, `IRIS-V1-CONTROL-C005`, and `IRIS-V1-TYPES-C004`, the binding is bounded to `Integer | String`. Reassignment to `true` (`Bool`) falls outside the union and must be rejected statically or dynamically before storing. Exit code must be non-zero.

### Measured Execution Results Across All Probes

Historical measurements from the method-replacement report and article 02 verification (no fresh evaluation run):

| Identifier | Probe Description | Backend | Exit Code | Stdout | Stderr | Signal |
| --- | --- | --- | --- | --- | --- | --- |
| `OPEN-SIGNATURE-01` | Incompatible method replacement (`-> String`) | VM (`--vm`) | `0` | `100\nfree shipping\n` | `""` | `null` |
| `OPEN-SIGNATURE-01` | Incompatible method replacement (`-> String`) | Reference (default) | `0` | `100\nfree shipping\n` | `""` | `null` |
| Control | Compatible method replacement (`-> Integer`) | VM (`--vm`) | `0` | `100\n90\n` | `""` | `null` |
| Control | Compatible method replacement (`-> Integer`) | Reference (default) | `0` | `100\n90\n` | `""` | `null` |
| `LOCAL-INFERRED-01` | Inferred local reassignment (`8080` -> `"9090"`) | VM (`--vm`) | `0` | `true\n` | `""` | `null` |
| `LOCAL-INFERRED-01` | Inferred local reassignment (`8080` -> `"9090"`) | Reference (default) | `0` | `true\n` | `""` | `null` |
| `LOCAL-ANNOTATED-01` | Explicit `Integer` reassignment (`8080` -> `"9090"`) | VM (`--vm`) | `0` | `true\n` | `""` | `null` |
| `LOCAL-ANNOTATED-01` | Explicit `Integer` reassignment (`8080` -> `"9090"`) | Reference (default) | `0` | `true\n` | `""` | `null` |
| `LOCAL-UNION-01` | Union `Integer \| String` reassignment (`8080` -> `true`) | VM (`--vm`) | `0` | `true\n` | `""` | `null` |
| `LOCAL-UNION-01` | Union `Integer \| String` reassignment (`8080` -> `true`) | Reference (default) | `0` | `true\n` | `""` | `null` |

## Measured Pass Controls and Baseline Enforcement

To establish that the tested runtime engine enforces type contracts at other boundaries, the following controls were previously measured and confirmed operational on the tested binary.

### Local Assignment Pass Controls

Compatible updates to mutable bindings succeed as expected under both backends:

```bash
# Inferred integer mutable reassigned to compatible integer
./target/debug/iris --vm -e 'mut port = 8080; port = 9090; print(port)'
# Reference evaluator
./target/debug/iris -e 'mut port = 8080; port = 9090; print(port)'
```

- Exit code: `0`
- Stdout: `9090\n`
- Stderr: `""`

```bash
# Union mutable reassigned to permitted union member
./target/debug/iris --vm -e 'mut port: Integer | String = 8080; port = "9090"; print(port)'
# Reference evaluator
./target/debug/iris -e 'mut port: Integer | String = 8080; port = "9090"; print(port)'
```

- Exit code: `0`
- Stdout: `9090\n`
- Stderr: `""`

### Method Parameter Guard Control

Method parameter boundaries intercept incompatible types prior to entering the method body:

```iris
module Input {
  public module fun read(value) {
    value
  }
}

module Server {
  public module fun start(port: Integer) -> Integer {
    print("entered start")
    port
  }
}

print(Server.start(Input.read(8080)))
print(Server.start(Input.read(-1)))
```

When evaluated with valid integers (`8080` and `-1`), both backends execute without error:
- Exit code: `0`
- Stdout: `entered start\n8080\nentered start\n-1\n`
- Stderr: `""`

When called with an incompatible string:

```iris
module Input {
  public module fun read(value) {
    value
  }
}

module Server {
  public module fun start(port: Integer) -> Integer {
    print("entered start")
    port
  }
}

print(Server.start(Input.read("8080")))
```

Execution command:

```bash
./target/debug/iris --vm -e 'module Input { public module fun read(value) { value } }; module Server { public module fun start(port: Integer) -> Integer { print("entered start"); port } }; print(Server.start(Input.read("8080")));'
./target/debug/iris -e 'module Input { public module fun read(value) { value } }; module Server { public module fun start(port: Integer) -> Integer { print("entered start"); port } }; print(Server.start(Input.read("8080")));'
```

- Exit code: `1`
- Stdout: `""` (the marker `entered start` is never reached)
- Stderr: `iris: -e: TypeContractError\n`

### Method Return Guard Control

Method return guards intercept incompatible values upon method exit:

```iris
module Input {
  public module fun read(value) {
    value
  }
}

module Settings {
  public module fun port(raw) -> Integer {
    print("entered port")
    Input.read(raw)
  }
}

print(Settings.port("8080"))
```

Execution command:

```bash
./target/debug/iris --vm -e 'module Input { public module fun read(value) { value } }; module Settings { public module fun port(raw) -> Integer { print("entered port"); Input.read(raw) } }; print(Settings.port("8080"));'
./target/debug/iris -e 'module Input { public module fun read(value) { value } }; module Settings { public module fun port(raw) -> Integer { print("entered port"); Input.read(raw) } }; print(Settings.port("8080"));'
```

- Exit code: `1`
- Stdout: `entered port\n` (method body executed)
- Stderr: `iris: -e: TypeContractError\n`

Both backend modes produced the stated results. These specific controls show rejection before the parameter-case body marker and after the return-case marker. They do not prove every callable boundary is correct or pinpoint the internal checking phase. Argument evaluation can itself have effects; the return-case marker survives failure, so neither result is a general side-effect rollback guarantee.

## Environmental Limits and Build Audit

- **Environment**: macOS, audit date 2026-09-07.
- **Tested binary**: Existing executable `./target/debug/iris` reporting `iris 0.1.0`. Source HEAD was `6f368bf503365fab2588354b7e37076bcfe9cb92`. This is not an asserted binary-to-source attribution for this rerun.
- **Build status**: During the original report verification, `/Users/huisama/.cargo/bin/cargo build --locked -p iris-cli` timed out after 120 seconds waiting on a build directory file lock. Article 02 used the existing binary without rebuilding. The binary is not asserted to include uncommitted changes from the dirty worktree.
- **Worktree state**: Parallel lexer and parser modifications, untracked files, and `.debug-journal.md` were present in the worktree and left completely untouched. The original investigation of this behavior occurred at the same commit `6f368bf503365fab2588354b7e37076bcfe9cb92` where clean compilation had previously succeeded with identical VM results.
- **Historical documentation verification**: Method replacement results predate article 02; local assignment and callable controls were checked for article 02. Shared VM changes were also present during article 02 verification. No fresh binary run was executed during this documentation extension.

## Root Cause and Impact Assessment

### Root Cause Analysis

The underlying root causes remain unconfirmed and unspeculated. Separate causes may explain method replacement publication versus local assignment bypasses.

Potential areas for investigation include:
- Whether open class validation omits subtyping checks (`IRIS-V1-TYPES-C038`) during candidate transaction verification.
- Whether AST validation, parser analysis, or semantic analysis tracks binding type cells for local variables during assignment lowering.
- Whether bytecode emission or the VM interpreter omits boundary check instructions on local assignment slots.
- Whether evaluator environments drop type contract metadata on local frames.

No speculation is asserted as fact. A repairing engineer must audit the source code directly.

Under `IRIS-V1-TYPES-C006`, boundary failures must report `TypeError`, `TypeContractError`, or a more specific ordinary diagnostic named by the owning chapter. This report avoids prescribing mandatory exact error strings.

### Impact Analysis

- **Observed**: A source-declared public method signature changes underneath existing receivers at runtime. Local mutable bindings accept values outside their declared or inferred bounds. Whether internal type metadata widens, is lost, or is bypassed was not measured.
- **Not observed / Non-claims**: There is no evidence of memory corruption, segmentation faults, host panics, or security boundary escapes. Parameter guards, return guards, and compatible reassignments continue to enforce contracts as demonstrated.

## Concrete Repair Checklist for Repair Agents

Repair agents taking on these defects must adhere to the following checklist:

1. **Rebuild in an isolated environment**:
   - Use an isolated worktree and target directory, or wait for legitimate builds to finish. Do not delete another process's lock or discard concurrent edits.
   - Rebuild the CLI binary (`cargo build -p iris-cli`) from a clean tree and record the exact Git commit revision.
   - Verify whether each defect reproduces on the freshly compiled binary before attempting code edits.

2. **Add failing regression tests for all four defect cases**:
   - Add negative tests for `OPEN-SIGNATURE-01`: incompatible return type replacement in open class must fail.
   - Add negative tests for `LOCAL-INFERRED-01`: reassignment of inferred local to incompatible type must fail.
   - Add negative tests for `LOCAL-ANNOTATED-01`: reassignment of explicitly typed local to incompatible type must fail.
   - Add negative tests for `LOCAL-UNION-01`: reassignment of union-typed local to type outside union must fail.

3. **Verify both runtime backends**:
   - Run tests against both the register bytecode virtual machine (`--vm`) and the tree-walking reference evaluator.
   - Ensure both backends consistently diagnose or reject the invalid operations.

4. **Verify compatible controls continue to pass**:
   - Ensure compatible method replacements (`Integer -> Integer`) succeed and publish cleanly.
   - Ensure compatible local assignments (`mut port = 8080; port = 9090` and union assignments to member types) continue to evaluate without error.
   - Ensure method parameter guards and return guards remain functioning.

5. **Test dynamic paths beyond static literals**:
   - Add dynamic reassignment tests where the assigned value comes from an untyped helper (such as `Input.read(...)`) rather than a literal expression, testing the runtime boundary crossing guard.
    - Verify that when a dynamic assignment fails at runtime, the target local binding cell retains its prior valid value rather than being corrupted or partially written. If static rejection prevents execution of literal cases, test dynamic-value cases via the test harness.

   - For the open-class failure, independently assert that the active revision and member table remain unchanged, including a sibling staged member, and that an existing instance still uses the original implementation. Use a harness if static rejection or uncatchable diagnostics prevent observing state from one source program.
   - These dynamic-value and failed-publication checks are proposed regression coverage, not results already measured in this report. The literal assignment cases are statically provable violations; runtime rejection alone does not close their required pre-execution diagnostic coverage.

6. **Preserve specification and diagnostic integrity**:
   - Do not weaken existing tests or alter frozen specification clauses.
   - Do not perform implicit data conversion (for example, do not auto-convert strings to integers to bypass assignment errors).
   - Do not suppress diagnostics or catch errors silently.
   - Ensure diagnostics emit `TypeError`, `TypeContractError`, or the appropriate chapter-specific diagnostic.

All defect statuses above remain the historical observations recorded on the original tested binary. Engineering teams must recheck current code before concluding repairs.

## Repair Verification and Current Implementation State (2026-09-08)

Current Status: Verified; final correctness review passed

An independent audit verified that the four historical defect cases reproduced on clean baseline commit `3dd4f23`. The repair was then integrated with concurrent parser work; the working-tree HEAD at the verification scope audit was `f260d5f68e7566e6649cb058323b6e919cd225c8`. The uncommitted implementation was built and verified in a fresh isolated target directory (`/var/folders/tq/hgbd_kk13_1bgdj9c8983s8h0000gn/T/opencode/iris-boundary-final-20260908`).

Workspace verification results:
- `cargo test --locked --workspace --quiet` exited with code `0` (including `iris-eval` with 739 passed and 1 ignored test, `iris-cli` with 52 passed tests, and native integrations).
- `cargo fmt --all -- --check` exited with code `0`.
- `cargo clippy --workspace --all-targets -- -D warnings` exited with code `0`.

Verification of actual CLI behavior on both backends (`--vm` and reference):
- Open class incompatible return signature probe (`OPEN-SIGNATURE-01`): exits with code `1`, stdout `100\n`, stderr containing `TypeContractError` on both backends. The candidate transaction aborts before publication, preserving the original method implementation for existing receivers.
- Literal local reassignments (`LOCAL-INFERRED-01`, `LOCAL-ANNOTATED-01`, `LOCAL-UNION-01`): all exit with code `1`, producing no stdout. The reference evaluator reports `TypeContractError` and the bytecode VM reports `TypeContractError: BINDING_FIXED_LOCAL_TYPE`. Static diagnostics reject provably invalid literal reassignments before execution.
- Dynamic assignment boundary guard: dynamic invalid reassignments are rejected at runtime prior to cell storage. The target cell retains its previous valid value (for example, retaining `8080`) rather than suffering corruption or partial write.
- Staged candidate isolation and atomic rollback: invalid mutations in open class blocks roll back atomically. Sibling staged methods (such as an alias or staged new body) do not leak into the active class revision upon failure.
- Type alias normalization: cloned AST aliases and normalized type aliases preserve equivalence across chunks and boundaries (such as `Port` and `Integer` signatures remaining compatible).
- Callable subtyping: method replacement validates the final candidate's parameter contravariance and return covariance against the previously published signature.

Conformance suite differential status:
- All 736 runnable source vectors agree between backends: `agreed=736 disagreed=0 held=0`.
- The `TYPES-V207` readonly-array discrepancy is resolved. Array guards admit the runtime-owned readonly representation without granting mutation permission; focused tests preserve readonly mutation rejection and incompatible-write rejection.
- Differential agreement is not a claim that every vector's normative expectation passes. The chapter-level results and conflicts are recorded below.
- Final Oracle review passed after correcting evaluator candidate lookup to stop at staged or published tombstones. Six evaluator and four dual-backend CLI regressions cover this correction; incompatible inherited replacements remain rejected.
- After that correction, the full workspace test command, workspace/all-target Clippy with `-D warnings`, formatting checks, and the 736-vector differential sweep were rerun successfully. The supported main-workspace CLI was rebuilt with `cargo build --locked -p iris-cli`.
- Validation ran on macOS arm64. Rust LSP was unavailable and was not claimed as a successful check. Temporary baseline worktrees and verification targets were removed after recording the evidence; no commits or pushes were performed for this repair.

### Chapter-Level Limits

The frozen vectors were not edited or reclassified. Clean baseline `3dd4f23` and the repaired working tree were run separately:

| Chapter | Clean baseline | Repaired working tree | Interpretation |
| --- | --- | --- | --- |
| CONTROL | 183 passed, 0 failed | 178 passed, 5 failed | Five positive fixtures rely on implicit widening from an inferred `Nil` local. |
| TYPES | 103 passed, 1 failed | 103 passed, 1 failed | `TYPES-V200` already expected revision 3 where the implementation reports 1. |
| META | 85 passed, 6 failed, 1 needs subsystem | 85 passed, 6 failed, 1 needs subsystem | Existing revision/status discrepancies remain; `META-V353` additionally reaches a fixed-local type rejection. |

`CONTROL-V014`, `V022`, `V298`, `V325`, and `V362` initialize mutable locals with `nil` and later store other types without a wider annotation. Their positive expectations conflict with `CONTROL-C004/C005`, which this report requires the implementation to enforce. The implementation does not add a special widening exception to make these fixtures pass. `V298` subsequently reports `MessageNotFoundError` from its exception-context path after the invalid write; its original same-context expectation is not satisfied. These fixture conflicts need owner resolution separately from this repair.

`CONTROL-V322` exposed a genuine preparation regression: a property assignment's result was inferred from its RHS even though the setter returns a different type. That inference was corrected, and the untouched vector now passes. Property/index setter result types remain unknown when the analyzer cannot establish them; ordinary storage assignments retain their stored-value type.
