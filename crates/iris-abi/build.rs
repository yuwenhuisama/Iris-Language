//! Compiles the conformance C fixtures.
//!
//! `IRIS-V1-FFI-V060` requires the fixture to be compiled AS C and called
//! through the Host table, so these are built with a C compiler and linked into
//! the test binary rather than being reimplemented in Rust.

fn main() {
    let fixtures = std::path::Path::new("../../conformance/iris-v1/fixtures/ffi");
    let sources = [
        "host_abi_v1.c",
        "rooted_handle.c",
        "thread_affinity.c",
        "native_error.c",
        "async_once.c",
        "negotiate_v2.c",
    ];
    for source in sources {
        println!("cargo:rerun-if-changed={}/{source}", fixtures.display());
    }
    println!("cargo:rerun-if-changed={}/iris_abi.h", fixtures.display());
    cc::Build::new()
        .include(fixtures)
        .files(sources.iter().map(|source| fixtures.join(source)))
        .compile("iris_ffi_fixtures");
}
