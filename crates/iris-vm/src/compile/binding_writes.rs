use super::{
    CompileError, Instruction, Register,
    lowering::{Binding, Lowering},
};
use iris_syntax::TypeExpression;

impl Lowering<'_, '_> {
    pub(super) fn write_binding(
        &mut self,
        binding: &Binding,
        source: Register,
    ) -> Result<(), CompileError> {
        self.check_binding_value(source, binding.annotation.as_ref());
        if binding.shared {
            self.instructions.push(Instruction::StoreCell {
                destination: source,
                cell: binding.register,
                source,
            });
        } else {
            self.instructions.push(Instruction::Move {
                destination: binding.register,
                source,
            });
        }
        Ok(())
    }

    pub(super) fn check_binding_value(
        &mut self,
        value: Register,
        annotation: Option<&TypeExpression>,
    ) {
        if let Some(annotation) = annotation {
            self.instructions.push(Instruction::CheckAnnotation {
                value,
                annotation: annotation.clone(),
            });
        }
    }
}
