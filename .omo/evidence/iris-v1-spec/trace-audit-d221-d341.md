# Trace Audit D-221..D-341

Scope: audit only. No specification files are edited by this evidence file.

Rules applied:

- Every decision from `D-221` through `D-341` appears exactly once.
- Broad cluster vectors `IRIS-V1-META-V047` and `IRIS-V1-META-V048` are rejected unless their scenario text directly names the audited behavior.
- Revision-event decisions use async clauses and async vectors.
- Contract decisions use type and runtime clauses, not package or meta cluster rows alone.
- `FIX` rows are machine-applicable instructions for the downstream writer.

## Counts

| Metric | Count |
| --- | ---: |
| Expected D-ID rows | 121 |
| Actual D-ID rows | 121 |
| Duplicate D-IDs | 0 |
| Missing D-IDs | 0 |
| `OK` rows | 46 |
| `FIX` rows | 75 |
| Current matrix rows citing broad `META-V047` or `META-V048` | 116 |
| Rows requiring async coverage | 6 |
| Contract rows requiring types/runtime coverage | 9 |

## Machine-Applicable Audit Table

| D-ID | Audit | Exact clause anchors | Direct coverage or minimal vector spec |
| --- | --- | --- | --- |
| `D-221` | `FIX` | `IRIS-V1-META-C053`, `IRIS-V1-META-C054`, `IRIS-V1-META-C081` | Replace broad `META-V047` with direct `IRIS-V1-META-V021` plus `ADD META-V049`: static `mixin A, B` and programmatic `include(A); include(B)` produce the same Module composition edge list, MRO, and validation failures. |
| `D-222` | `FIX` | `IRIS-V1-META-C022`, `IRIS-V1-META-C034`, `IRIS-V1-META-C042` | Replace broad `META-V047` with `ADD META-V050`: origin Class and Module bodies stage members, validate, publish atomically on success, and publish nothing when body execution raises. |
| `D-223` | `FIX` | `IRIS-V1-META-C023`, `IRIS-V1-META-C024` | Replace broad `META-V047` with `ADD META-V051`: in origin and open bodies, `self.define_method` targets the candidate while unqualified calls resolve locals first, then privileged `self` or Module `main`. |
| `D-224` | `OK` | `IRIS-V1-META-C026`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V007`: origin body local is not captured by a declared Method. |
| `D-225` | `FIX` | `IRIS-V1-META-C027`, `IRIS-V1-META-C113` | Keep `IRIS-V1-META-V007` for the declared-Method case and add `META-V052`: Closure escaping from a Class or Module body captures the body local, but no property, Module state, or storage slot is created. |
| `D-226` | `OK` | `IRIS-V1-META-C028`, `IRIS-V1-META-C046`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V008`: conditional body meta definition is dynamic-only reflection metadata. |
| `D-227` | `OK` | `IRIS-V1-META-C045`, `IRIS-V1-META-C016`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V009` and `IRIS-V1-META-V010`: direct import sees exported static extension, transitive re-export does not. |
| `D-228` | `OK` | `IRIS-V1-META-C048`, `IRIS-V1-META-C052`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V019` and `IRIS-V1-META-V020`: incompatible replacement fails, compatible authorized later import wins. |
| `D-229` | `OK` | `IRIS-V1-META-C019`, `IRIS-V1-META-C048`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V020`: later direct import wins deterministically by source order. |
| `D-230` | `OK` | `IRIS-V1-META-C049`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V020`; add no broad-only evidence. |
| `D-231` | `OK` | `IRIS-V1-META-C050`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V018`: missing `override` on same-Module replacement is rejected. |
| `D-232` | `FIX` | `IRIS-V1-META-C050`, `IRIS-V1-TYPES-C046`, `IRIS-V1-TYPES-C053` | Add `META-V053`: inherited member and composed Module member replacement both require `override`; replacing a Contract implementation also requires `impl` when applicable. |
| `D-233` | `OK` | `IRIS-V1-TYPES-C046`, `IRIS-V1-TYPES-C053` | Direct type coverage exists in `IRIS-V1-TYPES-V018`; retain `IRIS-V1-RUNTIME-V050` only as runtime support. |
| `D-234` | `OK` | `IRIS-V1-TYPES-C047`, `IRIS-V1-TYPES-C053` | Direct type coverage exists in `IRIS-V1-TYPES-V018`; retain `IRIS-V1-RUNTIME-V050` only as runtime support. |
| `D-235` | `FIX` | `IRIS-V1-TYPES-C040`, `IRIS-V1-RUNTIME-C023`, `IRIS-V1-TYPES-C053` | Replace broad meta evidence with `IRIS-V1-TYPES-V018` plus `ADD RUNTIME-V052`: same selector with different static argument Types does not select overloads and uses one ordinary Method identity. |
| `D-236` | `OK` | `IRIS-V1-TYPES-C049`, `IRIS-V1-TYPES-C053`, `IRIS-V1-GRAMMAR-C049` | Direct type coverage exists in `IRIS-V1-TYPES-V018`: `(value as C)..m` uses the qualified Contract slot. |
| `D-237` | `OK` | `IRIS-V1-TYPES-C032`, `IRIS-V1-TYPES-C049`, `IRIS-V1-TYPES-C053` | Direct type coverage exists in `IRIS-V1-TYPES-V018`: Contract view is explicit and qualified call remains explicit. |
| `D-238` | `OK` | `IRIS-V1-TYPES-C049`, `IRIS-V1-TYPES-C053`, `IRIS-V1-RUNTIME-C022` | Direct type coverage exists in `IRIS-V1-TYPES-V018`: ordinary dot and qualified Contract slots stay separate. |
| `D-239` | `OK` | `IRIS-V1-TYPES-C050`, `IRIS-V1-TYPES-C053` | Direct type coverage exists in `IRIS-V1-TYPES-V018`: Contract view identity, equality, and `same?` behavior are covered. |
| `D-240` | `OK` | `IRIS-V1-TYPES-C050`, `IRIS-V1-TYPES-C053` | Direct type coverage exists in `IRIS-V1-TYPES-V018`: identity-less receiver equality participates in Contract view equality. |
| `D-241` | `OK` | `IRIS-V1-TYPES-C051`, `IRIS-V1-RUNTIME-C147`, `IRIS-V1-TYPES-C053` | Direct type/runtime coverage exists in `IRIS-V1-TYPES-V018` and exact hash vector clause `IRIS-V1-RUNTIME-C147`. |
| `D-242` | `FIX` | `IRIS-V1-TYPES-C052`, `IRIS-V1-META-C004`, `IRIS-V1-TYPES-C077` | Replace broad `META-V047` with `ADD TYPES-V020`: same named Contract Type hash stays stable across allocation and source path changes, and changes with API major or closed arguments. |
| `D-243` | `OK` | `IRIS-V1-META-C003`, `IRIS-V1-META-C004`, `IRIS-V1-META-C010` | Direct coverage exists in `IRIS-V1-META-V046`: manifest identity and package corpus cover package ID plus API major. |
| `D-244` | `FIX` | `IRIS-V1-META-C004`, `IRIS-V1-META-C005`, `IRIS-V1-META-C010` | Keep `META-V046`; add `META-V054`: package `1.2.0` and `1.2.1` under the same `api_major` share nominal Type identity, while a different API major does not. |
| `D-245` | `FIX` | `IRIS-V1-META-C005`, `IRIS-V1-META-C010` | Keep `META-V046`; add `META-V055`: attempting to load two package implementations with the same `(package_id, api_major)` selects one active revision or aborts, with no hidden load-context identity. |
| `D-246` | `OK` | `IRIS-V1-META-C068`, `IRIS-V1-META-C070`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V040` and `IRIS-V1-META-V041`: compatible upgrade commits, failed migration leaves old package active. |
| `D-247` | `FIX` | `IRIS-V1-META-C062`, `IRIS-V1-RUNTIME-C019`, `IRIS-V1-META-C113` | Add `META-V056`: old package revision remains loaded while retained by an entered frame or Method object, then becomes reclaimable after references cease. |
| `D-248` | `OK` | `IRIS-V1-META-C069`, `IRIS-V1-META-C068`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V040` and `IRIS-V1-META-V041`; keep only concrete upgrade state and hook scenarios. |
| `D-249` | `FIX` | `IRIS-V1-META-C042`, `IRIS-V1-META-C070` | Add `META-V057`: upgrade hook performs an external log write then raises; Iris rollback preserves old package state but does not undo the external log. |
| `D-250` | `FIX` | `IRIS-V1-META-C041`, `IRIS-V1-META-C068`, `IRIS-V1-META-C113` | Add `META-V058`: same-major package upgrade reaches a global safepoint and publishes all affected revisions atomically, with no mixed old/new observation. |
| `D-251` | `FIX` | `IRIS-V1-META-C041`, `IRIS-V1-META-C043`, `IRIS-V1-META-C113` | Add `META-V059`: structural open commit pauses at safepoint, publishes all targets atomically, and no observer sees a partial mix. |
| `D-252` | `OK` | `IRIS-V1-META-C037`, `IRIS-V1-ASYNC-C018`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V014` and `IRIS-V1-ASYNC-V013`: `await` in open authority is rejected. |
| `D-253` | `OK` | `IRIS-V1-META-C038`, `IRIS-V1-META-C043`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V017`: nested opens form one transaction group and one safepoint commit. |
| `D-254` | `OK` | `IRIS-V1-META-C039`, `IRIS-V1-META-C043`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V015`: overlapping concurrent opens produce one commit and one `MetaTransactionConflictError`. |
| `D-255` | `FIX` | `IRIS-V1-META-C040`, `IRIS-V1-META-C021` | Add `META-V060`: after `MetaTransactionConflictError`, runtime performs no automatic retry, rebase, merge, or hidden block re-execution. |
| `D-256` | `OK` | `IRIS-V1-META-C036`, `IRIS-V1-META-C043`, `IRIS-V1-RUNTIME-C009` | Direct coverage exists in `IRIS-V1-META-V016`: ordinary instance send inside open uses published active revision. |
| `D-257` | `FIX` | `IRIS-V1-META-C036`, `IRIS-V1-META-C043` | Add `META-V061`: staged candidate Method is visible through transaction metadata but cannot be invoked on a candidate instance preview because v1 has none. |
| `D-258` | `OK` | `IRIS-V1-RUNTIME-C009`, `IRIS-V1-RUNTIME-C011`, `IRIS-V1-META-C043` | Direct runtime coverage exists in `IRIS-V1-RUNTIME-V050`: instances use logical Class active revision after structural commit. |
| `D-259` | `FIX` | `IRIS-V1-RUNTIME-C056`, `IRIS-V1-META-C043` | Add `RUNTIME-V053`: construction snapshots the active revision for allocation and `initialize`, then the completed instance uses the logical Class for later sends. |
| `D-260` | `FIX` | `IRIS-V1-RUNTIME-C059`, `IRIS-V1-META-C043` | Add `RUNTIME-V054`: no `new_current`, constructor retry, automatic instance migration, or hidden post-construction reconciliation surface exists. |
| `D-261` | `FIX` | `IRIS-V1-RUNTIME-C009`, `IRIS-V1-TYPES-C045`, `IRIS-V1-TYPES-C053` | Add `RUNTIME-V055`: Class reopen changes active revision but nominal Type identity and Contract conformance identity remain stable. |
| `D-262` | `FIX` | `IRIS-V1-RUNTIME-C009`, `IRIS-V1-TYPES-C028`, `IRIS-V1-TYPES-C079` | Add `RUNTIME-V056`: after runtime superclass change within bounds, `is`, `Type#subtype?`, and ancestry checks consult the current active revision. |
| `D-263` | `FIX` | `IRIS-V1-META-C002`, `IRIS-V1-META-C081`, `IRIS-V1-RUNTIME-C157` | Add `META-V062`: meta transaction changes structure only through declared capability checks, while ordinary dispatch and type algebra remain owned by runtime and type rules. |
| `D-264` | `FIX` | `IRIS-V1-RUNTIME-C060`, `IRIS-V1-IDENTITY-C115`, `IRIS-V1-IDENTITY-C116` | Add `RUNTIME-V057`: application may call `migrate_revision` explicitly on tracked instances, but runtime commit does not enumerate or auto-migrate live instances. |
| `D-265` | `FIX` | `IRIS-V1-RUNTIME-C060`, `IRIS-V1-META-C024` | Add `RUNTIME-V058`: `migrate_revision` is resolved as an ordinary dynamic message and can be replaced or missing like other selectors. |
| `D-266` | `FIX` | `IRIS-V1-RUNTIME-C016`, `IRIS-V1-RUNTIME-C060`, `IRIS-V1-META-C097` | Add `RUNTIME-V059`: migration receives read-only `ClassRevision` metadata; attempts to mutate or reactivate it fail. |
| `D-267` | `OK` | `IRIS-V1-META-C061`, `IRIS-V1-RUNTIME-C017`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V040` plus runtime revision corpus: successful publications consume per-Class revision numbers only. |
| `D-268` | `FIX` | `IRIS-V1-META-C061`, `IRIS-V1-ASYNC-C046`, `IRIS-V1-META-C113` | Add `META-V063`: multi-target structural commit receives one runtime-wide monotonic `commit_id`, and failed candidates consume none. |
| `D-269` | `FIX` | `IRIS-V1-META-C062`, `IRIS-V1-META-C063`, `IRIS-V1-RUNTIME-C019` | Add `META-V064`: once old executable revision state is unpinned, heavy state is reclaimed while lightweight audit metadata remains queryable. |
| `D-270` | `OK` | `IRIS-V1-META-C064`, `IRIS-V1-RUNTIME-C020`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V042` for rollback failure and `META-V047` directly names rollback. |
| `D-271` | `OK` | `IRIS-V1-META-C066`, `IRIS-V1-META-C064`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V042`: digest mismatch raises `RevisionArtifactUnavailableError` and publishes nothing. |
| `D-272` | `FIX` | `IRIS-V1-META-C067`, `IRIS-V1-META-C066`, `IRIS-V1-META-C113` | Add `META-V065`: revision artifact manifest digest changes on semantic source or native artifact change, but not on locator byte change. |
| `D-273` | `FIX` | `IRIS-V1-META-C065`, `IRIS-V1-RUNTIME-C021`, `IRIS-V1-META-C064` | Add `META-V066`: rollback to historical artifact that lacks a later required Contract member fails and publishes no revision. |
| `D-274` | `FIX` | `IRIS-V1-RUNTIME-C021`, `IRIS-V1-META-C065`, `IRIS-V1-META-C068` | Add `META-V067`: same-major release adding compatible ordinary static API succeeds, but removing current required API fails validation. |
| `D-275` | `FIX` | `IRIS-V1-TYPES-C042`, `IRIS-V1-META-C029`, `IRIS-V1-TYPES-C053` | Replace broad meta/runtime evidence with `ADD TYPES-V021`: Contract body with Method body, stored state, initializer, raw ivar, or open operation is rejected. |
| `D-276` | `FIX` | `IRIS-V1-TYPES-C043`, `IRIS-V1-GRAMMAR-C049`, `IRIS-V1-TYPES-C041` | Add `TYPES-V022`: Contract `C extends A, B` merges compatible requirements and rejects incompatible same-name requirements unless qualified slots are used. |
| `D-277` | `OK` | `IRIS-V1-TYPES-C041`, `IRIS-V1-GRAMMAR-C049`, `IRIS-V1-GRAMMAR-C046` | Direct grammar coverage exists in `IRIS-V1-GRAMMAR-V012`; retain type clause for Contract inheritance semantics. |
| `D-278` | `OK` | `IRIS-V1-TYPES-C044`, `IRIS-V1-GRAMMAR-C049`, `IRIS-V1-MIG-009` | Direct grammar coverage exists in `IRIS-V1-GRAMMAR-V012`; type clause owns Class `for` conformance. |
| `D-279` | `FIX` | `IRIS-V1-TYPES-C044`, `IRIS-V1-GRAMMAR-C049`, `IRIS-V1-GRAMMAR-C013` | Add `TYPES-V023`: Module or Contract using `for` as instance conformance is rejected; Class using `for` is accepted. |
| `D-280` | `OK` | `IRIS-V1-GRAMMAR-C052`, `IRIS-V1-GRAMMAR-C049` | Direct grammar coverage exists in `IRIS-V1-GRAMMAR-V012`: Class header clauses parse only in canonical order. |
| `D-281` | `OK` | `IRIS-V1-GRAMMAR-C052`, `IRIS-V1-GRAMMAR-C049` | Direct grammar coverage exists in `IRIS-V1-GRAMMAR-V012`: Module and Contract header clauses parse only in canonical order. |
| `D-282` | `FIX` | `IRIS-V1-TYPES-C055`, `IRIS-V1-GRAMMAR-C049`, `IRIS-V1-GRAMMAR-C015` | Add `GRAMMAR-V015`: `where T: A & B, U: C` parses as two constraint assignments; comma never means intersection. |
| `D-283` | `FIX` | `IRIS-V1-GRAMMAR-C049`, `IRIS-V1-TYPES-C009`, `IRIS-V1-TYPES-C017` | Add `TYPES-V024`: Class without `extends` has `Object` as static superclass bound and runtime root ancestry. |
| `D-284` | `FIX` | `IRIS-V1-TYPES-C041`, `IRIS-V1-TYPES-C056`, `IRIS-V1-META-C053` | Replace broad meta/runtime evidence with `ADD META-V068`: Contract without `extends` has no parents, Module without `mixin` has empty composition, unconstrained Module `Self` uses Object bound. |
| `D-285` | `FIX` | `IRIS-V1-RUNTIME-C148`, `IRIS-V1-RUNTIME-C149`, `IRIS-V1-META-C081` | Add `RUNTIME-V060`: `Nil` Class accepts ordinary Module composition within safety and capability bounds without breaking singleton identity. |
| `D-286` | `FIX` | `IRIS-V1-RUNTIME-C148`, `IRIS-V1-RUNTIME-C149`, `IRIS-V1-META-C082` | Add `RUNTIME-V061`: Bool and numeric Classes may gain or replace compatible dynamic members while primitive identity, immutability, and guards remain valid. |
| `D-287` | `OK` | `IRIS-V1-META-C082`, `IRIS-V1-RUNTIME-C149`, `IRIS-V1-RUNTIME-C154` | Direct coverage exists in `IRIS-V1-RUNTIME-V044`, `IRIS-V1-RUNTIME-V051`, and `IRIS-V1-META-V028`. |
| `D-288` | `FIX` | `IRIS-V1-META-C072`, `IRIS-V1-META-C080`, `IRIS-V1-RUNTIME-C154` | Add `META-V069`: syntax, reflection, Dynamic, native, and Host meta paths all deny the same forbidden operation with no bypass. |
| `D-289` | `FIX` | `IRIS-V1-META-C072`, `IRIS-V1-META-C076`, `IRIS-V1-RUNTIME-C152` | Add `META-V070`: reflected `MetaCapabilities` policy is immutable for a revision and cannot be widened by open blocks or runtime code. |
| `D-290` | `FIX` | `IRIS-V1-META-C074`, `IRIS-V1-META-C075`, `IRIS-V1-GRAMMAR-C052` | Add `GRAMMAR-V016`: `meta deny` is accepted only as static header denial syntax and rejected in executable or conditional body code. |
| `D-291` | `FIX` | `IRIS-V1-META-C073`, `IRIS-V1-META-C081`, `IRIS-V1-META-C113` | Add `META-V071`: every v1 capability name is accepted, an unknown `meta deny` name is rejected, and each category maps to the listed operations. |
| `D-292` | `FIX` | `IRIS-V1-META-C080`, `IRIS-V1-META-C081` | Add `META-V072`: denying `method_set` does not deny `method_body`; an operation requiring both fails when either required capability is missing. |
| `D-293` | `FIX` | `IRIS-V1-META-C081`, `IRIS-V1-META-C080` | Add `META-V073`: each core meta operation in the capability matrix is attempted with present and absent required capabilities, producing success or `MetaCapabilityError`. |
| `D-294` | `FIX` | `IRIS-V1-META-C076`, `IRIS-V1-META-C077`, `IRIS-V1-GRAMMAR-C052` | Add `META-V074`: subclass cannot re-enable a superclass `meta deny`; attempted widening remains denied after open or runtime superclass change. |
| `D-295` | `FIX` | `IRIS-V1-META-C076`, `IRIS-V1-META-C077`, `IRIS-V1-META-C081` | Add `META-V075`: effective capability calculation includes declared superclass chain and current runtime superclass chain denials. |
| `D-296` | `FIX` | `IRIS-V1-META-C078`, `IRIS-V1-META-C081` | Add `META-V076`: including a Module with `meta deny shape` narrows host capabilities, and removing it removes only that Module-origin denial. |
| `D-297` | `FIX` | `IRIS-V1-META-C079`, `IRIS-V1-TYPES-C045`, `IRIS-V1-TYPES-C053` | Add `TYPES-V025`: Class declaring `for Contract` must absorb the Contract `meta deny`; Contract inheritance unions deny requirements. |
| `D-298` | `OK` | `IRIS-V1-GRAMMAR-C052`, `IRIS-V1-META-C075` | Direct grammar coverage exists in `IRIS-V1-GRAMMAR-V012`: `meta` is the final header clause for each admitted declaration kind. |
| `D-299` | `FIX` | `IRIS-V1-RUNTIME-C154`, `IRIS-V1-META-C081`, `IRIS-V1-META-C080` | Add `META-V077`: `method_set` denial blocks adding a Method slot, while compatible body replacement succeeds only when `method_body` is present. |
| `D-300` | `FIX` | `IRIS-V1-RUNTIME-C154`, `IRIS-V1-META-C081`, `IRIS-V1-META-C080` | Add `META-V078`: `property_set` denial blocks property slot changes, while compatible accessor body replacement succeeds only when `property_body` is present. |
| `D-301` | `FIX` | `IRIS-V1-RUNTIME-C154`, `IRIS-V1-META-C081`, `IRIS-V1-META-C080` | Add `META-V079`: class/shared storage slot creation needs `class_state_set`; writing existing class/shared storage needs `class_state_write`. |
| `D-302` | `FIX` | `IRIS-V1-META-C055`, `IRIS-V1-META-C081` | Add `META-V080`: include, remove, reorder, and replace Module composition all require the single `modules` capability with no finer split. |
| `D-303` | `FIX` | `IRIS-V1-META-C083`, `IRIS-V1-META-C081`, `IRIS-V1-RUNTIME-C154` | Add `META-V081`: `shape` denial blocks declared layout changes but does not block undeclared raw ivar expansion unless `instance_state` is denied. |
| `D-304` | `OK` | `IRIS-V1-RUNTIME-C069`, `IRIS-V1-META-C083`, `IRIS-V1-META-C081` | Direct coverage exists in `IRIS-V1-META-V026` and `IRIS-V1-RUNTIME-V051`: absent undeclared raw ivar expansion is denied by `instance_state`. |
| `D-305` | `OK` | `IRIS-V1-META-C084`, `IRIS-V1-RUNTIME-C069`, `IRIS-V1-RUNTIME-V051` | Direct coverage exists in `IRIS-V1-RUNTIME-V029`, `IRIS-V1-RUNTIME-V051`, and `IRIS-V1-META-V039`. |
| `D-306` | `OK` | `IRIS-V1-RUNTIME-C070`, `IRIS-V1-META-C081`, `IRIS-V1-META-C083` | Direct coverage exists in `IRIS-V1-META-V027`: deletion succeeds after `instance_state` denial, recreation fails. |
| `D-307` | `FIX` | `IRIS-V1-RUNTIME-C068`, `IRIS-V1-TYPES-C003`, `IRIS-V1-META-C083` | Add `RUNTIME-V062`: undeclared dynamic ivar stores first an Integer, then a String, with reads typed as `Dynamic<Object>` and no fixed slot Contract. |
| `D-308` | `OK` | `IRIS-V1-RUNTIME-C068`, `IRIS-V1-META-C109`, `IRIS-V1-TYPES-C003` | Direct coverage exists in `IRIS-V1-RUNTIME-V028`: missing undeclared `@x` read returns `nil` and creates no slot. |
| `D-309` | `FIX` | `IRIS-V1-RUNTIME-C071`, `IRIS-V1-RUNTIME-C075`, `IRIS-V1-META-C111` | Add `RUNTIME-V063`: two receivers with `@x` keep separate receiver-name slots, and Class-object `@x` is distinct from hierarchy `@@x`. |
| `D-310` | `FIX` | `IRIS-V1-RUNTIME-C072`, `IRIS-V1-CONTROL-C028` | Add `RUNTIME-V064`: Closure created in an instance Method escapes, later reads and writes the captured receiver raw ivar, and cannot be rebound. |
| `D-311` | `FIX` | `IRIS-V1-RUNTIME-C072`, `IRIS-V1-RUNTIME-C078` | Add `RUNTIME-V065`: escaped Closure keeps the original private receiver capability; reflection, Dynamic, native, and Host paths cannot rebind it. |
| `D-312` | `FIX` | `IRIS-V1-GRAMMAR-C011`, `IRIS-V1-CONTROL-C008`, `IRIS-V1-META-C111` | Add `GRAMMAR-V017`: `@name` is accepted only as current-receiver syntax; `other.@name` is rejected, and external access must use reflection. |
| `D-313` | `FIX` | `IRIS-V1-RUNTIME-C078`, `IRIS-V1-RUNTIME-C079`, `IRIS-V1-META-C056` | Add `RUNTIME-V066`: private Method is callable from lexically authorized declaring Class code, rejected from subclass, external, Dynamic, and ordinary reflection code. |
| `D-314` | `OK` | `IRIS-V1-META-C056`, `IRIS-V1-RUNTIME-C050`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V022` and `IRIS-V1-META-V023`: edge authorization works and self-grant fails. |
| `D-315` | `FIX` | `IRIS-V1-META-C058`, `IRIS-V1-RUNTIME-C051`, `IRIS-V1-META-C113` | Add `RUNTIME-V067`: composed Module Method reads and writes current receiver raw ivars whether or not private Method access is granted. |
| `D-316` | `OK` | `IRIS-V1-META-C057`, `IRIS-V1-RUNTIME-C050`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V022`: private access works before edge removal and is revoked after. |
| `D-317` | `FIX` | `IRIS-V1-META-C059`, `IRIS-V1-RUNTIME-C079`, `IRIS-V1-META-C113` | Add `RUNTIME-V068`: protected Method call succeeds only through hierarchy-authorized implementation receiver rules and does not grant private access. |
| `D-318` | `FIX` | `IRIS-V1-META-C059`, `IRIS-V1-RUNTIME-C071`, `IRIS-V1-RUNTIME-C079` | Add `RUNTIME-V069`: protected property follows hierarchy visibility while raw ivar access remains current-receiver receiver-name storage. |
| `D-319` | `FIX` | `IRIS-V1-RUNTIME-C046`, `IRIS-V1-META-C053`, `IRIS-V1-META-C060` | Add `RUNTIME-V070`: later-composed Module has lookup precedence over earlier-composed Module for the same ordinary selector. |
| `D-320` | `FIX` | `IRIS-V1-RUNTIME-C046`, `IRIS-V1-META-C060`, `IRIS-V1-META-C054` | Add `RUNTIME-V071`: nested Module composition deduplicates by nearest closed Module identity in MRO. |
| `D-321` | `OK` | `IRIS-V1-META-C060`, `IRIS-V1-META-C113`, `IRIS-V1-RUNTIME-C046` | Direct coverage exists in `IRIS-V1-META-V021`: explicit remove plus include establishes Module reordering semantics. |
| `D-322` | `FIX` | `IRIS-V1-ASYNC-C045`, `IRIS-V1-META-C071` | Replace `META-V048` with `IRIS-V1-ASYNC-V028`: no immediate hooks during construction, validation, or commit; revision observation is after-commit. |
| `D-323` | `FIX` | `IRIS-V1-ASYNC-C046`, `IRIS-V1-META-C071` | Replace `META-V048` with `IRIS-V1-ASYNC-V028`: subscribable revision event payload is ReflectionPolicy-filtered and has no private or rollback authority. |
| `D-324` | `FIX` | `IRIS-V1-ASYNC-C048`, `IRIS-V1-ASYNC-C056`, `IRIS-V1-META-C071` | Replace `META-V048` with `IRIS-V1-ASYNC-V028`: subscriber failure creates ExceptionContext on event-error channel and cannot roll back commit. |
| `D-325` | `FIX` | `IRIS-V1-ASYNC-C047`, `IRIS-V1-ASYNC-C048`, `IRIS-V1-META-C071` | Replace `META-V048` with `IRIS-V1-ASYNC-V028`: revision event is queued after commit returns and subscriber code never runs in safepoint. |
| `D-326` | `FIX` | `IRIS-V1-ASYNC-C051`, `IRIS-V1-ASYNC-C052`, `IRIS-V1-ASYNC-C056` | Replace `IRIS-V1-META-C117` and `META-V048` with `IRIS-V1-ASYNC-V028`: full subscriber queue delivers `GapEvent(from_commit, to_commit)` before later retained events. |
| `D-327` | `FIX` | `IRIS-V1-ASYNC-C053`, `IRIS-V1-ASYNC-C054`, `IRIS-V1-ASYNC-C055` | Replace `META-V048` with `IRIS-V1-ASYNC-V028`: retained audit history yields events in commit order; pruned range raises `AuditHistoryUnavailableError` with no partial-complete stream. |
| `D-328` | `FIX` | `IRIS-V1-META-C100`, `IRIS-V1-META-C101`, `IRIS-V1-META-C113` | Replace broad `META-V048` with `IRIS-V1-META-V036`, `IRIS-V1-META-V037`, and `ADD META-V082`: reflection APIs reject any first-class `ReflectionCapability` token path. |
| `D-329` | `OK` | `IRIS-V1-META-C102`, `IRIS-V1-META-C103`, `IRIS-V1-META-C108` | Direct coverage exists in `IRIS-V1-META-V036`, `IRIS-V1-META-V037`, and `IRIS-V1-META-V044`: inspect and mutate grants are independent and scoped. |
| `D-330` | `OK` | `IRIS-V1-META-C008`, `IRIS-V1-META-C009`, `IRIS-V1-META-C010` | Direct coverage exists in `IRIS-V1-META-V043`, `IRIS-V1-META-V044`, and `IRIS-V1-META-V045`: requests are not grants, Host grants decide. |
| `D-331` | `FIX` | `IRIS-V1-META-C104`, `IRIS-V1-META-C103`, `IRIS-V1-META-C108` | Add `META-V083`: helper package with no grant calls reflection on behalf of granted caller or dependency and is denied because grants do not flow. |
| `D-332` | `FIX` | `IRIS-V1-META-C105`, `IRIS-V1-META-C104` | Add `META-V084`: Method, Closure, BoundMethod, and extension-open Method reflection checks use lexical definition package, including after Closure escape. |
| `D-333` | `FIX` | `IRIS-V1-META-C106`, `IRIS-V1-META-C103` | Add `META-V085`: origin body, declarative open body, and programmatic open Closure each use their concrete source or lexical package ReflectionPolicy. |
| `D-334` | `FIX` | `IRIS-V1-META-C107`, `IRIS-V1-FFI-C018`, `IRIS-V1-META-C103` | Add `META-V086`: native callable uses native artifact manifest package policy; manifestless native artifact has no reflection unless Host assigns trusted identity. |
| `D-335` | `OK` | `IRIS-V1-META-C109`, `IRIS-V1-RUNTIME-C068`, `IRIS-V1-META-C113` | Direct coverage exists in `IRIS-V1-META-V036`, `IRIS-V1-META-V037`, and `IRIS-V1-RUNTIME-V028`: missing get returns `nil`, set returns assigned value, absent remove errors. |
| `D-336` | `OK` | `IRIS-V1-META-C110`, `IRIS-V1-META-C111`, `IRIS-V1-GRAMMAR-C011` | Direct coverage exists in `IRIS-V1-META-V038`: invalid raw ivar reflection names raise `InvalidInstanceVariableNameError`. |
| `D-337` | `OK` | `IRIS-V1-GRAMMAR-C012`, `IRIS-V1-COLLECTIONS-C063`, `IRIS-V1-COLLECTIONS-C003` | Direct coverage exists in `IRIS-V1-COLLECTIONS-V041`: Symbol literal forms and post-escape quoted content are covered. |
| `D-338` | `OK` | `IRIS-V1-COLLECTIONS-C003`, `IRIS-V1-COLLECTIONS-C064` | Direct coverage exists in `IRIS-V1-COLLECTIONS-V041`: Symbol immutability, identity-less behavior, and equality are covered. |
| `D-339` | `FIX` | `IRIS-V1-COLLECTIONS-C085`, `IRIS-V1-COLLECTIONS-C091`, `IRIS-V1-COLLECTIONS-C064` | Keep `IRIS-V1-COLLECTIONS-V043` only if it includes Symbol hash values; otherwise add `COLLECTIONS-V044`: exact Symbol hash BLAKE3 context and output vector. |
| `D-340` | `OK` | `IRIS-V1-GRAMMAR-C009`, `IRIS-V1-COLLECTIONS-C065`, `IRIS-V1-COLLECTIONS-C063` | Direct coverage exists in `IRIS-V1-COLLECTIONS-V041`: identifier NFC normalization and case-sensitive Symbol canonical content are covered. |
| `D-341` | `FIX` | `IRIS-V1-GRAMMAR-C009`, `IRIS-V1-COLLECTIONS-C042`, `IRIS-V1-COLLECTIONS-C063` | Add `GRAMMAR-V018`: Unicode 17 XID identifier accepts XID start/continue and rejects Pattern_Syntax, Pattern_White_Space, controls, and default-ignorable code points. |

## Writer Notes

- Do not use `IRIS-V1-META-V047` or `IRIS-V1-META-V048` as the only vector for a row unless the scenario in the vector text names that exact behavior.
- For `D-322` through `D-327`, keep `IRIS-V1-ASYNC-V028` as the coverage vector and anchor the clauses in `07-async-resources-diagnostics.md`.
- For `D-233` through `D-241`, keep the core anchors in `05-types-contracts-generics.md` and `03-runtime-object-model.md`.
