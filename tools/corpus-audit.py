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

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

VECTOR_ID = re.compile(r"^IRIS-V1-[A-Z]+-V\d+$")
SCHEMA_VERSION = "iris-v1-vector-schema-1"
CATEGORIES = {"positive", "negative", "diagnostic", "differential"}

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


def main() -> int:
    corpus, declared = records(), spec_vector_ids()
    problems: list[str] = []
    seen: dict[str, str] = {}

    for path, record in corpus:
        name = record.get("id", "")
        relative = os.path.relpath(path, ROOT)

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
