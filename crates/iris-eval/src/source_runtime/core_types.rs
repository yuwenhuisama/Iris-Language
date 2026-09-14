use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{BuiltinClass, NominalType, Value};

impl SourceEvaluator {
    pub(super) fn async_result_type(
        &mut self,
        annotation: Option<&iris_syntax::TypeExpression>,
    ) -> Result<Value, EvaluationError> {
        match annotation {
            Some(annotation) => self.reify_type(annotation),
            None => self
                .kernel
                .class(BuiltinClass::Object)
                .map(|class| Value::Type(class, Vec::new()))
                .map_err(EvaluationError::Runtime),
        }
    }

    pub(super) fn allocate_task(&mut self, result: Value) -> iris_runtime::ObjectId {
        let identity = self.next_context_identity();
        self.task_types.insert(identity, result);
        identity
    }

    pub(super) fn core_type_test(
        &self,
        value: &Value,
        target: &Value,
    ) -> Result<Option<bool>, EvaluationError> {
        let (target, arguments) = match target {
            Value::Class(class) => (*class, &[][..]),
            Value::Type(class, arguments) | Value::ClosedClass(class, arguments) => {
                (*class, arguments.as_slice())
            }
            _ => return Ok(None),
        };
        if self
            .kernel
            .class(BuiltinClass::Object)
            .is_ok_and(|class| class == target)
        {
            return Ok(Some(true));
        }
        if self.kernel.core_class("Task") == Some(target) {
            return Ok(Some(match (value, arguments) {
                (Value::Task(_), []) => true,
                (Value::Task(identity), [result]) => {
                    self.task_types.get(identity)
                        == Some(&Value::Type(result.class(), result.arguments().to_vec()))
                }
                _ => false,
            }));
        }
        if super::decorator_core::RECORDS
            .iter()
            .any(|name| self.kernel.core_class(name) == Some(target))
        {
            return Ok(Some(
                matches!(value, Value::Decorator(record) if self.kernel.core_class(record.core_name()) == Some(target))
                    && arguments.is_empty(),
            ));
        }
        let class = match value {
            Value::Decorator(record) => self.kernel.core_class(record.core_name()),
            Value::Array(_) | Value::ImmutableArray(_) => self.kernel.core_class("Array"),
            Value::Hash(_) | Value::ImmutableHash(_) => self.kernel.core_class("Hash"),
            Value::Symbol(_) => self.class_name("Kernel::Symbol")?,
            Value::Tuple(_) => self.class_name("Kernel::Tuple")?,
            Value::Type(..) | Value::ComposedType(_) => self.class_name("Kernel::Type")?,
            _ => return Ok(None),
        };
        if class != Some(target) {
            return Ok(Some(false));
        }
        if arguments.is_empty() {
            return Ok(Some(true));
        }
        let matches = |held: &Value, expected: &NominalType| {
            *held == Value::Type(expected.class(), expected.arguments().to_vec())
        };
        Ok(Some(match (value, arguments) {
            (Value::ImmutableArray(array), [element]) => matches(array.element_type(), element),
            (Value::ImmutableHash(hash), [key, value]) => {
                matches(hash.key_type(), key) && matches(hash.value_type(), value)
            }
            _ => false,
        }))
    }
}
