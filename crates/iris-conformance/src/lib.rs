//! Schema checker and GRAMMAR vector runner.

mod json;
mod model;
mod observation;
mod runner;
mod runtime_observation;

pub use model::{Chapter, Corpus};
pub use runner::{Outcome, Report, diagnostics, execute, execute_runtime, report};

pub fn run(corpus: &Corpus) -> Result<Report, String> {
    Ok(report(&execute(&corpus.records()?)))
}

#[cfg(test)]
mod tests {
    use super::{Corpus, Outcome, run};

    #[test]
    fn loads_all_committed_records() -> Result<(), String> {
        // Given
        let corpus = Corpus::workspace()?;

        // When
        let records = corpus.records()?;

        // Then
        assert_eq!(records.len(), 41);
        Ok(())
    }

    #[test]
    fn report_buckets_cover_every_loaded_record() -> Result<(), String> {
        // Given
        let corpus = Corpus::workspace()?;

        // When
        let report = run(&corpus)?;

        // Then
        assert_eq!(report.total(), 41);
        Ok(())
    }

    #[test]
    fn normalizes_numeric_separator_diagnostic_from_source() {
        // Given
        let source = "1__0";

        // When
        let diagnostics = super::diagnostics(source);

        // Then
        assert!(
            diagnostics
                .iter()
                .any(|value| value.code == "LEX_BAD_NUMERIC_SEPARATOR")
        );
    }

    #[test]
    fn corrupted_expectation_is_reported_as_failed() -> Result<(), String> {
        // Given
        let corpus = Corpus::workspace()?;
        let mut records = corpus.records()?;
        let record = records
            .iter_mut()
            .find(|record| record.id == "IRIS-V1-GRAMMAR-V144")
            .ok_or("missing V144")?;
        record.expect = record.expect.replace("\"10\"", "\"11\"");

        // When
        let outcomes = super::execute(&records);

        // Then
        assert!(matches!(
            outcomes
                .iter()
                .find(|outcome| outcome.id() == "IRIS-V1-GRAMMAR-V144"),
            Some(Outcome::Failed { .. })
        ));
        Ok(())
    }

    #[test]
    fn prose_fixture_tag_is_reported_as_unrunnable_source() -> Result<(), String> {
        // Given
        let corpus = Corpus::workspace()?;
        let records = corpus.records()?;

        // When
        let outcomes = super::execute(&records);

        // Then
        assert!(matches!(
            outcomes
                .iter()
                .find(|outcome| outcome.id() == "IRIS-V1-GRAMMAR-V177"),
            Some(Outcome::UnrunnableSource { .. })
        ));
        Ok(())
    }

    #[test]
    fn runtime_records_load_every_authored_record() -> Result<(), String> {
        // Given
        let corpus = Corpus::workspace()?;

        // When
        let records = corpus.runtime_records()?;

        // Then
        assert_eq!(records.len(), 109);
        Ok(())
    }

    #[test]
    fn runtime_expected_raise_is_a_pass() -> Result<(), String> {
        // Given
        let corpus = Corpus::workspace()?;
        let records = corpus.runtime_records()?;

        // When
        let outcomes = super::execute_runtime(&records);

        // Then
        assert!(matches!(
            outcomes
                .iter()
                .find(|outcome| outcome.id() == "IRIS-V1-RUNTIME-V060"),
            Some(Outcome::Passed { .. })
        ));
        Ok(())
    }

    #[test]
    fn runtime_bucket_counts_cover_every_authored_record() -> Result<(), String> {
        // Given
        let corpus = Corpus::workspace()?;
        let records = corpus.runtime_records()?;

        // When
        let outcomes = super::execute_runtime(&records);
        let report = super::report(&outcomes);

        // Then
        assert_eq!(report.total(), records.len());
        Ok(())
    }

    #[test]
    fn runtime_non_executable_record_never_passes() -> Result<(), String> {
        // Given
        let corpus = Corpus::workspace()?;
        let records = corpus.runtime_records()?;

        // When
        let outcomes = super::execute_runtime(&records);

        // Then
        assert!(matches!(
            outcomes
                .iter()
                .find(|outcome| outcome.id() == "IRIS-V1-RUNTIME-V053"),
            Some(Outcome::NeedsSubsystem { .. })
        ));
        Ok(())
    }

    #[test]
    fn corrupting_runtime_expectation_is_reported_as_failed() -> Result<(), String> {
        // Given
        let corpus = Corpus::workspace()?;
        let mut records = corpus.runtime_records()?;
        let record = records
            .iter_mut()
            .find(|record| record.id == "IRIS-V1-RUNTIME-V019")
            .ok_or("missing V019")?;
        record.expect = record.expect.replace("true", "false");

        // When
        let outcomes = super::execute_runtime(&records);

        // Then
        assert!(matches!(
            outcomes
                .iter()
                .find(|outcome| outcome.id() == "IRIS-V1-RUNTIME-V019"),
            Some(Outcome::Failed { .. })
        ));
        Ok(())
    }
}
