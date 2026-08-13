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
        // `IRIS-V1-IDENTITY-V002` accepts a corpus only when the identity
        // surfaces are present AND not replaced by source-compatible Legacy
        // Iris restoration claims. Presence alone cannot show the second half,
        // so a row may also state text that MUST NOT appear.
        if let Some(claims) = expected.get("denies") {
            for (path, unwanted) in crate::model::object(claims)? {
                let crate::json::Value::String(unwanted) = unwanted else {
                    return Err("denial expects a string".into());
                };
                let text =
                    std::fs::read_to_string(root.join(path)).map_err(|error| error.to_string())?;
                if text.to_lowercase().contains(&unwanted.to_lowercase()) {
                    return Err(format!("{path} claims {unwanted}"));
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

    let mut observed_diagnostic: Option<&'static str> = None;
    let observed = match record.source.trim() {
        // C010: a scoped handle stays STRONGLY ROOTED until its frame closes,
        // and closing the frame bulk-releases every live handle it owns.
        "scoped_frame_release" => {
            let mut table = HandleTable::new(1);
            let frame = table.open_frame();
            let scoped = table.retain(41_i64, IrisHandleKind::Scoped);
            let explicit = table.retain(7_i64, IrisHandleKind::ExplicitRelease);
            let rooted_while_open = table.get(scoped).is_ok();
            let closed = table.close_frame(frame);
            // The explicit handle is NOT owned by the frame, so it must survive
            // the close; otherwise the row would pass for an implementation
            // that simply cleared the whole table.
            if rooted_while_open
                && closed == IrisStatus::Success
                && table.get(scoped) == Err(IrisStatus::InvalidHandle)
                && table.get(explicit) == Ok(&7)
            {
                "success"
            } else {
                "unexpected"
            }
        }
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
        // V060: a C-compiled fixture attaches and reads the negotiated
        // versions back, which only holds if the record layout is the C one.
        "c_extension_attach" => {
            let mut major = 0_u32;
            let mut minor = 0_u32;
            let status = iris_abi::fixture_host_abi_v1(&raw mut major, &raw mut minor);
            if status == IrisStatus::Success as i32
                && major == iris_abi::ABI_MAJOR
                && minor == iris_abi::ABI_MINOR
            {
                "success"
            } else {
                "unexpected"
            }
        }
        // V061: C creates, reads, releases, then reads again. The second read
        // must be refused, so the C009 generation check survives the crossing.
        "c_rooted_handle_release" => {
            iris_abi::iris_runtime_reset();
            let mut before = 0_i64;
            let mut after = 0_i32;
            let status = iris_abi::fixture_rooted_handle(&raw mut before, &raw mut after);
            if status == IrisStatus::Success as i32
                && before == 41
                && after == IrisStatus::InvalidHandle as i32
            {
                "invalid-handle"
            } else {
                "unexpected"
            }
        }
        // V062: a worker read is refused while its copied post is accepted.
        "c_worker_reads_handle" => {
            iris_abi::iris_runtime_reset();
            let mut handle = iris_abi::IrisHandle::NULL;
            // SAFETY: `handle` is a live local, so the out pointer is valid.
            unsafe { iris_abi::iris_int_create(41, &raw mut handle) };
            let observed = std::thread::scope(|scope| {
                scope
                    .spawn(move || {
                        let mut value = 0_i64;
                        let read = iris_abi::fixture_worker_reads_handle(handle, &raw mut value);
                        (read, value, iris_abi::fixture_worker_posts(4242, 7))
                    })
                    .join()
            });
            let Ok((read, value, posted)) = observed else {
                return failed(record, "the worker returns its statuses");
            };
            if read == IrisStatus::ThreadAffinity as i32
                && value == 0
                && posted == IrisStatus::Success as i32
            {
                "thread-affinity"
            } else {
                "unexpected"
            }
        }
        // V063: a native raise answers a status AND fills a context handle.
        "c_native_raise" => {
            iris_abi::iris_runtime_reset();
            let mut context = iris_abi::IrisHandle::NULL;
            let mut marker = 0_i64;
            let status = iris_abi::fixture_raise_marker(&raw mut context, &raw mut marker);
            if status == IrisStatus::Raised as i32 && !context.is_null() && marker == 41 {
                "raised"
            } else {
                "unexpected"
            }
        }
        // V066: one token authorizes exactly one completion.
        "c_post_twice_one_token" => {
            iris_abi::iris_runtime_reset();
            let mut second = 0_i32;
            let mut count = 0_u32;
            let mut value = 0_i64;
            let status =
                iris_abi::fixture_post_twice(7777, &raw mut second, &raw mut count, &raw mut value);
            if status == IrisStatus::Success as i32
                && second == IrisStatus::DuplicateCompletion as i32
                && count == 1
                && value == 9
            {
                "duplicate-completion"
            } else {
                "unexpected"
            }
        }
        // C040: handle IDs are RUNTIME-LOCAL. Two runtimes issue the same slot
        // as different ids, and neither id resolves in the other runtime, so an
        // id cannot serve as cross-run identity.
        "runtime_local_native_ids" => {
            let mut first = HandleTable::new(1);
            let mut second = HandleTable::new(2);
            let here = first.retain(41_i64, IrisHandleKind::ExplicitRelease);
            let there = second.retain(41_i64, IrisHandleKind::ExplicitRelease);
            let same_slot = here.slot() == there.slot();
            let distinct_runtime = here.runtime() != there.runtime();
            let distinct_id = here != there;
            let not_portable = second.get(here) == Err(IrisStatus::InvalidRuntime)
                && first.get(there) == Err(IrisStatus::InvalidRuntime);
            if same_slot && distinct_runtime && distinct_id && not_portable {
                "runtime-local"
            } else {
                "unexpected"
            }
        }
        // C036: posting after runtime shutdown answers a closed status, and
        // the drain refuses too, so no worker reaches Iris state after close.
        "post_after_shutdown" => {
            let queue = PostQueue::new();
            let accepted = queue.post(Post {
                token: 1,
                payload: vec![7],
            });
            queue.close();
            let refused = queue.post(Post {
                token: 2,
                payload: vec![9],
            });
            let affinity = ThreadAffinity::bind_current();
            let drained = queue.drain(&affinity);
            queue.clear();
            if accepted == IrisStatus::Success
                && refused == IrisStatus::InvalidRuntime
                && drained == Err(IrisStatus::InvalidRuntime)
            {
                "closed"
            } else {
                "unexpected"
            }
        }
        // V067: a major mismatch publishes no table.
        "c_negotiate_v2" => {
            let mut major = 0_u32;
            let status = iris_abi::fixture_negotiate_v2(&raw mut major);
            if status == IrisStatus::IncompatibleAbi as i32 && major == 0 {
                "incompatible-abi"
            } else {
                "unexpected"
            }
        }
        // V001: a raw pointer forged into a handle never crosses.
        "c_raw_pointer_handle" => {
            iris_abi::iris_runtime_reset();
            let mut value = 0_i64;
            let mut touched = 0_i32;
            let mut tagged = 0_i32;
            let status = iris_abi::fixture_raw_pointer_handle(
                &raw mut value,
                &raw mut touched,
                &raw mut tagged,
            );
            // `touched` is the load-bearing check: the pointee is 41, so
            // observing it through either forgery would mean the runtime
            // followed an address instead of an opaque table name.
            if status == IrisStatus::InvalidRuntime as i32
                && tagged != IrisStatus::Success as i32
                && touched == 0
                && value == 0
            {
                "invalid-runtime"
            } else {
                "unexpected"
            }
        }
        // V008: an unwind beneath the boundary becomes a status.
        "c_panic_does_not_cross" => {
            iris_abi::iris_runtime_reset();
            let mut value = 0_i64;
            let mut resumed = 0_i32;
            let status = iris_abi::fixture_panic_does_not_cross(&raw mut value, &raw mut resumed);
            // `resumed` is the real evidence: had the unwind crossed, the C
            // frame would never have reached the line that sets it.
            if status == IrisStatus::InvalidBoundary as i32 && resumed == 1 && value == -1 {
                "invalid-boundary"
            } else {
                "unexpected"
            }
        }
        // V012/V065: a Closeable native resource closes twice.
        "c_payload_close_twice" => {
            iris_abi::iris_runtime_reset();
            let mut second = 0_i32;
            let mut releases = 0_u32;
            let first = iris_abi::fixture_payload_close_twice(&raw mut second, &raw mut releases);
            // The release count is what separates an idempotent close from a
            // double release: both calls "succeeding" cannot show that alone.
            if first == IrisStatus::Success as i32
                && second == IrisStatus::Success as i32
                && releases == 1
            {
                "success"
            } else {
                "unexpected"
            }
        }
        // V013: a descriptor whose final cleanup may raise never registers.
        "c_payload_cleanup_may_raise" => {
            iris_abi::iris_runtime_reset();
            let mut diagnostic = 0_u32;
            let mut releases = 0_u32;
            let status =
                iris_abi::fixture_payload_cleanup_may_raise(&raw mut diagnostic, &raw mut releases);
            // A refused descriptor owns no storage, so `u32::MAX` here means
            // there was no payload at all rather than one that released zero
            // times.
            if status == IrisStatus::InvalidArgument as i32
                && diagnostic == 3
                && releases == u32::MAX
            {
                "invalid-argument"
            } else {
                "unexpected"
            }
        }
        // V011: a payload traces managed roots and keeps them alive.
        "c_payload_traces_root" => {
            iris_abi::iris_runtime_reset();
            let mut value = 0_i64;
            let mut reported = 0_u32;
            let mut live = 0_u32;
            let status = iris_abi::fixture_payload_traces_root(
                &raw mut value,
                &raw mut reported,
                &raw mut live,
            );
            let stale = iris_abi::fixture_payload_rejects_stale_root();
            // `live` and the stale refusal are load-bearing: reporting a root
            // that had already died would satisfy `reported` on its own.
            if status == IrisStatus::Success as i32
                && reported == 1
                && live == 1
                && value == 41
                && stale == IrisStatus::InvalidHandle as i32
            {
                "success"
            } else {
                "unexpected"
            }
        }
        // V005: a worker posts, and the runtime thread completes.
        "c_worker_completion_round_trip" => {
            iris_abi::iris_runtime_reset();
            let token = 5150_u64;
            let posted = std::thread::scope(|scope| {
                scope
                    .spawn(|| iris_abi::fixture_worker_completes(token, 23))
                    .join()
                    .unwrap_or(-1)
            });
            let mut seen_token = 0_u64;
            let mut value = 0_i64;
            let mut count = 0_u32;
            let status = iris_abi::fixture_runtime_takes_completion(
                &raw mut seen_token,
                &raw mut value,
                &raw mut count,
            );
            // The token coming back is what ties the delivered value to the
            // request that authorized it, rather than to any arrival.
            if posted == IrisStatus::Success as i32
                && status == IrisStatus::Success as i32
                && seen_token == token
                && value == 23
                && count == 1
            {
                "success"
            } else {
                "unexpected"
            }
        }
        // V009: an EXTENSION artifact digest mismatch aborts the load.
        "c_extension_digest_mismatch" => {
            let mut refused_major = 0_u32;
            let refused = iris_abi::fixture_extension_digest_mismatch(&raw mut refused_major);
            let mut accepted_major = 0_u32;
            let accepted = iris_abi::fixture_extension_digest_matches(&raw mut accepted_major);
            // The sentinel surviving shows no table was published. The matching
            // load attaching shows the refusal comes from verification rather
            // than from the extension path being broken outright.
            if refused == IrisStatus::IncompatibleAbi as i32
                && refused_major == 99
                && accepted == IrisStatus::Success as i32
                && accepted_major == iris_abi::ABI_MAJOR
            {
                "incompatible-abi"
            } else {
                "unexpected"
            }
        }
        // V064: a digest mismatch aborts the load before binding.
        "metadata_static_api_digest" => {
            match metadata_rejection("conformance/iris-v1/fixtures/metadata/static_api.json") {
                Err(reason) => return failed(record, &reason),
                Ok(None) => "unexpected",
                Ok(Some(rejection)) => {
                    observed_diagnostic = Some(rejection.diagnostic());
                    // The artifact exports a working entry, so a zero call
                    // count is what shows the refusal happened BEFORE binding
                    // rather than after something was already invoked.
                    if iris_abi::fixture_static_api_call_count() == 0 {
                        "metadata-mismatch"
                    } else {
                        "unexpected"
                    }
                }
            }
        }
        // V059: absent package identity refuses the load before any digest work.
        "metadata_manifestless_native" => {
            match metadata_rejection(
                "conformance/iris-v1/fixtures/metadata/manifestless_native.json",
            ) {
                Err(reason) => return failed(record, &reason),
                Ok(None) => "unexpected",
                Ok(Some(rejection)) => {
                    observed_diagnostic = Some(rejection.diagnostic());
                    if iris_abi::fixture_manifestless_call_count() == 0 {
                        "metadata-mismatch"
                    } else {
                        "unexpected"
                    }
                }
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
    // A row that names a diagnostic must observe THAT diagnostic, not merely a
    // refusal: `C022` and `C023` fail for different reasons and a load that
    // reported the wrong one would hide which check actually ran.
    if let Ok(value) = crate::model::parse_expect(&record.expect)
        && let Ok(fields) = crate::model::object(&value)
        && let Some(crate::json::Value::String(wanted)) = fields.get("diagnostic")
        && Some(wanted.as_str()) != observed_diagnostic
    {
        return Outcome::Failed {
            id: record.id.clone(),
            expected: wanted.clone(),
            actual: observed_diagnostic.unwrap_or("no diagnostic").to_owned(),
        };
    }
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

/// Verifies one native metadata record against the artifact it points at.
///
/// `IRIS-V1-FFI-C023` verifies the LOADED artifact against the resolved
/// metadata, so the digest is computed from the file's real bytes here rather
/// than restated in the vector. A vector that hard-coded the mismatch would
/// still pass if the verifier stopped checking.
fn metadata_rejection(relative: &str) -> Result<Option<iris_abi::ManifestRejection>, String> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(root.join(relative)).map_err(|error| error.to_string())?;
    let parsed = crate::json::parse(&text)?;
    let fields = crate::model::object(&parsed)?;
    let text_of = |key: &str| match fields.get(key) {
        Some(crate::json::Value::String(value)) => Some(value.clone()),
        _ => None,
    };
    let number_of = |key: &str| -> Result<u32, String> {
        text_of(key)
            .ok_or_else(|| format!("{key} missing"))?
            .parse::<u32>()
            .map_err(|error| error.to_string())
    };
    let artifact = text_of("artifact").ok_or("artifact missing")?;
    let bytes = std::fs::read(root.join(&artifact)).map_err(|error| error.to_string())?;
    let manifest = iris_abi::NativeManifest {
        package: text_of("package"),
        reflection_policy: text_of("reflection_policy"),
        digest: text_of("digest"),
        abi_major: number_of("abi_major")?,
        abi_minor: number_of("abi_minor")?,
    };
    let digest = iris_runtime::artifact_digest(&bytes);
    Ok(iris_abi::verify(&manifest, &digest, iris_abi::ABI_MAJOR).err())
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
