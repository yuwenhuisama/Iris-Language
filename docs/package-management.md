# Package CLI

Build the executable with `cargo build -p iris-cli`. Existing script, `-e`,
`--vm`, version, help, and REPL modes remain available.

```sh
iris package install ./app
iris package install ./app --allow-local-git
iris package build ./app --allow-native-build
iris package run ./app
iris package run ./app --vm --allow native.load,network.tcp
```

Flags follow the directory. `run` accepts `--vm` and `--allow` in either order,
once each. Permissions are comma-separated exact names. Without `--allow`, no
permissions are granted. Build consent does not grant runtime permissions.

## Source Packages

An application directory contains `iris.toml` and its declared sources:

```toml
manifest_version = 1
package_id = "org.example.app"
api_major = 1
version = "1.0.0"
iris_major = 1
sources = ["main.iris"]
entry_modules = ["Main"]

[permissions]
required = []
optional = []
```

For example, `main.iris` may contain `print(42)`. Install creates `iris.lock`;
run requires the lock and verifies it rather than silently installing or
updating anything. Editing a locked manifest or source causes an integrity
failure. To intentionally change a local project, review the changes, remove
its old lock explicitly, and install again. Keep lockfiles under version control.

Dependencies use `[dependencies.alias]` with `package_id`, `api_major`, a SemVer
`version` range, credential-free HTTPS/SSH `git` URL, and a full lowercase Git
`rev` (40 or 64 hex digits). Local `file:` URLs require `--allow-local-git` at
install time. Install snapshots pinned sources without running package code.

The evaluator runs dependency-first, then each package's sources in manifest
order, retaining each source's package ID. Imports are limited to the current
package's declared `entry_modules` plus those of declared direct dependencies.
Transitive dependencies and undeclared internal modules are not implicitly
importable. Module names are used directly, for example `import Network`, not
dependency aliases or invented package-qualified prefixes.

## Native Packages

The local application is source-only. Native packages are pinned Git
dependencies with a `[native]` table containing `abi_major`, `minimum_minor`,
`required_features`, `metadata`, `cargo_manifest`, `cargo_package`, and
`cargo_target`. They must track a root `Cargo.lock` and require `native.load`.
Their version-1 native JSON metadata must agree with manifest identity, API,
version, ABI, and permissions. See [Native Module V1](native-module-v1.md).

Only `build --allow-native-build` invokes Cargo. Cargo build scripts and native
libraries are trusted arbitrary code, not sandboxed code. Build receipts bind
the selected artifact to its source, metadata, target platform, and SHA256.
Run verifies receipts and digests, then the native host rechecks artifact and
metadata digests immediately before loading. Run never builds or fetches code.
Required permissions must all appear in `--allow`; optional permissions are
not automatically granted.

The CLI gives each loaded native module a lifetime quota of 4096 resource
allocations. Closing a resource does not replenish this quota. Grants are an
admission policy, not an OS sandbox. Lockfiles and receipts are trusted local
records, not signatures. Serialize operations on a project; hostile concurrent
filesystem mutation and malicious authorized build scripts are not isolated.

## VM Limit

The package VM runner supports multiple ordered source units provided they
belong to the same package. It validates import grants per unit, checks each unit
with the parser before joining, and joins them with newlines. Incomplete parse
shapes split across units, such as a declaration begun in one file and completed
in another, fail parse validation on the incomplete unit. The VM rejects empty
sources and explicitly rejects cross-package execution across multiple source
packages. A native-only dependency can declare `sources = []`, while the
application declares its sources importing that dependency's entry module. The CLI
neither executes cross-package VM bytecode nor falls back to the evaluator. Use
the reference evaluator for full multi-package source execution.

## Verification

```sh
cargo check -p iris-cli
cargo test -p iris-cli
cargo clippy -p iris-cli --all-targets -- -D warnings
```

The CLI tests include pure packages on both engines, source order, import
admission, missing locks, denied build/load, digest parsing, source tampering,
and a real pinned local Git native fixture built and loaded on both engines.
