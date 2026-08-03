# META chapter classification

`spec/iris-v1/08-modules-metaprogramming.md` states 51 vector rows. 1 is
committed here; the remaining 50 are NOT yet transcribed and are bucketed below
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

## Blocked

| Blocker | Rows | Why |
| --- | --- | --- |
| Open transactions | `V340`, `V341`, `V342`, `V345`, `V351`, `V353`, `V354`, `V355`, `V356`, `V426`, `V427`, `V428`, `V430`, `V436`, `V437`, `V439`, `V441` | `IRIS-V1-META-C022` through `C025` make a Class or Module origin body an executable construction transaction over a CANDIDATE, published atomically at a safepoint. `Class.open`, `define_method` and `superclass=` do not exist, and neither does candidate isolation: a declarative `open class` today edits the published Class directly. These are the same rows seven TYPES rows are blocked on. |
| Permissions and policy | `V359`, `V360`, `V361`, `V362`, `V363`, `V421`, `V422`, `V423`, `V435` | `IRIS-V1-META-C009` makes a permission a manifest REQUEST that only Host configuration can grant, and forbids a package from self-authorizing. The loader models no permission field, and there is no Host grant surface to check one against. |
| Upgrade, lock and digest | `V352`, `V357`, `V420`, `V429`, `V431` | `IRIS-V1-META-C066` and `C067` resolve an artifact through the active package store and verify a BLAKE3-256 canonical manifest digest. There is no package store, no `iris.lock`, and no digest scheme. `V357` is the one row naming `artifact.json` rather than a manifest. |
| Reflection views | `V344`, `V358`, `V416`, `V424`, `V425`, `V432` | `IRIS-V1-META-C100` and `C107` expose `Reflection::Object` ivar APIs and package-scoped reflection whose authorization is ambient to the executing package. The reflection surface these rows read does not exist. `V416` additionally needs `open module P::M` across two files in one package. |
| Cross-package import | `V346`, `V347`, `V348`, `V349`, `V417`, `V418`, `V419`, `V438` | Each needs a SECOND package to import from, plus dependency resolution. `D-432`'s three resolution tiers are implemented and observed by `IRIS-V1-CONTROL-V351`, but only for a Module already loaded into this runtime; a cross-package target needs the dependency machinery `IRIS-V1-META-C003` puts in the manifest. |
| Decorators | `V434` | `IRIS-V1-META-C001` owns decorators, whose static plan and application order have no implementation. |
| Other | `V343`, `V350`, `V433`, `V440` | Each depends on one of the above: `V343` and `V440` on candidate validation, `V350` on module composition authorization, `V433` on revision migration. |
