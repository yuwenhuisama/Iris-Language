"""Freeze-gate clauses that a static audit can decide today.

Chapter 12's freeze rules read as if they all need a release pipeline, but five
of them are decidable from the corpus and the specification as they stand. They
are separated from the ones that genuinely cannot be decided:

- `C041` needs a SECOND BACKEND. Comparing emitted float bits across
  interpreter, JIT and native requires more than one of them to exist.
- `C048` needs the per-chapter required-vector-class table `C046` refers to,
  which is stated in chapter 12 as prose and is not transcribed.
- `C064` needs a HUMAN. Deciding whether a behaviour's normative authority is a
  Legacy Iris script, old PDF text or an implementation quirk is a judgement
  about provenance, not a property of any file.
"""

from __future__ import annotations

import collections
import glob
import json
import os
import re

VECTOR_ID = re.compile(r"IRIS-V1-[A-Z]+-V\d+[A-Z]?")
CLAUSE_BLOCK = re.compile(
    r"^(IRIS-V1-[A-Z]+-C\d+[A-Z]?):(.*?)(?=\n\nIRIS-V1-|\n\n#|\Z)", re.M | re.S
)
BACKENDS = ("interpreter", "jit", "native")

# The same exclusions `tools/clause-coverage.py` applies: a clause that only
# scopes a chapter, defers to another, or introduces a table states no
# obligation a vector can observe. Counting them made 26 chapter-opening
# clauses look newly uncovered.
SCOPE = re.compile(r"^this chapter (defines|covers|specifies)", re.I)
EXCLUSION = re.compile(r"must not define|belongs to later chapters", re.I)
TABLE = re.compile(r"the following .*(table|inventory|list) is normative", re.I)


def _spec_text(root: str) -> dict[str, str]:
    found = {}
    for path in glob.glob(os.path.join(root, "spec/iris-v1/*.md")):
        with open(path, encoding="utf-8") as handle:
            found[os.path.basename(path)] = handle.read()
    return found


def _corpus(root: str) -> list[dict]:
    pattern = os.path.join(root, "conformance/iris-v1/vectors/**/*.json")
    records = []
    for path in sorted(glob.glob(pattern, recursive=True)):
        with open(path, encoding="utf-8") as handle:
            records.append(json.load(handle))
    return records


def check_published_ids(root: str, corpus: list[dict], problems: list[str]) -> None:
    """C046: every vector id any chapter publishes must survive in the corpus."""
    declared: set[str] = set()
    for text in _spec_text(root).values():
        declared.update(VECTOR_ID.findall(text))
    present = {record["id"] for record in corpus}
    for name in sorted(declared - present):
        problems.append(f"C046 published vector id absent from the corpus: {name}")


def check_class_balance(corpus: list[dict], problems: list[str]) -> None:
    """C049: a required vector class needs a positive AND a negative or diagnostic.

    Found three chapters stating only refusals. A chapter that never records a
    success proves its rules reject, never that anything is accepted.
    """
    by_chapter: dict[str, collections.Counter] = collections.defaultdict(
        collections.Counter
    )
    for record in corpus:
        by_chapter[record["source"]["chapter"]][record["category"]] += 1
    for chapter, counts in sorted(by_chapter.items()):
        refusing = counts["negative"] + counts["diagnostic"]
        if not counts["positive"]:
            problems.append(f"C049 chapter {chapter} states no positive vector")
        if not refusing:
            problems.append(
                f"C049 chapter {chapter} states no negative or diagnostic vector"
            )


def check_deferred(root: str, corpus: list[dict], problems: list[str]) -> None:
    """C059: a deferred item must not be tested as normative behaviour."""
    deferred: set[str] = set()
    for text in _spec_text(root).values():
        for match in CLAUSE_BLOCK.finditer(text):
            if "DEFERRED V1" in match.group(2):
                deferred.add(match.group(1))
    if not deferred:
        problems.append("C059 no clause carries a DEFERRED V1 marker to check")
        return
    for record in corpus:
        cited = set(record["source"]["clauses"]) & deferred
        if not cited:
            continue
        if record["category"] == "positive" and "bucket:executable" in record["tags"]:
            problems.append(
                f"C059 deferred clause {sorted(cited)} tested as normative behaviour: "
                f"{record['id']}"
            )


def check_backend_reasons(corpus: list[dict], problems: list[str]) -> None:
    """C060: a backend may be excluded only with a stated reason.

    Backend-dependent behaviour is non-conforming unless a source clause marks
    the backend not applicable, so an unexplained exclusion hides exactly the
    divergence the clause forbids.
    """
    for record in corpus:
        applicability = record.get("applicability", {})
        excluded = [
            backend
            for backend in BACKENDS
            if applicability.get(backend) == "not_applicable"
        ]
        if excluded and not applicability.get("reason", "").strip():
            problems.append(
                f"C060 backend {excluded} excluded without a reason: {record['id']}"
            )


def check_clause_coverage(root: str, corpus: list[dict], problems: list[str]) -> None:
    """C057: freeze rejects an uncovered normative clause.

    Reported against a recorded allowance rather than zero, because eight
    chapter-12 clauses are open for reasons documented in the milestone status.
    A NEW uncovered clause fails here; the known set does not.
    """
    cited: set[str] = set()
    for record in corpus:
        cited.update(record["source"]["clauses"])
    obligations: set[str] = set()
    for text in _spec_text(root).values():
        for match in CLAUSE_BLOCK.finditer(text):
            body = match.group(2).strip()
            if not re.search(r"\bMUST\b|\bSHALL\b", body):
                continue
            if SCOPE.match(body) or EXCLUSION.search(body) or TABLE.search(body):
                continue
            obligations.add(match.group(1))
    uncovered = obligations - cited
    unexpected = sorted(uncovered - KNOWN_OPEN)
    if unexpected:
        problems.append(f"C057 newly uncovered normative clauses: {unexpected}")


KNOWN_OPEN = {
    "IRIS-V1-CONFORMANCE-C041",
    "IRIS-V1-CONFORMANCE-C046",
    "IRIS-V1-CONFORMANCE-C048",
    "IRIS-V1-CONFORMANCE-C049",
    "IRIS-V1-CONFORMANCE-C057",
    "IRIS-V1-CONFORMANCE-C059",
    "IRIS-V1-CONFORMANCE-C060",
    "IRIS-V1-CONFORMANCE-C064",
}


def check(root: str, problems: list[str]) -> None:
    corpus = _corpus(root)
    check_published_ids(root, corpus, problems)
    check_class_balance(corpus, problems)
    check_deferred(root, corpus, problems)
    check_backend_reasons(corpus, problems)
    check_clause_coverage(root, corpus, problems)
