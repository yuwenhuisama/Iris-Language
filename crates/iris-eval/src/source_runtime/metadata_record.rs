use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ImmutableHash, Value};

impl SourceEvaluator {
    pub(super) fn metadata_record(
        &mut self,
        fields: Vec<(&str, Value)>,
    ) -> Result<Value, EvaluationError> {
        let types = self
            .snapshot_types
            .as_ref()
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let record = ImmutableHash::new(
            fields
                .into_iter()
                .map(|(name, value)| (Value::Symbol(name.into()), value))
                .collect(),
            types.symbol.clone(),
            types.object.clone(),
        );
        self.decorator_metadata.push(record.clone());
        Ok(Value::ImmutableHash(record))
    }
}
