use iris_syntax::Statement;

use super::lowering::{Binding, Lowering};
use super::{CompileError, Instruction, Register};

#[derive(Clone)]
pub(super) enum ReturnScope {
    Try {
        finally: Vec<Statement>,
        names: Vec<Binding>,
        contexts: Vec<(Register, Register)>,
    },
    Iterator(Register),
}

impl Lowering<'_, '_> {
    pub(super) fn enter_return_scope(&mut self, finally: &[Statement]) {
        self.return_scopes.push(ReturnScope::Try {
            finally: finally.to_vec(),
            names: self.names.clone(),
            contexts: self.exception_contexts.clone(),
        });
    }

    pub(super) fn return_value(&mut self, value: Register) -> Result<(), CompileError> {
        let scopes = self.return_scopes.clone();
        let names = self.names.clone();
        let contexts = self.exception_contexts.clone();
        while let Some(scope) = self.return_scopes.pop() {
            self.instructions.push(Instruction::LeaveTry);
            match scope {
                ReturnScope::Try {
                    finally,
                    names,
                    contexts,
                } => {
                    self.names = names;
                    self.exception_contexts = contexts;
                    self.body(&finally)?;
                }
                ReturnScope::Iterator(iterator) => {
                    self.instructions.push(Instruction::IteratorClose {
                        iterator,
                        context: None,
                    });
                }
            }
        }
        self.return_scopes = scopes;
        self.names = names;
        self.exception_contexts = contexts;
        if let Some(annotation) = self.return_annotation.clone() {
            self.instructions
                .push(Instruction::CheckReturn { value, annotation });
        }
        self.instructions.push(Instruction::Return { value });
        Ok(())
    }
}
