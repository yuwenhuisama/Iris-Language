//! A bytecode backend for Iris.
//!
//! `IRIS-V1-TRACE-C011` puts the VM out of scope for this specification wave,
//! so the instruction set is an engineering choice rather than a mandated one.
//! What IS constrained is agreement: a differential row asserts that every
//! backend produces the same semantic observation, and
//! `IRIS-V1-CONFORMANCE-C068` forbids comparing private bytecode layout, so
//! only the observable result matters here.
//!
//! This backend is deliberately PARTIAL. It compiles the constructs it fully
//! understands and DECLINES everything else, rather than approximating. A
//! backend that guesses would make a differential row agree for the wrong
//! reason, which is worse than leaving the row held.
//!
//! Arithmetic dispatches through [`iris_runtime::Kernel`], the same kernel the
//! tree-walking evaluator uses. Reimplementing bigint or IEEE-754 arithmetic
//! here would create exactly the silent numeric divergence that
//! `IRIS-V1-RUNTIME-V052` and `IRIS-V1-RUNTIME-V066` exist to detect, and the
//! rows would then be comparing two copies of the same bug.

mod compile;
mod machine;

pub use compile::{CompileError, FloatWidth, Program, compile};
pub use machine::{Machine, MachineError, run};
