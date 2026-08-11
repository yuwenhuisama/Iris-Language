#!/usr/bin/env python3
"""Run Iris snippets through the conformance runner and report what they answer.

Clause vectors must state what the implementation ACTUALLY does, verified
first, not what it is assumed to do. Writing an expectation from assumption
records a defect as the rule when the two disagree, which has already happened
in this project. This harness makes measuring cheap: it installs one scratch
vector per snippet, runs the chapter once, and prints the observed value.

Usage:
    python3 tools/probe.py CHAPTER 'name=source' ['name=source' ...]
    python3 tools/probe.py CHAPTER --file probes.txt   # one name=source per line
"""

from __future__ import annotations

import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

ARTIFACTS = {
    "RUNTIME": "spec/iris-v1/03-runtime-object-model.md",
    "GRAMMAR": "spec/iris-v1/02-lexical-grammar.md",
    "CONTROL": "spec/iris-v1/04-bindings-callables-control-flow.md",
    "TYPES": "spec/iris-v1/05-types-contracts-generics.md",
    "COLLECTIONS": "spec/iris-v1/06-collections-text-regex.md",
    "ASYNC": "spec/iris-v1/07-async-resources-diagnostics.md",
    "META": "spec/iris-v1/08-modules-metaprogramming.md",
    "FFI": "spec/iris-v1/09-native-host-ffi.md",
    "LIBRARY": "spec/iris-v1/10-serialization-standard-library.md",
    "IDENTITY": "spec/iris-v1/01-language-identity.md",
    "CONFORMANCE": "spec/iris-v1/12-conformance.md",
}

# A sentinel no program answers, so the runner always reports a mismatch and
# the mismatch text carries the observed value.
SENTINEL = {"value": {"symbol": "IRIS_PROBE_SENTINEL"}}
DIAGNOSTIC_SENTINEL = {"diagnostics": [{"code": "IRIS_PROBE_SENTINEL"}]}


def scratch_path(chapter: str) -> str:
    return os.path.join(
        ROOT, f"conformance/iris-v1/vectors/{chapter}/IRIS-V1-{chapter}-V000PROBE.json"
    )


def record(chapter: str, source: str) -> dict:
    return {
        "schema_version": "iris-v1-vector-schema-1",
        "id": f"IRIS-V1-{chapter}-V000PROBE",
        "name": "scratch probe",
        "category": "positive",
        "source": {
            "chapter": chapter,
            "artifact": ARTIFACTS[chapter],
            "clauses": [],
            "decisions": [],
        },
        "input": {"kind": "source", "source_text": source},
        "applicability": {
            "interpreter": "required",
            "jit": "required",
            "native": "not_applicable",
            "reason": "Scratch probe, never committed.",
        },
        "expect": SENTINEL,
        "tags": ["bucket:executable"],
    }


def observe(chapter: str, source: str, expect: dict | None = None) -> str:
    path = scratch_path(chapter)
    entry = record(chapter, source)
    if expect is not None:
        entry["expect"] = expect
        entry["category"] = "diagnostic"
        entry["tags"] = ["bucket:executable", "phase:parse"]
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(entry, handle)
    try:
        completed = subprocess.run(
            ["cargo", "run", "-q", "-p", "iris-conformance", "--", "--chapter", chapter],
            capture_output=True,
            text=True,
            cwd=ROOT,
            check=False,
        )
    finally:
        os.remove(path)
    # A parse diagnostic is reported on its own line rather than as a value,
    # so a probe that only reads the value line would show the program's
    # result and hide the rejection entirely.
    for line in completed.stdout.splitlines():
        if "diagnostics expected" in line:
            return line.rsplit("actual", 1)[1].strip()
    for line in completed.stdout.splitlines():
        if "actual:" in line:
            # A value mismatch repeats the expectation before the observation,
            # so keep only what follows the sentinel; an error line has neither.
            if "IRIS_PROBE_SENTINEL" in line:
                tail = line.split("IRIS_PROBE_SENTINEL", 1)[1]
                return tail.split("actual", 1)[1].strip() if "actual" in tail else tail.strip()
            return line.split("actual", 1)[1].lstrip(": ").strip()
    return "(no result; the program may not have run)"


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    chapter, rest = sys.argv[1], sys.argv[2:]
    if chapter not in ARTIFACTS:
        print(f"unknown chapter {chapter}; expected one of {', '.join(sorted(ARTIFACTS))}")
        return 2
    if rest[0] == "--file":
        with open(rest[1], encoding="utf-8") as handle:
            probes = [line.rstrip("\n") for line in handle if line.strip()]
    else:
        probes = rest
    for probe in probes:
        name, _, source = probe.partition("=")
        seen = observe(chapter, source)
        # A program rejected before evaluation reports no value, so ask again
        # for a diagnostic the runner cannot match and report what it saw.
        if "UnsupportedConstruct" in seen or "ParseDiagnostic" in seen:
            seen = f"{seen} | diagnostics {observe(chapter, source, DIAGNOSTIC_SENTINEL)}"
        print(f"{name[:36]:<38} -> {seen[:150]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
