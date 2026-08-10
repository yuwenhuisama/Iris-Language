use crate::{model::Record, observation::compare, runtime_observation::compare_runtime};
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
    NeedsSubsystem {
        id: String,
    },
    NoFixture {
        id: String,
    },
    Differential {
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
            | Self::UnrunnableSource { id }
            | Self::NeedsSubsystem { id }
            | Self::NoFixture { id }
            | Self::Differential { id } => id,
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
    pub needs_subsystem: usize,
    pub no_fixture: usize,
    pub differential: usize,
}
impl Report {
    pub fn total(self) -> usize {
        self.passed
            + self.failed
            + self.deferred
            + self.authored_expect
            + self.unrunnable_source
            + self.needs_subsystem
            + self.no_fixture
            + self.differential
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
    let parsed = parse(source);
    values.extend(parsed.diagnostics.iter().map(|value| Diagnostic {
        code: value.code.into(),
        severity: "error".into(),
        phase: "parse".into(),
        clause: String::new(),
    }));
    // A malformed source never produces a well-formed program, so static
    // analysis runs only once parsing ACCEPTED the program. Reporting both
    // would attribute parse damage to the static phase.
    if parsed.program_accepted {
        values.extend(
            iris_parser::analyze(&parsed.program)
                .iter()
                .map(|value| Diagnostic {
                    code: value.code.into(),
                    severity: "error".into(),
                    phase: "static".into(),
                    clause: String::new(),
                }),
        );
    }
    values.sort_by(|left, right| left.code.cmp(&right.code));
    values.dedup_by(|left, right| left.code == right.code);
    values
}

pub fn execute(records: &[Record]) -> Vec<Outcome> {
    records.iter().map(execute_record).collect()
}
pub fn execute_runtime(records: &[Record]) -> Vec<Outcome> {
    records.iter().map(execute_runtime_record).collect()
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
                Outcome::NeedsSubsystem { .. } => report.needs_subsystem += 1,
                Outcome::NoFixture { .. } => report.no_fixture += 1,
                Outcome::Differential { .. } => report.differential += 1,
            };
            report
        })
}

/// Validates a vector record's shape for a CONFORMANCE row.
///
/// `IRIS-V1-CONFORMANCE-C016` and the surrounding clauses fix what a record
/// must carry, so the fixture under test is ordinary JSON and the expectation
/// names the fields that must hold. Nothing is executed.
fn validate_record_shape(record: &Record) -> Outcome {
    let checked = || -> Result<(), String> {
        let fixture = crate::model::parse_expect(&record.source)?;
        let fixture = crate::model::object(&fixture)?;
        let expected = crate::model::parse_expect(&record.expect)?;
        let expected = crate::model::object(&expected)?;
        let Some(fields) = expected.get("record_fields") else {
            return Err("record_fields expected".into());
        };
        for (path, wanted) in crate::model::object(fields)? {
            let mut current = fixture.get(path.split('.').next().unwrap_or(path));
            for step in path.split('.').skip(1) {
                current = current
                    .and_then(|value| crate::model::object(value).ok())
                    .and_then(|value| value.get(step));
            }
            let Some(found) = current else {
                return Err(format!("record field {path} missing"));
            };
            // `json::Value` has no equality, so the comparison is over a
            // canonical rendering of each side.
            if render_json(found) != render_json(wanted) {
                return Err(format!(
                    "record field {path}: expected {}, found {}",
                    render_json(wanted),
                    render_json(found)
                ));
            }
        }
        Ok(())
    };
    match checked() {
        Ok(()) => Outcome::Passed {
            id: record.id.clone(),
        },
        Err(reason) => Outcome::Failed {
            id: record.id.clone(),
            expected: "record shape to validate".into(),
            actual: reason,
        },
    }
}

/// Renders a parsed JSON value canonically for comparison.
fn render_json(value: &crate::json::Value) -> String {
    match value {
        crate::json::Value::Array(values) => format!(
            "[{}]",
            values.iter().map(render_json).collect::<Vec<_>>().join(",")
        ),
        crate::json::Value::Bool(flag) => flag.to_string(),
        crate::json::Value::Null => "null".into(),
        crate::json::Value::Number => "number".into(),
        crate::json::Value::Object(entries) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|(key, held)| format!("{key}:{}", render_json(held)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        crate::json::Value::String(text) => format!("{text:?}"),
    }
}

/// Validates a documentation claim for an IDENTITY row.
///
/// The row states facts about the artifact tree, so each is checked against the
/// workspace: a file count under a directory, a required path, or a minimum
/// number of matches for a pattern in a file. Nothing is executed.
fn validate_documentation(record: &Record) -> Outcome {
    let checked = || -> Result<(), String> {
        let expected = crate::model::parse_expect(&record.expect)?;
        let expected = crate::model::object(&expected)?;
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        if let Some(counts) = expected.get("artifact_counts") {
            for (directory, wanted) in crate::model::object(counts)? {
                let crate::json::Value::String(wanted) = wanted else {
                    return Err("artifact count expects a string".into());
                };
                let found = std::fs::read_dir(root.join(directory))
                    .map_err(|error| error.to_string())?
                    .filter(|entry| {
                        entry.as_ref().is_ok_and(|entry| {
                            entry.path().extension().is_some_and(|kind| kind == "md")
                        })
                    })
                    .count();
                if found.to_string() != *wanted {
                    return Err(format!(
                        "{directory}: expected {wanted} artifacts, found {found}"
                    ));
                }
            }
        }
        if let Some(paths) = expected.get("required_paths") {
            for path in crate::model::array(paths)? {
                let crate::json::Value::String(path) = path else {
                    return Err("required path expects a string".into());
                };
                if !root.join(path).exists() {
                    return Err(format!("required path {path} is absent"));
                }
            }
        }
        if let Some(claims) = expected.get("declares") {
            for (path, wanted) in crate::model::object(claims)? {
                let crate::json::Value::String(wanted) = wanted else {
                    return Err("declaration expects a string".into());
                };
                let text =
                    std::fs::read_to_string(root.join(path)).map_err(|error| error.to_string())?;
                if !text.to_lowercase().contains(&wanted.to_lowercase()) {
                    return Err(format!("{path} does not declare {wanted}"));
                }
            }
        }
        Ok(())
    };
    match checked() {
        Ok(()) => Outcome::Passed {
            id: record.id.clone(),
        },
        Err(reason) => Outcome::Failed {
            id: record.id.clone(),
            expected: "documentation claim to hold".into(),
            actual: reason,
        },
    }
}

/// Runs one C ABI scenario for an FFI row.
///
/// The FFI vector table describes behaviour AT the C ABI, so a scenario names
/// an ABI operation and the row states the status it must answer. Nothing here
/// evaluates Iris source.
fn validate_abi_scenario(record: &Record) -> Outcome {
    use iris_abi::{HandleTable, IrisHandleKind, IrisStatus, Post, PostQueue, ThreadAffinity};

    let observed = match record.source.trim() {
        // C008: a live handle keeps denoting its target across a collection
        // cycle, which the table models by rooting the value it owns.
        "handle_survives_collection" => {
            let mut table = HandleTable::new(1);
            let handle = table.retain(41_i64, IrisHandleKind::ExplicitRelease);
            let roots: Vec<_> = table.roots().copied().collect();
            match table.get(handle) {
                Ok(41) if roots == vec![41] => "success",
                _ => "unexpected",
            }
        }
        // C009: a handle belongs to exactly one runtime.
        "handle_from_another_runtime" => {
            let mut first = HandleTable::new(1);
            let second = HandleTable::<i64>::new(2);
            let handle = first.retain(41, IrisHandleKind::ExplicitRelease);
            status_name(second.get(handle).err())
        }
        // C009: a released handle does not denote its slot after reuse.
        "released_handle_after_reuse" => {
            let mut table = HandleTable::new(1);
            let stale = table.retain(41_i64, IrisHandleKind::ExplicitRelease);
            table.release(stale);
            table.retain(7_i64, IrisHandleKind::ExplicitRelease);
            status_name(table.get(stale).err())
        }
        // C012: a worker thread may not read through a handle.
        "worker_thread_reads_handle" => {
            let affinity = ThreadAffinity::bind_current();
            let observed = std::thread::scope(|scope| scope.spawn(|| affinity.check()).join());
            match observed {
                Ok(status) => status_name(Some(status)),
                Err(_) => "unexpected",
            }
        }
        // C037: one completion token authorizes exactly one completion.
        "duplicate_completion_token" => {
            let queue = PostQueue::new();
            let post = || Post {
                token: 7,
                payload: vec![9],
            };
            if queue.post(post()) != IrisStatus::Success {
                "unexpected"
            } else {
                status_name(Some(queue.post(post())))
            }
        }
        // C039: an ABI major mismatch rejects attachment.
        "abi_major_mismatch" => status_name(iris_abi::attach(2, 0).err()),
        // C039: a minor-compatible record appends fields, and a reader only
        // reads what the supplied size covers.
        "abi_minor_size_tagged" => {
            let Ok(table) = iris_abi::attach(1, 0) else {
                return failed(record, "a compatible request attaches");
            };
            let older = iris_abi::IrisAbiTable { size: 8, ..table };
            if table.covers(16) && !older.covers(16) && older.covers(8) {
                "success"
            } else {
                "unexpected"
            }
        }
        other => return failed(record, &format!("unknown ABI scenario {other}")),
    };

    let expected = crate::model::parse_expect(&record.expect)
        .ok()
        .and_then(|value| {
            crate::model::object(&value)
                .ok()
                .and_then(|value| value.get("abi_status").cloned())
        });
    match expected {
        Some(crate::json::Value::String(wanted)) if wanted == observed => Outcome::Passed {
            id: record.id.clone(),
        },
        Some(crate::json::Value::String(wanted)) => Outcome::Failed {
            id: record.id.clone(),
            expected: wanted,
            actual: observed.to_owned(),
        },
        _ => failed(record, "abi_status expected"),
    }
}

/// The stable name of an ABI status, for a vector to state.
fn status_name(status: Option<iris_abi::IrisStatus>) -> &'static str {
    match status {
        None => "success",
        Some(iris_abi::IrisStatus::Success) => "success",
        Some(iris_abi::IrisStatus::InvalidHandle) => "invalid-handle",
        Some(iris_abi::IrisStatus::InvalidRuntime) => "invalid-runtime",
        Some(iris_abi::IrisStatus::ThreadAffinity) => "thread-affinity",
        Some(iris_abi::IrisStatus::Raised) => "raised",
        Some(iris_abi::IrisStatus::DuplicateCompletion) => "duplicate-completion",
        Some(iris_abi::IrisStatus::IncompatibleAbi) => "incompatible-abi",
        Some(iris_abi::IrisStatus::InvalidArgument) => "invalid-argument",
        Some(iris_abi::IrisStatus::InvalidBoundary) => "invalid-boundary",
    }
}

fn failed(record: &Record, detail: &str) -> Outcome {
    Outcome::Failed {
        id: record.id.clone(),
        expected: "a stated ABI status".into(),
        actual: detail.to_owned(),
    }
}

fn execute_runtime_record(record: &Record) -> Outcome {
    match record
        .tags
        .iter()
        .find_map(|tag| tag.strip_prefix("bucket:"))
    {
        Some("needs-subsystem") => Outcome::NeedsSubsystem {
            id: record.id.clone(),
        },
        Some("no-fixture") => Outcome::NoFixture {
            id: record.id.clone(),
        },
        Some("differential") => Outcome::Differential {
            id: record.id.clone(),
        },
        // A CONFORMANCE record-validation row observes the SHAPE of a vector
        // record rather than any language behaviour, so it is checked as data
        // against its own stated expectation instead of being executed.
        Some("record-validation") => validate_record_shape(record),
        // An FFI row observes C ABI behaviour: handle validity, runtime
        // ownership, thread affinity, completion tokens and ABI negotiation.
        // Those are not language behaviour, so the scenario names an ABI
        // operation and the row states the status it must answer.
        Some("abi") => validate_abi_scenario(record),
        // An IDENTITY documentation row asserts facts about the artifact tree
        // itself, so it is checked against the workspace rather than executed.
        Some("documentation") => validate_documentation(record),
        Some("executable") | None => match compare_runtime(record) {
            Ok(()) => Outcome::Passed {
                id: record.id.clone(),
            },
            Err(actual) => Outcome::Failed {
                id: record.id.clone(),
                expected: record.expect.clone(),
                actual,
            },
        },
        Some(bucket) => Outcome::Failed {
            id: record.id.clone(),
            expected: "a recognized runtime bucket".into(),
            actual: format!("unrecognized runtime bucket {bucket:?}"),
        },
    }
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
