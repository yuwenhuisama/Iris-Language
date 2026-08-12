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


def check_chapter_coverage(root: str, corpus: list[dict], problems: list[str]) -> None:
    """C048: a chapter with normative clauses needs vector class coverage.

    The required-class table lives in C046 as Markdown, one row per chapter, so
    it is read from the specification rather than restated here: a table edited
    without updating the corpus must fail, which a hardcoded copy could not
    detect.

    Two chapters carry normative clauses without a table row, CONFORMANCE and
    TRACE, because the table predates them owning executable obligations. C048
    excepts a clause the traceability matrix marks as documentation structure,
    and both chapters are cited there, so their coverage is required from the
    corpus rather than from a row they do not have.
    """
    text = _spec_text(root).get("12-conformance.md", "")
    try:
        table = text[
            text.index("IRIS-V1-CONFORMANCE-C046:") : text.index(
                "IRIS-V1-CONFORMANCE-C047:"
            )
        ]
    except ValueError:
        problems.append("C048 the C046 required-class table is not locatable")
        return
    listed = {name for name, _ in re.findall(r"^\| `([A-Z]+)` \| (.+?) \|$", table, re.M)}
    if not listed:
        problems.append("C048 the C046 required-class table declares no chapter")
        return

    with_clauses: set[str] = set()
    for name, body in _spec_text(root).items():
        for match in CLAUSE_BLOCK.finditer(body):
            with_clauses.add(match.group(1).split("-")[2])

    covered = {record["source"]["chapter"] for record in corpus}
    for chapter in sorted(with_clauses):
        if chapter not in covered:
            problems.append(
                f"C048 chapter {chapter} has normative clauses and no vector coverage"
            )
    for chapter in sorted(listed - with_clauses):
        problems.append(
            f"C048 the required-class table lists {chapter}, which declares no clause"
        )


FORBIDDEN_SOURCE = re.compile(
    r"legacy/|Legacy Iris|\.pdf|lex\.yy|y\.tab|Pointer Extension|File Extension",
    re.I,
)
DISPOSES = re.compile(r"MUST NOT|evidence only", re.I)


def check_forbidden_authority(root: str, corpus: list[dict], problems: list[str]) -> None:
    """C064: a forbidden source may be referenced only under a clause adopting it.

    The clause forbids Legacy Iris scripts, generated parser files, old PDF text
    and native extension examples from acting as normative authority, EXCEPT
    where a vector cites a frozen clause that explicitly disposes of the
    behaviour. Both halves are checked here: which vectors touch such a source,
    and whether each cites a clause that actually names one.

    The permitted clause set is DERIVED from the specification rather than
    listed, so a clause added or reworded later is picked up instead of silently
    falling outside a hardcoded allowance.

    C064's remaining half is NOT decidable here. Whether an expectation rests on
    a "current implementation quirk" is a judgement about why a value was
    written, which no property of the file records; that half is reported by
    `docs/milestone-2-status.md` as requiring owner review.
    """
    disposing: set[str] = set()
    for text in _spec_text(root).values():
        for match in CLAUSE_BLOCK.finditer(text):
            body = match.group(2)
            if FORBIDDEN_SOURCE.search(body) and DISPOSES.search(body):
                disposing.add(match.group(1))
    if not disposing:
        problems.append("C064 no frozen clause disposes of a forbidden source")
        return

    for record in corpus:
        blob = json.dumps(record)
        if not FORBIDDEN_SOURCE.search(blob):
            continue
        if not set(record["source"]["clauses"]) & disposing:
            problems.append(
                f"C064 references a forbidden source without citing a clause that "
                f"disposes of it: {record['id']}"
            )


def check_expectation_provenance(root: str, corpus: list[dict], problems: list[str]) -> None:
    """C064: an expectation must not invent an identifier the runtime chose.

    A test picks its own symbols freely, and those appear in the vector's OWN
    source: `break :stopped` justifies expecting `:stopped`. A symbol appearing
    in NEITHER the specification NOR the vector's source came from the runtime,
    which is a current implementation quirk recorded as though it were the rule.

    Found exactly one: ASYNC-V909 asserted the diagnostic event label
    `UnobservedFailure` and a four-slot tuple shape, while its clause requires
    only that the event carry an ExceptionContext rather than the bare raised
    value. The label and shape are implementation-chosen and appear nowhere in
    the specification, so a rename would have failed the row with no clause
    violated.

    Scoped to locally authored rows. A spec-declared row transcribes a published
    table row, so its values carry the specification's authority by definition.
    """
    spec = " ".join(_spec_text(root).values())
    declared: set[str] = set()
    for text in _spec_text(root).values():
        declared.update(VECTOR_ID.findall(text))
    for record in corpus:
        if record["id"] in declared:
            continue
        if "bucket:executable" not in record.get("tags", []):
            continue
        source = json.dumps(record.get("input", {}))
        values = set(
            re.findall(r'"(?:symbol|string)":\s*"([^"]+)"', json.dumps(record.get("expect", {})))
        )
        orphans = sorted(
            value for value in values if value not in spec and value not in source
        )
        if orphans:
            problems.append(
                f"C064 expectation names an identifier from neither the specification "
                f"nor its own source: {record['id']} {orphans}"
            )


DIFFERENTIAL_CLAUSES = {"IRIS-V1-CONFORMANCE-C041"}


def check_differential_claims(corpus: list[dict], problems: list[str]) -> None:
    """A clause requiring backend agreement may only be cited by a differential row.

    `C041` compares emitted float bits ACROSS interpreter, JIT and native. Only
    the interpreter exists, so the clause is deferred to the virtual machine.
    The hazard is that deferral looks identical to completion once someone cites
    the clause from an ordinary single-backend row: coverage would report it
    satisfied while nothing compared anything.

    A row claiming such a clause must therefore be tagged `bucket:differential`,
    which the runner reports separately and never counts as passing.
    """
    for record in corpus:
        claimed = set(record["source"]["clauses"]) & DIFFERENTIAL_CLAUSES
        if not claimed:
            continue
        if "bucket:differential" not in record.get("tags", []):
            problems.append(
                f"C041 backend-agreement clause {sorted(claimed)} cited by a row that "
                f"is not differential: {record['id']}"
            )


def check(root: str, problems: list[str]) -> None:
    corpus = _corpus(root)
    check_published_ids(root, corpus, problems)
    check_class_balance(corpus, problems)
    check_deferred(root, corpus, problems)
    check_backend_reasons(corpus, problems)
    check_clause_coverage(root, corpus, problems)
    check_chapter_coverage(root, corpus, problems)
    check_forbidden_authority(root, corpus, problems)
    check_expectation_provenance(root, corpus, problems)
    check_differential_claims(corpus, problems)
