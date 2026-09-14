use super::{Machine, MachineError, VerifyError};
use crate::compile::Program;
use iris_runtime::MethodBody;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MethodBodyError {
    IndexOverflow { body: MethodBody },
    UnknownFunction { body: MethodBody, function: usize },
}

impl MethodBodyError {
    pub(super) fn invalid_function(self) -> MachineError {
        let function = match self {
            Self::IndexOverflow { .. } => usize::MAX,
            Self::UnknownFunction { function, .. } => function,
        };
        MachineError::Invalid(VerifyError::UnknownFunction { function })
    }

    pub(super) fn with_overflow(self, overflow: MachineError) -> MachineError {
        match self {
            Self::IndexOverflow { .. } => overflow,
            Self::UnknownFunction { .. } => self.invalid_function(),
        }
    }
}

impl Machine {
    pub(super) fn resolve_method_body(
        &self,
        body: MethodBody,
        _program: &Program,
    ) -> Result<super::code::ResolvedBody, MethodBodyError> {
        let function =
            usize::try_from(body.raw()).map_err(|_| MethodBodyError::IndexOverflow { body })?;
        self.code
            .resolve(function)
            .ok_or(MethodBodyError::UnknownFunction { body, function })
    }
}

#[cfg(test)]
mod tests {
    use super::super::Machine;
    use super::MethodBodyError;
    use iris_runtime::MethodBody;

    #[test]
    fn last_function_resolves_when_body_is_in_current_program() -> Result<(), String> {
        let program = crate::compile(
            "class Target { public fun first() { 1 } public fun last() { 2 } } Target.new().last()",
        )
        .map_err(|error| format!("{error:?}"))?;
        let mut machine = Machine::new().map_err(|error| format!("{error:?}"))?;
        machine
            .code
            .register(&program)
            .map_err(|error| format!("{error:?}"))?;
        let index = program.functions.len() - 1;
        let body = MethodBody::new(u64::try_from(index).map_err(|error| error.to_string())?);

        let when = machine.resolve_method_body(body, &program);

        assert_eq!(when.map(|owner| owner.function), Ok(index));
        Ok(())
    }

    #[test]
    fn unknown_function_keeps_index_when_body_is_at_program_boundary() -> Result<(), String> {
        let program =
            crate::compile("class Target { public fun value() { 1 } } Target.new().value()")
                .map_err(|error| format!("{error:?}"))?;
        let machine = Machine::new().map_err(|error| format!("{error:?}"))?;
        let function = program.functions.len();
        let body = MethodBody::new(u64::try_from(function).map_err(|error| error.to_string())?);

        let when = machine.resolve_method_body(body, &program);

        assert_eq!(
            when,
            Err(MethodBodyError::UnknownFunction { body, function })
        );
        Ok(())
    }

    #[test]
    fn maximum_body_is_rejected_without_truncation() -> Result<(), String> {
        let program = crate::compile("nil").map_err(|error| format!("{error:?}"))?;
        let machine = Machine::new().map_err(|error| format!("{error:?}"))?;
        let body = MethodBody::new(u64::MAX);

        let when = machine.resolve_method_body(body, &program);

        let expected = match usize::try_from(u64::MAX) {
            Ok(function) => MethodBodyError::UnknownFunction { body, function },
            Err(_) => MethodBodyError::IndexOverflow { body },
        };
        assert_eq!(when, Err(expected));
        Ok(())
    }

    #[test]
    fn overflow_keeps_caller_error_when_policy_is_selected() {
        let given = MethodBodyError::IndexOverflow {
            body: MethodBody::new(u64::MAX),
        };

        let when = given.with_overflow(super::MachineError::UnsupportedConstruct);

        assert_eq!(when, super::MachineError::UnsupportedConstruct);
    }

    #[test]
    fn unknown_function_keeps_index_when_overflow_policy_differs() {
        let given = MethodBodyError::UnknownFunction {
            body: MethodBody::new(7),
            function: 7,
        };

        let when = given.with_overflow(super::MachineError::UnsupportedConstruct);

        assert_eq!(
            when,
            super::MachineError::Invalid(super::VerifyError::UnknownFunction { function: 7 })
        );
    }
}
