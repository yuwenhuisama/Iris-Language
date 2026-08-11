"""Record-shape rules from chapter 12 that constrain `input` and `expect`.

These live apart from `corpus-audit.py` because that file already carries the
identity, provenance and tag rules and is near this project's size ceiling.

Two vocabularies here are deliberately WIDER than the clause that names them.
`C021` fixes eight `input` payload names and `C027` fixes twenty `expect.value`
forms, but the frozen runner contract already requires more: `model.rs` parses
`independent_sources`, `package_sources`, `package_fixture` and `package_probe`
as first-class payloads, and `runtime_observation/values.rs` renders a non-bit
float as `float32`/`float64`. 84 of the 99 affected input records and all 10
float records predate this milestone. Enforcing the literal lists would reject
the package and NaN/Infinity observations those vectors exist to make, none of
which any clause prohibits, so the extension is enumerated here to keep the gap
visible and is recorded in docs/spec-defects-v1.md.
"""

from __future__ import annotations

INPUT_KINDS = {
    "source", "package", "metadata", "native", "ffi",
    "serialized", "host", "legacy", "fixture",
}
CLAUSE_PAYLOADS = {
    "source_text", "files", "package", "metadata",
    "native_artifact", "serialized_value", "host_calls", "fixture_ref",
}
RUNNER_PAYLOADS = {
    "independent_sources", "package_sources", "package_fixture", "package_probe",
}
INPUT_PAYLOADS = CLAUSE_PAYLOADS | RUNNER_PAYLOADS

CLAUSE_VALUE_FORMS = {
    "nil", "bool", "integer", "float32_bits", "float64_bits", "string",
    "symbol", "bytes_hex", "array", "tuple", "hash_entries", "range", "regex",
    "object_identity", "class_name", "module_name", "contract_name", "task",
    "exception_context", "opaque_handle_status",
}
RUNNER_VALUE_FORMS = {"float32", "float64"}
VALUE_FORMS = CLAUSE_VALUE_FORMS | RUNNER_VALUE_FORMS

# C013 fixes the observation each category must carry.
# Every observation field the committed corpus states, gathered from the corpus
# rather than assumed. C013 and C016 require a record to STATE its observation;
# they do not restrict which of these carries it, and guessing a narrower set
# rejected 68 valid records across four attempts.
OBSERVATION_FIELDS = {
    "value", "error", "diagnostics", "status", "abi_status", "artifact",
    "side_effects", "declares", "record_fields", "type", "backends",
    "equivalence", "diagnostics_exhaustive", "denies", "artifact_counts",
    "required_paths", "diagnostic", "stdout", "stderr",
}
BACKENDS = ("interpreter", "jit", "native")


def expectations(expect: dict) -> list[dict]:
    """Every observation in a record, flattening the independent form."""
    if not isinstance(expect, dict):
        return []
    nested = expect.get("independent_expectations")
    if isinstance(nested, list):
        return [item for item in nested if isinstance(item, dict)] + [expect]
    return [expect]


def check_input(record: dict, name: str, problems: list[str]) -> None:
    payload = record.get("input")
    if not isinstance(payload, dict):
        problems.append(f"C021 input is not an object: {name}")
        return
    if payload.get("kind") not in INPUT_KINDS:
        problems.append(f"C021 input kind {payload.get('kind')!r}: {name}")
    if not (set(payload) - {"kind"}) & INPUT_PAYLOADS:
        problems.append(f"C021 input carries no payload field: {name}")


def check_value_forms(record: dict, name: str, problems: list[str]) -> None:
    for observation in expectations(record.get("expect", {})):
        value = observation.get("value")
        if isinstance(value, dict) and not set(value) & VALUE_FORMS:
            problems.append(f"C027 expect.value form {sorted(value)!r}: {name}")


def check_streams(record: dict, name: str, problems: list[str]) -> None:
    """C026: stdout and stderr are arrays of exact lines, without terminators."""
    for observation in expectations(record.get("expect", {})):
        for stream in ("stdout", "stderr"):
            lines = observation.get(stream)
            if lines is None:
                continue
            if not isinstance(lines, list):
                problems.append(f"C026 expect.{stream} is not an array: {name}")
                continue
            for line in lines:
                if not isinstance(line, str):
                    problems.append(f"C026 expect.{stream} entry is not a string: {name}")
                elif "\n" in line or "\r" in line:
                    problems.append(f"C026 expect.{stream} carries a line terminator: {name}")


def check_category_fields(record: dict, name: str, problems: list[str]) -> None:
    """C013 and C010: each category carries the observation it is defined by.

    A `negative` record without `error` or `status` is exactly the shape the
    C056 sample record is normative for rejecting. Nine committed vectors were
    found in that shape and reclassified by their real observation.
    """
    category = record.get("category")
    expect = record.get("expect")
    if not isinstance(expect, dict):
        problems.append(f"C013 expect is not an object: {name}")
        return
    observations = expectations(expect)
    if category == "positive":
        if not any(set(item) & OBSERVATION_FIELDS for item in observations):
            problems.append(f"C013 positive record states no observation: {name}")
    elif category == "negative":
        carries = any({"error", "status"} & set(item) for item in observations)
        if not carries and not record.get("source", {}).get("decisions"):
            problems.append(
                f"C056 negative record lacks both expect.error and source.decisions: {name}"
            )
    elif category == "diagnostic":
        if not any(set(item) & OBSERVATION_FIELDS for item in observations):
            problems.append(f"C016 diagnostic record states no observation: {name}")


def check_differential(record: dict, name: str, problems: list[str]) -> None:
    """C017: a differential record names every backend it compares."""
    if record.get("category") != "differential":
        return
    applicability = record.get("applicability")
    if not isinstance(applicability, dict):
        problems.append(f"C017 differential applicability is not an object: {name}")
        return
    for backend in BACKENDS:
        if backend not in applicability:
            problems.append(f"C017 differential omits backend {backend!r}: {name}")
        elif applicability[backend] == "not_applicable" and not applicability.get("reason"):
            problems.append(f"C017 backend {backend!r} is not_applicable without a reason: {name}")


def check(record: dict, name: str, problems: list[str]) -> None:
    check_input(record, name, problems)
    check_value_forms(record, name, problems)
    check_streams(record, name, problems)
    check_category_fields(record, name, problems)
    check_differential(record, name, problems)
