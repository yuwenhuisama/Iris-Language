# Native Module V1 Engineering Contract

This additive module ABI does not change the fixture `iris-abi` ABI or frozen
specification. The standalone `crates/iris-native-sdk` package has no runtime
dependency. Its C counterpart is `crates/iris-native-sdk/include/iris_native_v1.h`.

## Tables And Calls

The exported `iris_native_module_v1(out: *mut ModuleV1) -> i32` writes a descriptor.
HostV1 and ModuleV1 are independently named records with HeaderV1 fields
`major: u32, minor: u32, size: usize, features: u64`. V1 is major 1, minor 0,
features 0. The host initializes `out.header.size` to its descriptor capacity;
the module must check that capacity before writing. Both sides check version,
size and required feature bits before reading callbacks. All records use C layout.

ModuleV1 has `function_count, resource_count: usize`, and nullable C callbacks
`function_at(index, out_function) -> i32`, `resource_at(index, out_resource) -> i32`.
Its final field is `metadata_sha256: [u8; 32]`, the SHA-256 of the exact trusted
metadata JSON bytes, including whitespace. The host compares this with its own
computed digest before accepting any descriptor entries. This binds the package
identity, version, permissions and ordered signatures to the descriptor, not just
its entry counts. Generate the digest at module build time from the shipped JSON.
Function indices flatten metadata modules/functions in declaration order. Resource
indices similarly flatten modules/resources. Entries must exactly match trusted
metadata. Descriptor callbacks and returned function pointers remain valid until
library unload. ResourceV1 is `{size, close, destroy}`; both callbacks are required.

FunctionV1 is `unsafe extern "C" fn(*const HostV1, *mut c_void,
*const Handle, usize, *mut CallResultV1) -> i32`. CloseV1 receives host, call,
`cookie: u64`, result instead. DestroyV1 receives only the cookie and cannot call
the host. CallResultV1 is `{size: usize, value: u64, context: u64}`. Initialize it
with its full size and zero handles. Success must return a nonzero value handle
(including Nil); failure Raised must return an error context handle.

Statuses are raw i32: success 0, invalid handle 1, invalid runtime 2, thread
affinity 3, raised 4, duplicate completion 5, incompatible ABI 6, invalid argument
7, invalid boundary 8. Unknown integers are protocol failures, never Rust enums.
Scalar kinds are Nil 0, Bool 1, Integer 2. Bool integer payload is exactly 0 or 1;
Nil payload is zero. Blob kinds are String 3 and Bytes 4. Integers are signed i64
at this boundary; larger Iris Integers are rejected without truncation.

Host callbacks (all return i32) are defined literally in the header:
`scalar_read/create`, `blob_info/read/create`, `resource_create/read`, and
`error_raise`. Blob reads copy the complete blob into a caller buffer with at
least the advertised length. Creation copies input buffers; String requires UTF-8.
Slices are pointer plus byte length; zero length permits null. Nonempty pointers
must be valid for the complete operation. Output pointers are aligned, writable,
and disjoint from inputs and the opaque context. No callback retains pointers.
`error_raise` takes UTF-8 code/message and signed OS code (zero means absent),
writes context, and returns Raised. The frame sets up an ExceptionContext whose
ExceptionOrigin carries a native bridge record with the declaring package ID,
code, message, and OS code. Codes are exposed as Iris Symbols; messages and OS
codes remain available to the host error object.

Calls are synchronous and thread-affine. The opaque call pointer and all handles
expire on return. Handles are nonzero, never reused, and cannot be retained across
calls. There are no nested Iris calls, persistent handles, managed pointers, or GC
callbacks. Host values are owned during the frame; the result is cloned/rooted
before ending the frame. Modules must catch panics internally, never unwind or
longjmp through C, and never use callbacks from a background thread.

Resources use host-owned records and private module cookie indices, not pointers
or Iris-visible numeric IDs. Aliases share closed state. `resource_read` verifies
module ownership and resource type and rejects closed records. Explicit close is
idempotent even if the module close reports failure: the first call consumes the
open state. Close must release OS handles; destroy releases private bookkeeping
and any still-open OS handles. Registry teardown destroys retained records on its
owning thread before unloading libraries. First release retains records with a
policy quota rather than promising GC reclamation. A resource surviving registry
teardown is inert, not a way to invoke an unloaded library.
Cookie ownership transfers only after `resource_create` succeeds. If creation
fails (including quota exhaustion), the module must release its untransferred
cookie and OS handles itself. A cookie must be unique across the loaded module's
resource records, including closed records retained until registry teardown.

## Trusted Metadata

The host parses strict serde JSON before loading any machine code:

```json
{
  "schema_version": 1,
  "package_id": "org.example.native",
  "version": "1.0.0",
  "api_major": 1,
  "iris_major": 1,
  "abi": {"major": 1, "minimum_minor": 0, "required_features": 0},
  "permissions": {"required": ["native.load", "network.tcp"], "optional": []},
  "reflection_policy": "deny",
  "modules": [{
    "name": "Example::Native",
    "functions": [{"name": "connect", "parameters": [{"name": "address", "type": "String"}], "returns": "resource:Socket"}],
    "resources": [{"name": "Socket", "contracts": ["Closeable"], "managed_roots": false, "storage": "external-cookie-v1"}]
  }]
}
```

Type vocabulary is `Nil`, `Bool`, `Integer`, `String`, `Bytes`, and
`resource:<name>` (local to the declaring module). Reflection policy currently
accepts only `deny`. Resources require exactly Closeable, false managed_roots,
and external-cookie-v1. Identifiers are validated; duplicate exports are refused.

Load requires an explicit NativePolicy, artifact path, metadata bytes, and trusted
SHA-256 digests for BOTH metadata and artifact. The host computes actual SHA-256
and compares it, validates compatibility, required permissions, and policy native
authorization BEFORE dlopen: dynamic library constructors execute at load time.
The trusted caller must prevent concurrent artifact replacement until loading
completes and trust transitive dynamic dependencies. Native authorization is not
an OS sandbox; authorized native code has process privileges. Permissions are an
admission policy, not a claim to contain hostile machine code.

## Embedding

`iris_native_host::NativeRegistry` is shared via `Rc` by engine adapters. Load
metadata before compiling/evaluating source. Engine adapters bind declared
native modules, never intercept arbitrary source strings. Package runners must
retain source order, package identity, and explicit imports. VM package shapes
that cannot preserve these properties must return an unsupported error, never
fall back to the evaluator.

Public Rust adapters:

```text
NativeRegistry::new() -> NativeRegistry
NativeRegistry::load(TrustedModule<'_>, &NativePolicy) -> Result<(), NativeError>
NativeRegistry::call(&str, &[Value]) -> Result<Value, NativeError>
NativeRegistry::close(&ExternalResource) -> Result<Value, NativeError>
iris_eval::Session::with_natives(Rc<NativeRegistry>) -> Result<Session, EvaluationError>
Session::evaluate_in_package(&str, &str) -> Result<Value, EvaluationError>
iris_eval::evaluate_with_natives(&str, Rc<NativeRegistry>) -> Result<Value, EvaluationError>
iris_eval::evaluate_packages_with_natives(&[(String, String)], Rc<NativeRegistry>) -> Result<Value, EvaluationError>
iris_eval::evaluate_package_tree_with_natives(&[PackageSource], Rc<NativeRegistry>) -> Result<Value, EvaluationError>
iris_vm::compile_with_native(&str, &NativeRegistry) -> Result<Program, CompileError>
iris_vm::compile_with_natives(&str, &NativeRegistry) -> Result<Program, CompileError>
iris_vm::compile_packages_with_natives(&[(String, String)], &NativeRegistry) -> Result<Program, CompileError>
iris_vm::compile_package_tree_with_natives(&[PackageSource], &NativeRegistry) -> Result<Program, CompileError>
iris_vm::Machine::with_natives(Rc<NativeRegistry>) -> Result<Machine, KernelError>
iris_vm::run_with_natives(&Program, Rc<NativeRegistry>) -> Result<Value, MachineError>
```

`PackageSource` contains `package_id`, `api_major: u32`, `version: String`,
`path`, `source`, and an explicit
`allowed_imports: BTreeSet<String>`. The source evaluator supports multi-package
trees and preserves ordered package identities. The VM native compiler supports
ordered source units sharing the same package identity, validating imports per
unit and checking each source with the parser before joining units with newlines.
All source units for one package ID must agree on API major, version, and import
permissions. The evaluator enters the declared major and version and preserves
declaration ownership across package calls. VM `Program::package_identity()`
retains the package ID, major, and version; nominal Type and Contract hashes use
the ID and major, not the release version. Manifestless scripts retain
`runtime-local` / major 1 identity. The trusted tuple adapters have no version
input and use major 1 with an unknown version.
The VM rejects empty sources and rejects cross-package source execution explicitly.
Native-only dependencies do not add source units and retain their own metadata
ownership. VM `Reflection::Package` and native Module Type reflection remain
unsupported rather than reporting the entry package's metadata.
VM package execution also refuses built-in Type package/hash reflection until
built-in declaration ownership is represented; it never labels built-ins as
belonging to the entry package.
`import NativeModule` is required in both engines. Native declaration
replacement is refused. The native resource wrapper carries no plugin pointer
or numeric cookie; a registry must remain alive to operate on it.
`using(resource)` runs native close in both engines.
