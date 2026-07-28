use crate::{model::Record, observation::compare};
use iris_lexer::{convert_literals, lex};
use iris_parser::parse;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: String,
    pub phase: String,
    pub clause: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    Passed {
        id: String,
    },
    Failed {
        id: String,
        expected: String,
        actual: String,
    },
    Deferred {
        id: String,
    },
    AuthoredExpect {
        id: String,
    },
    UnrunnableSource {
        id: String,
    },
}
impl Outcome {
    pub fn id(&self) -> &str {
        match self {
            Self::Passed { id }
            | Self::Failed { id, .. }
            | Self::Deferred { id }
            | Self::AuthoredExpect { id }
            | Self::UnrunnableSource { id } => id,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Report {
    pub passed: usize,
    pub failed: usize,
    pub deferred: usize,
    pub authored_expect: usize,
    pub unrunnable_source: usize,
}
impl Report {
    pub fn total(self) -> usize {
        self.passed + self.failed + self.deferred + self.authored_expect + self.unrunnable_source
    }
}

pub fn diagnostics(source: &str) -> Vec<Diagnostic> {
    let mut values = Vec::new();
    let lexical = lex(source.as_bytes());
    values.extend(lexical.diagnostics().iter().map(|value| Diagnostic {
        code: value.code().into(),
        severity: "error".into(),
        phase: "lex".into(),
        clause: String::new(),
    }));
    let conversion = convert_literals(source);
    values.extend(conversion.diagnostics().iter().map(|code| Diagnostic {
        code: (*code).into(),
        severity: "error".into(),
        phase: "lex".into(),
        clause: String::new(),
    }));
    values.extend(conversion.warnings().iter().map(|_| Diagnostic {
        code: "PRECISION_WARNING".into(),
        severity: "warning".into(),
        phase: "lex".into(),
        clause: String::new(),
    }));
    values.extend(parse(source).diagnostics.iter().map(|value| Diagnostic {
        code: value.code.into(),
        severity: "error".into(),
        phase: "parse".into(),
        clause: String::new(),
    }));
    values.sort_by(|left, right| left.code.cmp(&right.code));
    values.dedup_by(|left, right| left.code == right.code);
    values
}

pub fn execute(records: &[Record]) -> Vec<Outcome> {
    records.iter().map(execute_record).collect()
}
pub fn report(outcomes: &[Outcome]) -> Report {
    outcomes
        .iter()
        .fold(Report::default(), |mut report, outcome| {
            match outcome {
                Outcome::Passed { .. } => report.passed += 1,
                Outcome::Failed { .. } => report.failed += 1,
                Outcome::Deferred { .. } => report.deferred += 1,
                Outcome::AuthoredExpect { .. } => report.authored_expect += 1,
                Outcome::UnrunnableSource { .. } => report.unrunnable_source += 1,
            };
            report
        })
}

fn execute_record(record: &Record) -> Outcome {
    if record.tags.iter().any(|tag| tag == "status:deferred") {
        return Outcome::Deferred {
            id: record.id.clone(),
        };
    }
    if record
        .tags
        .iter()
        .any(|tag| tag == "status:authored-expect")
    {
        return Outcome::AuthoredExpect {
            id: record.id.clone(),
        };
    }
    if record.tags.iter().any(|tag| tag == "status:prose-fixture") {
        return Outcome::UnrunnableSource {
            id: record.id.clone(),
        };
    }
    let parsed = parse(&record.source);
    match compare(record, &parsed) {
        Ok(()) => Outcome::Passed {
            id: record.id.clone(),
        },
        Err(actual) => Outcome::Failed {
            id: record.id.clone(),
            expected: record.expect.clone(),
            actual,
        },
    }
}
