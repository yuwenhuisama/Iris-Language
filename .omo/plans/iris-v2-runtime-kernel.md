# Iris v1 Milestone 2: Runtime Object Kernel

**Status: DRAFT — awaiting owner approval. No implementation may begin until the four open decisions below are settled.**

Branch `new-iris-dev`. Prerequisite: milestone 1 complete, accepted, and pushed (`9247daf`).

---

## 1. Why this milestone is scoped as a vertical slice

Chapter 03 cannot be implemented as a standalone checklist. Its own concepts — methods, bound methods, closures, construction, dispatch, missing-message routing — cannot be observed at all with the milestone-1 `iris-eval`, which evaluates literals only. Any attempt to "finish chapter 03" in isolation produces either an unvalidated object model or silent spillover into chapters 04-06.

Therefore milestone 2 is defined as a **runtime object kernel**: enough of chapter 03, plus the minimum chapter 04 execution machinery, to run real message-send programs end to end.

## 2. Goal statement

Milestone 2 delivers a runtime object kernel in which executable Iris source can create and interact with runtime objects through dynamic message send, with logical Class identity, active Class revisions, deterministic MRO-based lookup, method and bound-method invocation, construction, object identity/equality/truthiness/identity-hash behavior, and the built-in numeric model — sufficient to execute the classified executable subset of chapter 03.

The milestone produces honest conformance evidence for chapter 03, including explicit non-executable classifications. It does **not** claim chapter 03 completion.

## 3. Acceptance target

Derived from `conformance/iris-v1/RUNTIME-classification.md`, which classifies all 109 unique chapter-03 vectors. Counts independently verified.

| Bucket | Count |
| --- | ---: |
| executable | 60 |
| needs-subsystem | 28 |
| no-fixture | 18 |
| differential | 3 |
| **Total** | **109** |

**Target:** `passed: 60, failed: 0, needs_subsystem: 28, no_fixture: 18, differential: 3`.

This number was derived by classifying every vector **before** setting the target, not after. In milestone 1 the target `35 passed` was set before anyone checked whether the frozen rows were executable; 9 of 41 turned out to be prose and the target had to be renegotiated mid-milestone. That failure mode is structurally prevented here.

The runner MUST report each bucket separately and MUST NOT collapse non-executable vectors into a flattering aggregate.

## 4. Architecture constraints

These are load-bearing. Violating any one forces a rewrite rather than an extension.

### 4.1 Transactional class mutation is designed in from the first runtime commit

The legitimate simplification is "revision-shaped data model now, minimal validation strategy now" — **not** "mutable classes now, revisions later". Dispatch MUST always read through `LogicalClass.active`, never through mutable class fields. Retrofitting this later would touch dispatch, method tables, MRO, handles, and the native boundary simultaneously.

```text
LogicalClass { id: ClassId, active: RevisionId }
ClassRevision { id, parent, modules, methods, properties, class_vars, mro, static_spine_fingerprint }
```

Mutation path: derive `CandidateRevision` from active → apply edits → validate → persist → atomically swap `active`. Invalid candidates MUST NOT partially mutate live state (`IRIS-V1-IDENTITY-C011`).

### 4.2 GC-readiness is mandatory; GC itself is a non-goal

Objects are allocated through one runtime-owned heap/table and addressed by **opaque IDs**, never raw Rust references. `Rc<RefCell<_>>` may appear only as a short-lived detail inside the runtime table, never as the architecture visible to values, dispatch, or the ABI. Evaluator frames hold roots explicitly.

Milestone 2 programs are short-lived test fixtures; not reclaiming memory is acceptable. Collection lands in a later milestone.

### 4.3 Identity hash is decoupled from storage location from day one

`IRIS-V1-RUNTIME-C088`: an object MUST retain its identity hash for its whole lifetime **and across GC movement**, and the hash MUST NOT expose a raw memory address. Every identity-bearing object therefore carries a stable ID independent of where it is stored. Adding this after the fact would perturb every hashing vector.

### 4.4 No panics as language semantics

`IRIS-V1-IDENTITY-C022` forbids exposing frozen-language failures as Rust panics, C++ exceptions crossing the C ABI, assertions, aborts, or undefined behavior. Recoverable Iris failures MUST be typed errors, never `panic!`/`unwrap`/`expect` in production paths.

### 4.5 Object model is decoupled from execution strategy

The interpreter's evaluation details MUST NOT leak into the object table. This preserves the later `frontend → IR → {interpreter | JIT}` split. `IRIS-V1-IDENTITY-C020` requires a JIT to deoptimize on dependency change, which is only possible with an interpreter as fallback — so interpreter-first is a structural prerequisite, not a preference.

## 5. Build order

Derived from the dependency grouping in `conformance/iris-v1/RUNTIME-classification.md`.

| # | Capability | Executable vectors |
| ---: | --- | --- |
| 1 | value foundation | `V018`, `V019`, `V070` |
| 2 | numeric model | `V035`, `V036`, `V039`-`V043`, `V046`-`V048`, `V056`-`V063`, `V065`, `V067`, `V069`, `V071`, `V104`, `V112` |
| 3 | stable hashing | `V001`-`V010`, `V036`, `V071` |
| 4 | logical Class + active revision | `V011`, `V012`, `V013`, `V014`, `V017`, `V026`, `V083`, `V106` |
| 5 | dispatch/selector | `V016`, `V055`, `V061`, `V074`, `V084`, `V094`, `V095`, `V103`, `V109` |
| 6 | MRO/module composition | `V015`, `V089`, `V090` |
| 7 | construction lifecycle | `V026`, `V083` |
| 8 | properties/ivars/class-vars | `V070`, `V089`, `V096`, `V103` |
| 9 | visibility/super | `V093` |
| 10 | equality/ordering/hashing/identity | `V001`-`V010`, `V018`, `V019`, `V035`, `V036`, `V071`, `V094`, `V109` |
| 11 | truthiness/missing-message | `V097` |
| 12 | meta safety | `V108` |

Repeated IDs denote genuine dependencies, not extra vectors.

Crate boundary decision required at step 1: whether the runtime lives in a new `iris-runtime` crate or initially inside `iris-eval`. Recommendation: a new `iris-runtime` crate, because `iris-eval`'s current literal-only surface is a different responsibility.

## 6. Non-goals

Explicitly out of scope. Vectors requiring these are already classified `needs-subsystem` and are not counted.

1. Full chapter 03 completion — 28 vectors need subsystems outside the kernel.
2. Full chapter 04 conformance; only the minimum callable execution needed to run kernel vectors.
3. Collections library (Hash/Array containers), which alone blocks a substantial share of the 28.
4. Contracts, generics, static type system, Contract-qualified dispatch beyond correctly-shaped placeholder behavior.
5. Async/Task, packages, metaprogramming, serialization, reflection APIs, resource quotas.
6. FFI exports and the C Host ABI. `crates/iris-abi` stays empty; the kernel must merely not preclude it.
7. Real GC. Allocation must be GC-ready; collection is deferred.
8. JIT, dispatch caches, and any performance optimization beyond correctness.
9. Authoring fixtures for the 18 `no-fixture` rows — that fabricates conformance evidence.
10. Any edit to `spec/`, or weakening/reclassifying vectors to improve counts.

## 7. Process rules

Carried forward from milestone 1, where each was learned the hard way.

- `spec/` is frozen. Defects are recorded in `docs/spec-defects-v1.md`, never worked around.
- Conformance vectors are evidence. Never weakened, reclassified, or special-cased to move a number.
- A worker that cannot hit a target reports the real number. Fabrication is worse than a low score.
- The conformance runner must observe the real implementation API, never substitute its own rendering. Milestone 1 shipped a runner that graded declaration programs against its own hardcoded strings; a regression test now locks this.
- Every task: `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` all exit 0, and `git status --short -- spec legacy` is empty.
- One atomic commit per task, English plain-sentence message.

## 8. Open decisions — OWNER APPROVAL REQUIRED

| # | Decision | Options | Recommendation |
| ---: | --- | --- | --- |
| 1 | Scope | (A) runtime kernel vertical slice; (B) chapter 03 only; (C) split M2a identity/revisions/dispatch + M2b execution | **(A)** |
| 2 | Acceptance target | 60 passed / 0 failed / 49 non-executable | **adopt** |
| 3 | `num-bigint` dependency | adopt for Integer, keep JSON hand-rolled | **adopt** |
| 4 | Backend | interpreter-first; JIT desktop-only later | **confirm** |

Rationale for 3: the milestone-1 Integer representation is a decimal string, which is unfit once arithmetic, ordering, hashing, and identity are involved. Hand-rolling a correct bignum would compete directly with object-model work for the same time budget and carries real correctness risk. `num-bigint` is mature and does not cross the C ABI, so it cannot affect binary compatibility identity. JSON stays hand-rolled: the existing std-only parser is adequate and replacing it is churn without architectural benefit.

## 9. Escalation triggers

- If more than roughly half the 60 executable vectors turn out to require full chapter 04 control flow or chapter 05 contracts, split formally into M2a and M2b.
- If any chapter-03 row proves internally inconsistent or references an undefined ID, record it in `docs/spec-defects-v1.md` and continue; do not guess.
- If native embedding requirements surface before M2 ends, resist filling `iris-abi`; define internal opaque-handle invariants first and expose the C ABI in its own milestone.

## 10. Amendment: frontend AST gap discovered during execution

Recorded after build-order steps 1-12 landed. Not a change of goal; a correction of a planning omission.

Section 6 listed "full chapter 04 conformance" as a non-goal and kept only "the minimum callable execution needed to run kernel vectors". Executing against the corpus showed that minimum is substantially larger than assumed.

Verified facts:

- `iris-syntax::Expression` carries only `Name`, `Literal`, `Unary`, `Binary`, `Assignment`, `Grouped`. It has no `Array`, `Symbol`, `Call`, or `Member` node.
- `iris-runtime` dispatch resolves a `Method` but exposes no public typed method-body invocation returning a `Value`.
- `iris-runtime` had zero dependents; `iris-eval` still wired only to `iris-lexer`.

Re-classification of the 60 executable chapter-03 vectors by what their input actually requires:

| Requirement | Count |
| --- | ---: |
| declarations or fixtures: class, property, super, mixin | 21 |
| call and member-access syntax | 18 |
| array literals | 4 |
| plain operators, reachable with the current AST | 5 |
| overview-only rows with no coverage fixture | 12 |

Only 5 of 60 are reachable without extending the frontend. The remaining work is genuinely chapter-04 surface, not incidental.

Consequence: the `passed: 60` target from section 3 is still the correct definition of chapter-03 executable coverage, but reaching it requires frontend AST and parser extension that section 5's build order never listed. Milestone 2 therefore needs an additional build step before any vector can be executed:

**Build-order step 13 — frontend expression surface.** Extend `iris-syntax` and `iris-parser` with array literals, Symbols, member access, and call expressions, then expose a typed method-body invocation on `iris-runtime` so `iris-eval` can bridge source to the kernel.

Class and property declaration from source remains deferred; it serves the 21 declaration/fixture vectors and is tracked separately.

## 11. Amendment correction: frontend gap closed

Recorded after build-order step 13 landed. Section 10's table said only 5 of the 60 executable vectors were reachable without frontend expansion. **That figure is now obsolete and MUST NOT be cited as current state.** It described the tree before the frontend work.

Landed since section 10 was written:

| Commit | Change |
| --- | --- |
| `53e0d88` | Array, Symbol, Member, ContractView, Call AST nodes; identifier-digit lexer fix |
| `bfb09d5` | Named infix retains its selector; `same?` parses as a distinct Identity operator |
| `275c89b` | Evaluator bridged to the runtime kernel via `iris_eval::evaluate` |
| `cdf9822` | Numeric tokens terminate at member-access dots |
| `ba873c6` | Slash after a numeric literal lexes as division, not a Regex opener |
| `8c8d3b4` | Member getters resolve as ordinary sends |
| `0fa156e` | `<` family longest match and compound assignments lex as complete tokens |

Measured directly with a probe over 21 sampled `executable` vectors:

- 17 evaluate to their expected value
- 2 correctly raise the expected typed error, `V060` DivisionByZero and `V069` Range
- 2 remain genuinely blocked: `V016` needs an ordinary-object fixture, `V048` needs the `Float64(-Infinity)` construction form

So 19 of 21 sampled rows already behave per spec. The remaining barrier to reporting a chapter-03 number is that no RUNTIME vector corpus or runner chapter exists yet, not frontend capability.
