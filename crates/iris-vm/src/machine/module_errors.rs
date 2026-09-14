use super::{Machine, MachineError};
use iris_runtime::{ModuleCandidateError, RuntimeStructuralError, Value};

impl Machine {
    pub(super) fn module_candidate_error(&mut self, error: ModuleCandidateError) -> MachineError {
        match error {
            ModuleCandidateError::CapabilityDenied { .. } => {
                self.raise_core_value(Value::Symbol("MetaCapabilityError".into()))
            }
            _ => MachineError::MetaTransactionError,
        }
    }

    pub(super) fn structural_error(&mut self, error: RuntimeStructuralError) -> MachineError {
        match error {
            RuntimeStructuralError::Class(error) => MachineError::Class(error),
            RuntimeStructuralError::Module(error) => self.module_candidate_error(error),
        }
    }
}
