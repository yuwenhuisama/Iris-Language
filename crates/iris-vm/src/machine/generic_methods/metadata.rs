use crate::compile::{
    Program,
    decorators::{Application, Target},
};
use crate::machine::{Machine, MachineError};
use iris_runtime::{ClassId, ImmutableHash, Value};
use iris_syntax::TypeExpression;

impl Machine {
    fn declaration_record(&mut self, fields: Vec<(&str, Value)>) -> Result<Value, MachineError> {
        let record = ImmutableHash::new(
            fields
                .into_iter()
                .map(|(name, value)| (Value::Symbol(name.into()), value))
                .collect(),
            Value::Type(self.builtin_class("Symbol")?, Vec::new()),
            Value::Type(self.builtin_class("Object")?, Vec::new()),
        );
        self.decorator_metadata.push(record.clone());
        Ok(Value::ImmutableHash(record))
    }

    pub(in crate::machine) fn decorator_signature_metadata(
        &mut self,
        application: &Application,
        context: (&Program, &[ClassId]),
    ) -> Result<Value, MachineError> {
        let (program, _) = context;
        let signature = program.functions[application
            .method
            .ok_or(MachineError::UnsupportedConstruct)?]
        .signature
        .as_ref()
        .ok_or(MachineError::UnsupportedConstruct)?;
        let mut unbound = signature.type_parameters.clone();
        match application.target {
            Target::Class(index) | Target::Reopen { class: index, .. } => {
                unbound.extend(program.classes[index].type_parameters.iter().cloned())
            }
            Target::Module(_) | Target::Contract(_) => {}
        }
        let mut parameters = Vec::new();
        for parameter in &signature.parameters {
            let annotation = parameter
                .annotation
                .clone()
                .unwrap_or(TypeExpression::Name("Object".into()));
            let annotation = self.declaration_annotation((&annotation, &unbound), context)?;
            let category = match parameter.category {
                iris_syntax::ParameterCategory::Positional => "positional",
                iris_syntax::ParameterCategory::Keyword => "keyword",
                iris_syntax::ParameterCategory::Rest => "rest",
                iris_syntax::ParameterCategory::KeywordRest => "keyword_rest",
                iris_syntax::ParameterCategory::Block => "block",
            };
            parameters.push(self.declaration_record(vec![
                ("name", Value::Symbol(parameter.name.clone())),
                ("type", annotation),
                ("category", Value::Symbol(category.into())),
                ("optional", Value::Bool(parameter.default.is_some())),
            ])?);
        }
        self.declaration_record(vec![
            ("parameters", Value::ReadonlyArray(parameters)),
            ("is_async", Value::Bool(signature.is_async)),
        ])
    }

    fn declaration_annotation(
        &mut self,
        (annotation, unbound): (&TypeExpression, &[String]),
        context: (&Program, &[ClassId]),
    ) -> Result<Value, MachineError> {
        match annotation {
            TypeExpression::Name(name) if unbound.contains(name) => self.declaration_record(vec![
                ("kind", Value::Symbol("parameter".into())),
                ("name", Value::Symbol(name.clone())),
            ]),
            TypeExpression::Name(_) | TypeExpression::Typeof(_) => {
                self.reify_type(annotation, context.0, context.1)
            }
            TypeExpression::Generic { name, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.declaration_annotation((argument, unbound), context))
                    .collect::<Result<Vec<_>, _>>()?;
                self.declaration_record(vec![
                    ("kind", Value::Symbol("generic".into())),
                    ("name", Value::Symbol(name.clone())),
                    ("arguments", Value::ReadonlyArray(arguments)),
                ])
            }
            TypeExpression::Function { parameters, result } => {
                let parameters = parameters
                    .iter()
                    .map(|parameter| self.declaration_annotation((parameter, unbound), context))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = self.declaration_annotation((result, unbound), context)?;
                self.declaration_record(vec![
                    ("kind", Value::Symbol("function".into())),
                    ("parameters", Value::ReadonlyArray(parameters)),
                    ("result", result),
                ])
            }
            TypeExpression::Union(members) | TypeExpression::Intersection(members) => {
                let kind = if matches!(annotation, TypeExpression::Union(_)) {
                    "union"
                } else {
                    "intersection"
                };
                let members = members
                    .iter()
                    .map(|member| self.declaration_annotation((member, unbound), context))
                    .collect::<Result<Vec<_>, _>>()?;
                self.declaration_record(vec![
                    ("kind", Value::Symbol(kind.into())),
                    ("members", Value::ReadonlyArray(members)),
                ])
            }
        }
    }
}
