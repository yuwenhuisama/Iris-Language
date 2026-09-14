# VM Does Not Observe a User-Defined to_bool Override

Status: Open observation on the tested binary; recheck after a fresh build before repair.

Local tracking ID: `VM-TO-BOOL-01` (not a specification or conformance-vector ID).

## Summary

A user Class overrides `to_bool() -> Bool` to print a marker and return `false`.
The tested VM neither prints that marker on a direct call nor selects the false
branch when the instance is used in an `if` condition. The reference evaluator
executes the override in both cases and selects the expected branch.

This is not only an `if`-lowering observation: an isolated explicit method call
also returns the wrong value. The internal cause has not been investigated.
The output does not establish whether the override was lost during declaration,
skipped during lookup, bypassed by an intrinsic, or affected by another path.

## Environment and Provenance

- Platform: macOS.
- Existing executable: `./target/debug/iris`, version output `iris 0.1.0`.
- Source HEAD during this report's verification:
  `16b1b1a3320409318638dbccea649533718916bc`.
- No rebuild was performed for this report. Concurrent uncommitted changes exist
  in builtins, lexer/parser, evaluator, runtime, VM, CLI, and specification files.
  The executable is **not attributed to that HEAD or to the current dirty tree**.
- The initial article 03 observation occurred while source HEAD was `a422054`,
  also using an existing executable. This report freshly reran the programs below.
- Each probe used Node `spawnSync` with a 15000ms timeout, passing the source
  through `-e`. All eight executions completed with exit 0, empty stderr, and
  no terminating signal. A successful process exit is not semantic correctness.

## Normative Basis

- [IRIS-V1-RUNTIME-C009](../spec/iris-v1/03-runtime-object-model.md): ordinary
  instance sends consult the logical Class's active revision.
- [IRIS-V1-RUNTIME-C046](../spec/iris-v1/03-runtime-object-model.md): lookup starts
  at the Class's own active members before composed Modules and superclass MRO.
- [IRIS-V1-RUNTIME-C093](../spec/iris-v1/03-runtime-object-model.md): truth testing
  evaluates the operand once, sends `to_bool` once, requires an actual Bool,
  and does not recursively convert the result.
- [IRIS-V1-RUNTIME-C094](../spec/iris-v1/03-runtime-object-model.md): root Object
  initially returns true; Nil returns false; Bool returns itself. The root
  default must not supersede a valid user override.
- [IRIS-V1-RUNTIME-C095](../spec/iris-v1/03-runtime-object-model.md): raised values
  propagate; non-Bool normal results cause `TypeContractError`; dependent
  branches, RHS expressions, and writeback must not run after a failed test.
- [IRIS-V1-CONTROL-C039/C040](../spec/iris-v1/04-bindings-callables-control-flow.md):
  conditions use the dynamic protocol; logical operations retain their specified
  short-circuit and operand-return behavior.

The reference result below matches the relevant rule for this program. It is
not the source of the expected semantics and is not a blanket reference-backend
conformance claim.

## Main Reproducer

Run from the repository root in a Unix-style shell. These commands perform
independent executions of identical source:

```bash
SOURCE='class Gate {
  public override fun to_bool() -> Bool {
    print("check gate")
    false
  }
}
let gate = Gate.new()
print(gate.to_bool())
if gate {
  print("then")
} else {
  print("else")
}'

./target/debug/iris --vm -e "$SOURCE"
./target/debug/iris -e "$SOURCE"
```

Expected stdout, including the final newline:

```text
check gate
false
check gate
else
```

The first marker belongs to the explicit call. The second belongs to the single
truth test in `if`; this is not two protocol calls by one condition.

Actual VM stdout:

```text
true
then
```

Actual reference stdout matches the expected output. Both exit 0 with empty stderr.

## Isolated Reproducers

Direct call only:

```bash
SOURCE='class Gate { public override fun to_bool() -> Bool { print("check gate"); false } }; print(Gate.new().to_bool())'
./target/debug/iris --vm -e "$SOURCE"
./target/debug/iris -e "$SOURCE"
```

Condition only:

```bash
SOURCE='class Gate { public override fun to_bool() -> Bool { print("check gate"); false } }; if Gate.new() { print("then") } else { print("else") }'
./target/debug/iris --vm -e "$SOURCE"
./target/debug/iris -e "$SOURCE"
```

| Probe | Expected stdout | VM stdout | Reference stdout |
| --- | --- | --- | --- |
| Direct call | `check gate\nfalse\n` | `true\n` | `check gate\nfalse\n` |
| Condition only | `check gate\nelse\n` | `then\n` | `check gate\nelse\n` |

Both backends exit 0 with empty stderr in both probes. Isolating the condition
rules out the preceding explicit call as a necessary trigger.

## Passing Control

```bash
SOURCE='class DefaultGate {}
print(DefaultGate.new().to_bool())
print(false.to_bool())
print(true.to_bool())
module Probe {
  public module fun rhs() -> String {
    print("run rhs")
    "fallback"
  }
}
print(false && Probe.rhs())
print(true || Probe.rhs())
print(false || Probe.rhs())
print(!true)'

./target/debug/iris --vm -e "$SOURCE"
./target/debug/iris -e "$SOURCE"
```

Both backends exit 0, stderr is empty, and stdout is:

```text
true
false
true
false
true
run rhs
fallback
false
```

The control verifies the inherited default, direct Bool methods, and these
Bool-based logical operations. It does not prove custom-object logical operators
or every builtin truth conversion correct.

## Impact and Limits

- Demonstrated: explicit invocation returns true instead of false and skips the
  user marker; `if` chooses the opposite branch and also skips the marker.
- Application decisions relying on this override can therefore take the wrong
  path. No concrete authorization bypass or other security exploit was tested.
- `while`, match guards, logical assignment, custom-object `&&`/`||`/`!`,
  inherited user overrides, reopened overrides, exceptions from `to_bool`,
  non-Bool returns, and method-missing fallback were not tested for this report.
- No claim about all VM dispatch, all truthiness, performance, or memory safety.
- Article 04's Module/Contract observations and the separately repaired signature
  and local-binding defects are different reports; shared causes are not assumed.

## Repair Handoff and Acceptance Criteria

1. Build the intended revision in an isolated worktree/target directory, or
   coordinate with active builds. Record revision, local changes, and binary
   provenance. Do not remove another process's lock or discard concurrent work.
2. Reproduce the main and isolated failures before changing code. Add regression
   assertions for values, marker counts, and branch selection, not just exit code.
3. Make direct invocation execute the user override once and return false;
   make the isolated condition execute it once and select else. The combined
   program must produce exactly the expected four lines on both backends.
4. Preserve the passing control. Do not fix custom overrides by breaking Object's
   default, Bool behavior, short-circuit evaluation, or operand-return semantics.
5. Audit registration, ordinary lookup, and specialized conversion paths to locate
   the actual cause. Do not assume that patching `if` alone addresses direct calls.
6. Add targeted coverage for inherited/reopened overrides and the other truth-test
   consumers as appropriate to the fix. Check single evaluation, exception
   propagation, non-Bool rejection, and suppression of dependent work against the
   owning clauses. These are requested coverage, not additional proven failures.
7. Do not weaken tests, bypass the override, or change frozen semantics to make
   the outputs agree. Backend agreement alone is not sufficient.

No runtime, specification, or automated test changes were made for this report.
No commit, push, or external issue publication was performed.
