# META chapter classification

`spec/iris-v1/08-modules-metaprogramming.md` states 51 vector rows. 13 are
committed here; the remaining 38 are NOT yet transcribed and are bucketed below
by what actually blocks each one.

Every row names a concrete on-disk fixture, `fixtures/meta/vNNN/iris.toml` for
50 of them and `fixtures/meta/v357/artifact.json` for the remaining one, rather
than carrying its program inline. The corpus now reads such a tree: a package
fixture supplies `package_id`, `api_major` and ordered source entries, and
`IRIS-V1-META-C010` aborts the load when that manifest is missing or invalid.
That input form is what this chapter needed before ANY row could be transcribed.

The loader deliberately models only the manifest subset the transcribed row
observes. `IRIS-V1-META-C003` also lists a package version, an Iris language
range, dependency constraints, permission requests and native artifact
declarations. An unmodelled key is IGNORED rather than honoured, so a fixture
may carry one without this pretending to support it, and a row that OBSERVES
such a field stays blocked below.

## Transcribed

| Row | Category | Bucket | Note |
| --- | --- | --- | --- |
| `V415` | positive | executable | `IRIS-V1-META-C017` initializes a package in manifest-declared source order, and `IRIS-V1-META-C011` puts every executable statement inside a Module body, so a package source file is declarations only. The load observes the initialized Module order `[First, Second]`. |

| `V342` | positive | executable | `IRIS-V1-META-C023` defines the Method on the current candidate and `C026` keeps it from closing over the body's transaction locals, so `answer` resolves the DECLARED `value()` and answers 8 rather than the local's 7. |

| `V341` | negative | executable | `IRIS-V1-META-C022` lets a Class body run ordinary control flow including `raise`, and publishes NOTHING from the failed candidate, so `Box` stays unpublished and its staged `ok` Method is not committed. |
| `V441` | diagnostic | executable | `IRIS-V1-TYPES-C042` forbids a Method body in a Contract and `IRIS-V1-META-C031` rejects opening one before publication without creating a candidate, so both codes are reported and neither input publishes Contract `C`. |

| `V361` | diagnostic | executable | `IRIS-V1-META-C081`'s capability matrix denies a meta operation missing its capability with `MetaCapabilityError` while the allowed lane commits. The vector covers the METHOD and PROPERTY lanes -- `method_set`, `method_body` and `property_set` -- against separate owners. The row also schedules `class_state_write`, `shape` and `instance_state` lanes over raw ivars, which need `Reflection::Object` ivar APIs that do not exist; that remainder is recorded rather than asserted. |

| `V360` | diagnostic | executable | `IRIS-V1-META-C076` subtracts denials from the declared superclass chain, Module-sourced denies and Contract-required denies alike, and `C077` forbids a subclass or open from re-enabling an ancestor's denial. The vector observes all three origins combining and the open being refused. The row also schedules removing a Module to drop only its own deny, and static diagnostics for a body-level `meta` and an unknown name; the parse-level diagnostics exist and the removal lane needs `Reflection::Class.remove_module` policy recomputation, which is recorded rather than asserted. |

| `V359` | differential | executable | `IRIS-V1-RUNTIME-C148` lets a stable built-in compose Modules and gain compatible Methods on open while `C150` protects its superclass, and `IRIS-V1-META-C081` names the missing capability in the refusal. The vector observes the gained behaviour, the retained primitive identity, and all three superclass refusals. |

| `V419` | negative | executable | `IRIS-V1-META-C017` makes Module initialization an ACYCLIC deterministic DAG and requires a dependency or initialization cycle to be a compile or link error. The two source files import each other, so the package fails to link and neither Module is initialized. This row needs no second package: its cycle is WITHIN one package's sources. |

| `V417` | diagnostic | executable | `IRIS-V1-META-C013` keeps wildcard imports out of Iris v1 source and the v1.23 errata `IRIS-V1-GRAMMAR-C068` supplies the dotted package path the row writes, so the fixture parses far enough for the wildcard to be rejected under the code the row names. |

| `V434` | differential | executable | `IRIS-V1-RUNTIME-C161` keeps `@name` per receiver, `C072` keeps an escaping Closure bound to the receiver that created it, and `C077` answers a private Method only for its declaring Class's lexical call. The row also schedules Class-object `@x` and `@@x` distinctness and a Dynamic call path, which need `Reflection::Object` get/set APIs; that remainder is recorded rather than asserted. |

| `V358` | positive | executable | `IRIS-V1-TYPES-C043` forms Contract inheritance as a plain relation and `IRIS-V1-META-C078` adds a Module edge only through `mixin`, so both views are EMPTY with no implicit parent or edge. The row also observes the normalized `Empty::Self` upper bound, which needs a `Self` bound reflection that does not exist; that third half is recorded rather than asserted. |

| `V344` | positive | executable | `IRIS-V1-META-C027` makes a Class body local an ordinary lexical local that a nested Closure may capture and an escaping Closure may outlive, and `C025` keeps it from becoming a property or storage slot. Two Closures each keep their OWN local and neither name appears in the Class's properties. |

| `V426` | positive | executable | `IRIS-V1-META-C097` exposes `active_revision`, and `C022` makes each body ONE atomic publication, so a committed `Box.open` advances the revision by exactly one and its defined selector answers. The vector asserts the DIFFERENCE rather than an absolute number: a Class's baseline reflects how the kernel builds it, which is an implementation fact rather than a specified one. The row also lists an audit diff, which needs the Revision view's audit data and is recorded rather than asserted. |

## Blocked

| Blocker | Rows | Why |
| --- | --- | --- |
| Open transactions | `V340`, `V345`, `V351`, `V353`, `V354`, `V355`, `V356`, `V427`, `V428`, `V430`, `V436`, `V437`, `V439` | `IRIS-V1-META-C022` through `C025` make a Class or Module origin body an executable construction transaction over a CANDIDATE, published atomically at a safepoint. Candidate isolation now EXISTS: a body accumulates into one candidate and publishes atomically or rolls back, Class bodies run executable statements, and `self.define_method` targets the candidate. Programmatic `Class#open` and `define_method` now exist too, with the C033 refusal of a Contract or closed generic target. Nested transaction GROUPS now exist too: `C038` joins nested opens into one group that publishes or rolls back together and reuses an already-present target's candidate, `C039` compares each candidate's base revision and raises `MetaTransactionConflict`, and `C041`'s atomic group publication shares one commit identity. `C040`'s catchable retry works now that `IRIS-V1-CONTROL-C056` makes a specification-named error catchable. What remains is `Module#open` and `superclass=`. Module-side `define_method` is now implemented, which closed `V342`. These are the same rows seven TYPES rows are blocked on. |
| Permissions and policy | `V362`, `V363`, `V421`, `V422`, `V423`, `V435` | Re-probed row by row rather than as one bucket, which closed `V359`, `V360` and `V361`: `meta deny` and the `C081` capability checks were already implemented and enforcing, and only the observation surface was missing. These six genuinely need what the bucket claimed. `IRIS-V1-META-C009` makes a permission a manifest REQUEST that only Host configuration can grant and forbids a package from self-authorizing; the loader models no permission field and there is no Host grant surface to check one against, which blocks `V421`, `V422`, `V423`, `V363` and `V435`. `V362` additionally needs the `Reflection::Object` ivar APIs of `IRIS-V1-META-C100`: `list_ivars` exists but `get_ivar`, `set_ivar` and `remove_ivar` do not. |
| Upgrade, lock and digest | `V352`, `V357`, `V420`, `V429`, `V431` | `IRIS-V1-META-C066` and `C067` resolve an artifact through the active package store and verify a BLAKE3-256 canonical manifest digest. There is no package store, no `iris.lock`, and no digest scheme. `V357` is the one row naming `artifact.json` rather than a manifest. |
| Reflection views | `V416`, `V424`, `V425`, `V432` | `IRIS-V1-META-C100` and `C107` expose `Reflection::Object` ivar APIs and package-scoped reflection whose authorization is ambient to the executing package. The reflection surface these rows read does not exist. `V416` additionally needs `open module P::M` across two files in one package. |
| Cross-package import | `V346`, `V347`, `V348`, `V349`, `V418`, `V438` | Multi-package LOADING exists: a fixture tree resolves `dependencies` per `IRIS-V1-META-C006`, orders dependencies before dependents per `C017`, aborts on a missing or cyclic dependency per `C007`, and loads the ordered packages onto ONE runtime, so a consumer reaches what its dependency exported. The facade spellings `export import` and `export from` parse, and the v1.24 errata `IRIS-V1-GRAMMAR-C069` supplies the `C049` `override` marker. What blocks all six is ACTIVATION, and probing corrected where that lives. `IRIS-V1-META-C045` is the governing clause, not `C016` alone: cross-Module static visibility requires `export open ...` AND each consumer directly importing the extension Module, with transitive imports and re-exports never activating. `C019` then makes direct import SOURCE ORDER decide override order, and `C048` allows a later import to replace an earlier member only on complete static-signature compatibility. All of that is a STATIC gate, so it belongs in the analyser rather than the evaluator: today an `open class` takes effect globally and immediately, and removing a consumer's import does not deactivate it. `V346` and `V438` observe `IRIS-STATIC-MEMBER-NOT-FOUND`, which appears ONLY in vector rows and in no clause, and which needs static member-existence checking that does not exist at all -- `A.new().nothing()` is equally undiagnosed, the same blocker `IRIS-V1-TYPES-V208` carries. |
| Decorators | `V429`, `V431` | `IRIS-V1-META-C001` owns decorators, whose static plan and application order have no implementation. |
| Other | `V343`, `V350`, `V433`, `V440` | Each depends on one of the above: `V343` and `V440` on candidate validation, `V350` on module composition authorization, `V433` on revision migration. |
