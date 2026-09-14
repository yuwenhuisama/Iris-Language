# Module Candidate Substrate

`ClassRegistry` coordinates staged Classes and Modules. Module owners remain real
`ModuleId`s; `ModuleRevisionId` is a separate, runtime-issued identity space.

## Backend Use

1. Reserve an unpublished Module with `stage_module_origin`, or join an existing
   target with `begin_module_transaction`. Repeated begins reuse its candidate.
2. Install static origin members with `stage_module_origin_method`. Source open
   additions use `stage_module_declaration_method`, which permits declared public
   visibility while checking MethodSet authority and composition collisions. Transformations
   instead use `stage_module_method` for private additions and
   `stage_module_method_body` for local compatible replacements. The latter creates
   a new Method ID without changing visibility. `staged_module` is a read-only
   backend view, not ordinary reflection or a private-access grant.
3. Validate complete callable signatures, native admission, and Contract obligations
   in the backend. Runtime Method bodies are opaque handles, not signature metadata.
4. Compose reserved Modules into a new Class with
   `stage_class_origin(spine, published_superclass, &[CompositionEdge::new(module, private_access)])`,
   or use `begin_transaction(class)` and `recompose_candidate(class, module, true)`
   for an existing Class. The latter adds an ordinary edge, never a private grant.
   Checked Class methods use `publish_method`, `publish_decorated_method`, or
   `publish_candidate_decorated_method`; static members use `publish_origin_method`.
5. Call `commit_structural_group` at the outermost transaction boundary. The receipt
   contains both revision sets and one shared commit ID. Use the Runtime-level
   entry point when provisional Class raw-ivar storage or shared cells participate.
   `assign_candidate_class_raw_ivar` and the candidate Class-variable APIs stage
   open-state changes without modifying active storage. Origin-only APIs retain
   their stricter unpublished-owner contract. Storage roots record their base
   revision so bypassing the Runtime commit route cannot promote stale payloads.
6. On a body/admission/staging error, abort with `roll_back_group`. Commit errors
   already discard both candidate sets. Never publish either half with a legacy
   Class-only commit API while a mixed group is open.

Validation and identity-capacity checks precede installation. Failed commits consume
neither revision IDs nor commit IDs. Reserved Module and Method IDs are not reused;
candidate-owned Method artifacts are removed on abort or failed mixed publication.
Published and historical Method artifacts are retained. Backend body artifacts,
captures, and external side effects remain backend responsibilities.
An empty commit returns `commit_id: None`. Ordinary Module lookup and host dispatch
remain active-only. Retained Method IDs preserve their original bodies.

## Scope And Limits

- Module-origin composition can refer to other staged Module origins. Commit checks
  the complete graph for unknown owners, cycles, final capability denials, and
  transformation-addition collisions introduced by later composition.
- Existing Module composition and all post-reservation policy edits explicitly
  reject. This avoids silently invalidating previously stored host MROs and policy.
- Class origins and existing Class candidates can compose provisional Modules in
  the same mixed commit, including transitive staged Module components. A borrowed,
  transaction-only resolver uses candidate Modules before active Modules; it never
  installs temporary public revisions or clones the registry. Class staging and
  checked method mutations use this resolver. Publication recomputes final MROs
  and policies and rechecks recorded checked method/recomposition requirements.
  Late Module policy denials, cycles, and unknown edges discard both candidate sets.
- Class superclasses must still be published when the origin is staged; unpublished
  Class parents are rejected separately. A staged origin's `mro()` is a staging-time
  snapshot, not the final graph. `require_candidate_meta_capability` recomputes the
  current candidate policy without granting ordinary reflection authority.
- Private composition edges grant host-private access only after successful commit.
  Ordinary Class/Module lookup, dispatch, and reflective Method entry remain
  active-only. Legacy Class-only publication APIs are not mixed-group commit APIs.
- No canonical declaration replay, opaque application provenance, rollback rebuild,
  wrapper execution, parser integration, or backend adoption is included.
- Legacy `define_module*` and `define_module_method` retain immediate, unchecked
  installation semantics. They now retain immutable Module history with commit
  zero (outside coordinated publication); changing a legacy slot invalidates any
  staged candidate based on its prior revision. New mixed publications use the
  coordinator's commit sequence without legacy Module calls consuming that sequence.

Run the executable API probe with `cargo run -p iris-runtime --example module_candidate_probe`.
