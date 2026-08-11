#!/usr/bin/env python3
"""Audit the conformance corpus against the clauses that govern the corpus.

Chapter 12 constrains the corpus itself, not language behaviour, so its rules
cannot be checked by a vector that runs a program. They are checked here.

Two corpus-wide violations were found by hand before this existed: 91 records
used a `category` outside the C013 vocabulary, and 111 locally authored ids
used a `V###A` spelling C008 forbids. Neither was visible to any test, which is
why the checks now live in a tool that runs on demand rather than in a
one-off script.

Usage:
    python3 tools/corpus-audit.py          # report violations, exit 1 if any
"""

from __future__ import annotations

import glob
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import corpus_freeze_rules
import corpus_manifest_rules
import corpus_record_rules

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

VECTOR_ID = re.compile(r"^IRIS-V1-[A-Z]+-V\d+$")
SCHEMA_VERSION = "iris-v1-vector-schema-1"
CATEGORIES = {"positive", "negative", "diagnostic", "differential"}
CHAPTERS = {
    "IDENTITY", "GRAMMAR", "RUNTIME", "CONTROL", "TYPES", "COLLECTIONS",
    "ASYNC", "META", "FFI", "LIBRARY", "MIGRATION", "CONFORMANCE", "TRACE",
}
TOP_LEVEL_FIELDS = {
    "schema_version", "id", "name", "category", "source", "input",
    "applicability", "expect", "tags",
}
APPLICABILITY_VALUES = {"required", "optional", "not_applicable", "prohibited"}
LEGACY_TAGS = {
    "legacy:preserve", "legacy:intentional-divergence", "legacy:removed",
    "legacy:deferred", "legacy:not-applicable",
}
CLAUSE_ID = re.compile(r"^IRIS-V1-[A-Z]+-C\d+[A-Z]?$")
DECISION_ID = re.compile(r"^D-\d{3}$")
TAG = re.compile(r"^[a-z0-9][a-z0-9:_-]*$")

# `IRIS-V1-CONFORMANCE-C008` admits digits only, but the CONTROL vector table
# PUBLISHES these ids with a trailing letter, and C008 equally requires a
# chapter-owned id to stay stable once published. The chapter table is
# authoritative for vector content, so these are preserved and the conflict is
# recorded in docs/spec-defects-v1.md rather than resolved by renaming them.
SPEC_PUBLISHED_EXCEPTIONS = {
    "IRIS-V1-CONTROL-V288A",
    "IRIS-V1-CONTROL-V289A",
    "IRIS-V1-CONTROL-V302A",
    "IRIS-V1-CONTROL-V335A",
    "IRIS-V1-CONTROL-V337A",
    "IRIS-V1-CONTROL-V339A",
    "IRIS-V1-CONTROL-V342A",
    "IRIS-V1-CONTROL-V344A",
    "IRIS-V1-CONTROL-V347A",
    "IRIS-V1-CONTROL-V351A",
    "IRIS-V1-CONTROL-V354A",
    "IRIS-V1-CONTROL-V355A",
    "IRIS-V1-CONTROL-V355B",
    "IRIS-V1-CONTROL-V357A",
}


def records() -> list[tuple[str, dict]]:
    found = []
    pattern = os.path.join(ROOT, "conformance/iris-v1/vectors/**/*.json")
    for path in sorted(glob.glob(pattern, recursive=True)):
        with open(path, encoding="utf-8") as handle:
            found.append((path, json.load(handle)))
    return found


def spec_vector_ids() -> set[str]:
    """Every vector id the frozen specification declares.

    The trailing letter is part of the id: capturing `V\\d+` alone truncates
    `V288A` to `V288` and makes a spec-published row look locally authored.
    """
    names: set[str] = set()
    for path in glob.glob(os.path.join(ROOT, "spec/iris-v1/*.md")):
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        for match in re.finditer(r"IRIS-V1-([A-Z]+)-(V\d+[A-Z]?)\b", text):
            names.add(f"IRIS-V1-{match.group(1)}-{match.group(2)}")
    return names


def spec_ids(pattern: re.Pattern[str]) -> set[str]:
    names: set[str] = set()
    for path in glob.glob(os.path.join(ROOT, "spec/iris-v1/*.md")):
        with open(path, encoding="utf-8") as handle:
            names.update(pattern.findall(handle.read()))
    return names


def main() -> int:
    corpus, declared = records(), spec_vector_ids()
    clauses = spec_ids(re.compile(r"IRIS-V1-[A-Z]+-C\d+[A-Z]?"))
    decisions = spec_ids(re.compile(r"(?<![A-Z0-9-])(D-\d{3})(?!\d)"))
    problems: list[str] = []
    seen: dict[str, str] = {}

    for path, record in corpus:
        name = record.get("id", "")
        relative = os.path.relpath(path, ROOT)

        if set(record) != TOP_LEVEL_FIELDS:
            problems.append(f"C018/C019 stable top-level fields: {name} ({relative})")

        # C008: the id shape, and one file per id.
        if not VECTOR_ID.match(name) and name not in SPEC_PUBLISHED_EXCEPTIONS:
            problems.append(f"C008 id shape: {name} ({relative})")
        if os.path.basename(path)[:-5] != name:
            problems.append(f"C008 filename does not match id: {relative}")
        if name in seen:
            problems.append(f"C008 duplicate id: {name} ({relative}, {seen[name]})")
        seen[name] = relative

        # C012 fixes the initial schema version.
        if record.get("schema_version") != SCHEMA_VERSION:
            problems.append(f"C012 schema_version: {name}")

        # C013 fixes the category vocabulary.
        if record.get("category") not in CATEGORIES:
            problems.append(f"C013 category {record.get('category')!r}: {name}")

        # C009 requires a human-readable name distinct from identity.
        if not record.get("name"):
            problems.append(f"C009 missing name: {name}")

        source = record.get("source")
        if not isinstance(source, dict):
            problems.append(f"C020 source is not an object: {name}")
            source = {}
        else:
            if set(source) != {"chapter", "artifact", "clauses", "decisions"}:
                problems.append(f"C020 source fields: {name}")
            if source.get("chapter") not in CHAPTERS:
                problems.append(f"C054 unknown chapter {source.get('chapter')!r}: {name}")
            for clause in source.get("clauses", []):
                if not isinstance(clause, str) or not CLAUSE_ID.fullmatch(clause) or clause not in clauses:
                    problems.append(f"C054 unresolved source clause {clause!r}: {name}")
            for decision in source.get("decisions", []):
                if not isinstance(decision, str) or not DECISION_ID.fullmatch(decision) or decision not in decisions:
                    problems.append(f"C054 unresolved D-ID {decision!r}: {name}")

        applicability = record.get("applicability")
        if not isinstance(applicability, dict):
            problems.append(f"C023 applicability is not an object: {name}")
        else:
            backend_keys = {"interpreter", "jit", "native"}
            if not backend_keys <= set(applicability):
                problems.append(f"C023 applicability backends: {name}")
            if any(applicability.get(backend) not in APPLICABILITY_VALUES for backend in backend_keys):
                problems.append(f"C023 applicability values: {name}")
            if any(applicability.get(backend) != "required" for backend in backend_keys):
                if not isinstance(applicability.get("reason"), str) or not applicability["reason"]:
                    problems.append(f"C024 applicability reason: {name}")

        corpus_record_rules.check(record, name, problems)

        tags = record.get("tags")
        if not isinstance(tags, list):
            problems.append(f"C034 tags are not an array: {name}")
        else:
            for tag in tags:
                if not isinstance(tag, str) or not TAG.fullmatch(tag):
                    problems.append(f"C034 tag spelling {tag!r}: {name}")
                elif tag.startswith("legacy:") and tag not in LEGACY_TAGS:
                    problems.append(f"C035 legacy tag {tag!r}: {name}")

    corpus_manifest_rules.check(ROOT, problems)
    corpus_freeze_rules.check(ROOT, problems)

    local = [name for name in seen if name not in declared]
    print(f"records: {len(corpus)}")
    print(f"  spec-declared ids: {len(corpus) - len(local)}")
    print(f"  locally authored:  {len(local)}")
    print(f"  C008 exceptions the spec itself publishes: {len(SPEC_PUBLISHED_EXCEPTIONS)}")
    if problems:
        print(f"\n{len(problems)} violations:")
        for problem in problems:
            print(f"  {problem}")
        return 1
    print("\nno violations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
