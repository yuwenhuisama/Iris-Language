#!/usr/bin/env python3
"""Audit the frozen specification against the TRACE rules that govern it.

`IRIS-V1-TRACE-C003` is deliberately NOT checked here. A check comparing
"states a requirement" against "uses an RFC 2119 term" is unsatisfiable,
because the first set is a subset of the second. The reachable version, looking
for a lowercase requirement word, finds three frozen-spec clauses and two
informative notes where lowercase is correct; those are recorded in
docs/spec-defects-v1.md rather than reported on every run.

The TRACE and MIGRATION chapters constrain how the specification is WRITTEN,
not how an implementation behaves, so their obligations cannot be observed by a
vector that runs an Iris program. They are checked here against the spec text.

Note the id pattern: clause ids may carry a trailing letter, as `C053A` and
`C053B` do. Matching `C\\d+` alone truncates those and reports an
ID-carrying paragraph as unlabelled. The same truncation once made the
spec-published vector id `V288A` look locally authored, so both patterns here
capture the optional letter deliberately.

Usage:
    python3 tools/spec-audit.py          # report violations, exit 1 if any
"""

from __future__ import annotations

import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

CLAUSE_ID = re.compile(r"^(IRIS-V1-[A-Z]+-[CN]\d+[A-Z]?):")
# An executable example carries an `EX###` id rather than a clause id, and
# `IRIS-V1-TRACE-C005` requires one for every example, so the informative check
# has to see past that prefix too.
ANY_ID = re.compile(r"^(IRIS-V1-[A-Z]+-(?:[CN]\d+[A-Z]?|EX\d+)):")
NORMATIVE = re.compile(r"\bMUST\b|\bSHALL\b")

# `IRIS-V1-TRACE-C004` fixes the labels informative text must carry.
INFORMATIVE_LABELS = (
    "Informative note:",
    "Informative example:",
    "Implementation note:",
    "Historical note:",
)

# `IRIS-V1-TRACE-C002` fixes the product inventory under spec/iris-v1.
PRODUCT_ARTIFACTS = 14

# `IRIS-V1-MIGRATION-C001` fixes the disposition vocabulary for a ledger row.
DISPOSITIONS = ("preserve", "intentional-divergence", "removed", "deferred")


def text(path: str) -> str:
    with open(os.path.join(ROOT, path), encoding="utf-8") as handle:
        return handle.read()


def chapters() -> list[str]:
    paths = glob.glob(os.path.join(ROOT, "spec/iris-v1/*.md"))
    return sorted(p for p in paths if re.search(r"/\d\d-|README|traceability", p))


def paragraphs(path: str) -> list[str]:
    with open(path, encoding="utf-8") as handle:
        return [block.strip() for block in handle.read().split("\n\n")]


def main() -> int:
    problems: list[str] = []
    normative = labelled = 0

    for path in chapters():
        relative = os.path.relpath(path, ROOT)
        for block in paragraphs(path):
            if block.startswith(("|", "- ", "```", ">")):
                continue
            # An informative paragraph carries its clause ID FIRST, as in
            # `IRIS-V1-IDENTITY-N001: Informative note: ...`, so the label is
            # matched after that prefix. Testing the raw block start matched
            # nothing at all and made this check silently vacuous.
            identified = ANY_ID.match(block)
            body = block[identified.end():].lstrip() if identified else block
            if any(body.startswith(label) for label in INFORMATIVE_LABELS):
                labelled += 1
                # C003 reserves RFC 2119 terms for NORMATIVE clauses, so an
                # informative paragraph stating a requirement is mislabelled.
                if NORMATIVE.search(block):
                    problems.append(f"C004 informative text states a requirement: {relative}")
                continue
            if not NORMATIVE.search(block):
                continue
            normative += 1
            # C005: a paragraph carrying normative requirements must have a
            # stable clause ID.
            if not CLAUSE_ID.match(block):
                problems.append(f"C005 normative paragraph without an id: {relative}: {block[:70]}")

    # C002: the product inventory is a fixed set of artifacts.
    artifacts = glob.glob(os.path.join(ROOT, "spec/iris-v1/*.md"))
    if len(artifacts) != PRODUCT_ARTIFACTS:
        problems.append(f"C002 product artifacts: expected {PRODUCT_ARTIFACTS}, found {len(artifacts)}")

    # `IRIS-V1-MIGRATION-C001` requires every ledger row to record legacy
    # evidence, a v1 replacement, rationale, a migration example, and EXACTLY
    # one disposition tag. A row carrying two tags, or none, has no disposition.
    ledger = os.path.join(ROOT, "spec/iris-v1/11-migration-divergence.md")
    with open(ledger, encoding="utf-8") as handle:
        rows = [line for line in handle.read().splitlines() if line.startswith("| IRIS-V1-MIG-")]
    for row in rows:
        cells = [cell.strip() for cell in row.split("|")[1:-1]]
        name = cells[0] if cells else "?"
        if len(cells) < 5 or any(not cell for cell in cells):
            problems.append(f"C001 ledger row incomplete: {name}")
        tags = [tag for tag in DISPOSITIONS if f"`{tag}`" in row]
        if len(tags) != 1:
            problems.append(f"C001 ledger row disposition tags {tags}: {name}")

    # C008: clause IDs must begin their paragraph so tooling can locate them
    # without parsing prose. Headings remain ordinary GitHub Markdown headings.
    for path in chapters():
        relative = os.path.relpath(path, ROOT)
        for block in paragraphs(path):
            if not block.startswith(("#", "|")) and NORMATIVE.search(block) and not CLAUSE_ID.match(block):
                problems.append(f"C008 normative clause is not paragraph-addressable: {relative}")

    grammar = text("spec/iris-v1/02-lexical-grammar.md")
    # C010: the D-509 errata makes `typeof` the one widened keyword and C013
    # states the authoritative count.
    keyword_clause = re.search(r"IRIS-V1-GRAMMAR-C013:(.*?)(?=\n\nIRIS-V1-)", grammar, re.S)
    if keyword_clause is None or "exactly 49 lowercase words" not in keyword_clause.group(1) or "`typeof`" not in keyword_clause.group(1):
        problems.append("C010 reserved-keyword errata inventory is missing its 49-word typeof revision")

    readme = text("spec/iris-v1/README.md")
    # C016: the terminology table carries the fixed canonical vocabulary.
    for term in ("Iris v1", "Legacy Iris", "Contract", "MutableString", "ReflectionPolicy", "DEFERRED V1"):
        if f"| {term} |" not in readme:
            problems.append(f"C016 canonical terminology entry missing: {term}")

    # C019-C022: the frozen revision procedure must keep its four distinct
    # safeguards, rather than silently reducing errata to a generic note.
    revision_requirements = {
        "C019": ("recorded owner approval",),
        "C020": ("MUST increment the specification revision", "Simplified Chinese translation"),
        "C021": ("MUST NOT reinterpret a decided semantic", "new total explicitly"),
        "C022": ("committed conformance corpus", "MUST NOT adjust, retag, or delete a vector"),
    }
    for clause, required in revision_requirements.items():
        match = re.search(rf"^IRIS-V1-TRACE-{clause}:(.*?)(?=\n\n(?:^IRIS-V1-|##)|\Z)", readme, re.M | re.S)
        if match is None or any(item not in match.group(1) for item in required):
            problems.append(f"{clause} revision-procedure safeguard missing")

    migration = text("spec/iris-v1/11-migration-divergence.md")
    # C012 explicitly excludes D-509/review-only extras from C011's exact
    # Notepad++ inventory. Parse its rows rather than merely checking prose.
    extras = re.search(r"IRIS-V1-MIGRATION-C012:.*?\n\n(\| Word .*?)(?=\n\n##)", migration, re.S)
    expected_extras = {"and", "defer", "implements", "involve", "native", "not", "or", "repeat", "satisfies", "undef"}
    found_extras = set(re.findall(r"(?m)^\| `([a-z]+)` \|", extras.group(1))) if extras else set()
    if found_extras != expected_extras:
        problems.append(f"C012 non-highlighter extras mismatch: expected {sorted(expected_extras)}, found {sorted(found_extras)}")

    print(f"product artifacts: {len(artifacts)}")
    print(f"migration ledger rows: {len(rows)}")
    print(f"normative paragraphs: {normative}")
    print(f"labelled informative paragraphs: {labelled}")
    if problems:
        print(f"\n{len(problems)} violations:")
        for problem in problems:
            print(f"  {problem}")
        return 1
    print("\nno violations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
