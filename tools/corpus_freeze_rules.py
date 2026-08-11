"""Freeze-gate rules from chapter 12 that a static audit can honestly enforce.

Most freeze clauses need a freeze PROCESS that does not exist: a release
pipeline, an owner review, a second backend. Those are recorded as open in
docs/milestone-2-status.md rather than faked here. The four below are different
because their obligation is a property of the artifacts already on disk.
"""

from __future__ import annotations

import glob
import json
import os
import re

EXAMPLE_DEFINITION = re.compile(r"^(IRIS-V1-[A-Z]+-EX\d+): *(.*)$", re.M)
INFORMATIVE = re.compile(r"\s*Informative\b", re.I)
LIBRARY_PUBLISHED = [f"IRIS-V1-LIBRARY-V{index:03d}" for index in range(1, 15)]
LEGACY_DISPOSITIONS = {
    "legacy:preserve", "legacy:intentional-divergence", "legacy:removed",
    "legacy:deferred", "legacy:not-applicable",
}


def _spec_text(root: str) -> dict[str, str]:
    texts = {}
    for path in glob.glob(os.path.join(root, "spec/iris-v1/*.md")):
        with open(path, encoding="utf-8") as handle:
            texts[os.path.basename(path)] = handle.read()
    return texts


def _corpus_ids(root: str) -> set[str]:
    names = set()
    pattern = os.path.join(root, "conformance/iris-v1/vectors/**/*.json")
    for path in glob.glob(pattern, recursive=True):
        with open(path, encoding="utf-8") as handle:
            names.add(json.load(handle)["id"])
    return names


def check_examples(root: str, problems: list[str]) -> None:
    """C004: an example that affects conformance needs a vector or a stated reason.

    All 48 published examples label themselves `Informative`, so none currently
    carries a conformance obligation. This check exists so that adding one that
    does NOT say so fails the gate instead of passing unnoticed.
    """
    cited: set[str] = set()
    pattern = os.path.join(root, "conformance/iris-v1/vectors/**/*.json")
    for path in glob.glob(pattern, recursive=True):
        with open(path, encoding="utf-8") as handle:
            cited.update(re.findall(r"IRIS-V1-[A-Z]+-EX\d+", handle.read()))

    matrix_path = os.path.join(root, "spec/iris-v1/traceability-matrix.md")
    with open(matrix_path, encoding="utf-8") as handle:
        matrix = handle.read()

    for name, text in _spec_text(root).items():
        for match in EXAMPLE_DEFINITION.finditer(text):
            example, description = match.group(1), match.group(2)
            if INFORMATIVE.match(description):
                continue
            if example in cited or example in matrix:
                continue
            problems.append(
                f"C004 example {example} in {name} is not marked Informative and has "
                "neither a vector nor a traceability row"
            )


def check_library_preservation(root: str, problems: list[str]) -> None:
    """C063: freeze treats LIBRARY complete only if V001-V014 are preserved."""
    corpus = _corpus_ids(root)
    missing = [name for name in LIBRARY_PUBLISHED if name not in corpus]
    if missing:
        problems.append(f"C063 LIBRARY published vectors are absent from the corpus: {missing}")


def check_migration_ledger(root: str, problems: list[str]) -> None:
    """C071: every disposition the ledger USES needs a corresponding vector.

    Scoped to the dispositions the 37 ledger rows actually carry, not to the
    whole C035 vocabulary: requiring a vector for a disposition no row uses
    would demand evidence for a claim the specification never makes.
    """
    migration = os.path.join(root, "spec/iris-v1/11-migration-divergence.md")
    with open(migration, encoding="utf-8") as handle:
        ledger = handle.read()
    used: set[str] = set()
    for _, rest in re.findall(r"^\| (IRIS-V1-MIG-\d+) \|(.*)$", ledger, re.M):
        for disposition in LEGACY_DISPOSITIONS:
            if f"`{disposition.removeprefix('legacy:')}`" in rest:
                used.add(disposition)
    if not used:
        problems.append("C071 migration ledger rows declare no disposition")
        return

    corpus_tags: set[str] = set()
    pattern = os.path.join(root, "conformance/iris-v1/vectors/**/*.json")
    for path in glob.glob(pattern, recursive=True):
        with open(path, encoding="utf-8") as handle:
            record = json.load(handle)
        corpus_tags.update(tag for tag in record.get("tags", []) if tag.startswith("legacy:"))

    unrepresented = sorted(used - corpus_tags)
    if unrepresented:
        problems.append(
            f"C071 migration ledger dispositions carry no vector: {unrepresented}"
        )


def check_spec_tree(root: str, problems: list[str]) -> None:
    """C051: corpus files must not appear under the specification tree.

    The specification directory holds Markdown artifacts only. A vector,
    fixture or manifest committed there would be corpus material living in the
    product artifact inventory, which C051 admits only under an approved plan.
    """
    for path in glob.glob(os.path.join(root, "spec/iris-v1/**/*"), recursive=True):
        if os.path.isfile(path) and not path.endswith(".md"):
            problems.append(f"C051 non-specification file under spec/iris-v1/: {os.path.relpath(path, root)}")


def check_anchors(root: str, problems: list[str]) -> None:
    """C058: every clause a record cites must resolve in the frozen chapters."""
    declared: set[str] = set()
    for text in _spec_text(root).values():
        declared.update(re.findall(r"IRIS-V1-[A-Z]+-C\d+[A-Z]?", text))

    pattern = os.path.join(root, "conformance/iris-v1/vectors/**/*.json")
    for path in glob.glob(pattern, recursive=True):
        with open(path, encoding="utf-8") as handle:
            record = json.load(handle)
        for clause in record["source"]["clauses"]:
            if clause not in declared:
                problems.append(f"C058 unresolved clause anchor {clause}: {record['id']}")


def check(root: str, problems: list[str]) -> None:
    check_examples(root, problems)
    check_library_preservation(root, problems)
    check_migration_ledger(root, problems)
    check_spec_tree(root, problems)
    check_anchors(root, problems)
