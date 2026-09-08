use iris_syntax::{
    Declaration, ExportDeclaration, Program, ProgramEntry, TypeAliasDeclaration, TypeExpression,
};

mod declarations;
mod expressions;
mod statements;

pub(super) struct Aliases<'a> {
    declarations: Vec<&'a TypeAliasDeclaration>,
    shadowed: Vec<String>,
}

impl<'a> Aliases<'a> {
    pub(super) fn new(program: &'a Program, context: &'a [TypeAliasDeclaration]) -> Self {
        let mut declarations: Vec<_> = context.iter().collect();
        for entry in &program.entries {
            let ProgramEntry::Declaration(declaration) = entry else {
                continue;
            };
            let mut declaration = declaration;
            while let Declaration::Export(export) = declaration {
                match export.as_ref() {
                    ExportDeclaration::Declaration(inner) => declaration = inner,
                    ExportDeclaration::Names(_) => break,
                }
            }
            if let Declaration::TypeAlias(alias) = declaration {
                declarations.push(alias);
            }
        }
        Self {
            declarations,
            shadowed: Vec::new(),
        }
    }

    pub(super) fn program(&mut self, program: &mut Program) {
        for entry in &mut program.entries {
            match entry {
                ProgramEntry::Declaration(declaration) => self.declaration(declaration),
                ProgramEntry::Statement(statement) => self.statement(statement),
            }
        }
        synchronize(program);
    }

    pub(super) fn annotation(&mut self, annotation: &mut TypeExpression) {
        if let Some(resolved) = self.resolve(annotation, &mut Vec::new()) {
            *annotation = resolved;
        }
        if let TypeExpression::Typeof(expression) = annotation {
            self.expression(expression);
        }
    }

    fn optional(&mut self, annotation: &mut Option<TypeExpression>) {
        if let Some(annotation) = annotation {
            self.annotation(annotation);
        }
    }

    fn types(&mut self, annotations: &mut [TypeExpression]) {
        for annotation in annotations {
            self.annotation(annotation);
        }
    }

    fn resolve(
        &self,
        annotation: &TypeExpression,
        visiting: &mut Vec<String>,
    ) -> Option<TypeExpression> {
        let (name, arguments) = match annotation {
            TypeExpression::Name(name) => (name, Vec::new()),
            TypeExpression::Generic { name, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|value| self.resolve(value, visiting))
                    .collect::<Option<Vec<_>>>()?;
                (name, arguments)
            }
            TypeExpression::Union(members) => {
                let mut normalized = Vec::new();
                for member in members {
                    match self.resolve(member, visiting)? {
                        TypeExpression::Union(nested) => normalized.extend(nested),
                        member => normalized.push(member),
                    }
                }
                return Some(TypeExpression::Union(normalized));
            }
            TypeExpression::Intersection(members) => {
                return Some(TypeExpression::Intersection(
                    members
                        .iter()
                        .map(|value| self.resolve(value, visiting))
                        .collect::<Option<Vec<_>>>()?,
                ));
            }
            TypeExpression::Function { parameters, result } => {
                return Some(TypeExpression::Function {
                    parameters: parameters
                        .iter()
                        .map(|value| self.resolve(value, visiting))
                        .collect::<Option<Vec<_>>>()?,
                    result: Box::new(self.resolve(result, visiting)?),
                });
            }
            TypeExpression::Typeof(_) => return Some(annotation.clone()),
        };
        if !self.shadowed.contains(name)
            && let Some(alias) = self
                .declarations
                .iter()
                .rev()
                .find(|alias| alias.name == *name)
            && alias.parameters.len() == arguments.len()
        {
            if visiting.contains(name) {
                return None;
            }
            visiting.push(name.clone());
            let definition = Self {
                declarations: self.declarations.clone(),
                shadowed: alias.parameters.clone(),
            };
            let resolved = definition
                .resolve(&alias.target, visiting)
                .map(|mut target| {
                    substitute(&mut target, &alias.parameters, &arguments);
                    target
                });
            visiting.pop();
            return resolved;
        }
        Some(match annotation {
            TypeExpression::Generic { .. } => TypeExpression::Generic {
                name: name.clone(),
                arguments,
            },
            _ => annotation.clone(),
        })
    }
}

fn substitute(target: &mut TypeExpression, parameters: &[String], arguments: &[TypeExpression]) {
    match target {
        TypeExpression::Name(name) => {
            if let Some(index) = parameters.iter().position(|parameter| parameter == name) {
                *target = arguments[index].clone();
            }
        }
        TypeExpression::Generic {
            arguments: members, ..
        }
        | TypeExpression::Union(members)
        | TypeExpression::Intersection(members) => {
            for member in members {
                substitute(member, parameters, arguments);
            }
        }
        TypeExpression::Function {
            parameters: inputs,
            result,
        } => {
            for input in inputs {
                substitute(input, parameters, arguments);
            }
            substitute(result, parameters, arguments);
        }
        TypeExpression::Typeof(_) => {}
    }
}

pub(super) fn synchronize(program: &mut Program) {
    program.statements = program
        .entries
        .iter()
        .filter_map(|entry| match entry {
            ProgramEntry::Statement(statement) => Some(statement.clone()),
            ProgramEntry::Declaration(_) => None,
        })
        .collect();
    program.declarations = program
        .entries
        .iter()
        .filter_map(|entry| match entry {
            ProgramEntry::Declaration(declaration) => Some(declaration.clone()),
            ProgramEntry::Statement(_) => None,
        })
        .collect();
}
