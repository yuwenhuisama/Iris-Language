use super::Aliases;
use iris_syntax::{Declaration, Decorator, ExportDeclaration};

impl Aliases<'_> {
    pub(super) fn declaration(&mut self, declaration: &mut Declaration) {
        let scope = self.shadowed.len();
        match declaration {
            Declaration::Class(value) => {
                self.shadowed.extend(value.parameters.iter().cloned());
                self.decorators(&mut value.decorators);
                self.optional(&mut value.extends);
                self.types(&mut value.implements);
                for mixin in &mut value.mixins {
                    self.annotation(&mut mixin.target);
                }
                for constraint in &mut value.constraints {
                    self.annotation(&mut constraint.bound);
                }
                self.body(&mut value.body);
            }
            Declaration::Module(value) => {
                self.shadowed.extend(value.parameters.iter().cloned());
                self.decorators(&mut value.decorators);
                self.types(&mut value.contract_for);
                for mixin in &mut value.mixins {
                    self.annotation(&mut mixin.target);
                }
                for constraint in &mut value.constraints {
                    self.annotation(&mut constraint.bound);
                }
                self.body(&mut value.body);
            }
            Declaration::Contract(value) => {
                self.shadowed.extend(value.parameters.iter().cloned());
                self.decorators(&mut value.decorators);
                self.types(&mut value.parents);
                for constraint in &mut value.constraints {
                    self.annotation(&mut constraint.bound);
                }
                self.body(&mut value.body);
            }
            Declaration::Export(export) => match export.as_mut() {
                ExportDeclaration::Declaration(inner) => self.declaration(inner),
                ExportDeclaration::Names(_) => {}
            },
            Declaration::TypeAlias(_) | Declaration::Import(_) => {}
        }
        self.shadowed.truncate(scope);
    }

    pub(super) fn decorators(&mut self, decorators: &mut [Decorator]) {
        for decorator in decorators {
            for argument in &mut decorator.arguments {
                self.expression(argument);
            }
        }
    }
}
