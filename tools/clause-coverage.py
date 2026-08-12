#!/usr/bin/env python3
"""Report which frozen specification clauses any conformance vector cites.

The vector tables are complete, but a vector row and a normative clause are
different units: one row can rest on several clauses, and many clauses are
named by no row at all. Behaviour that is correct but uncited is behaviour a
refactor can break with nothing failing, which is how this project's
iterator-close and extension-digest defects survived.

Usage:
    python3 tools/clause-coverage.py                 # summary table
    python3 tools/clause-coverage.py CHAPTER         # uncited clauses, with text
    python3 tools/clause-coverage.py CHAPTER --ids   # uncited clause ids only
"""

from __future__ import annotations

import collections
import glob
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# A clause DEFINITION starts a line; a mention inside prose or a table does not.
# The trailing letter is part of the id. Capturing `C\d+` alone made
# `GRAMMAR-C048A`, `C053A` and `C053B` invisible to this report: they were never
# counted as declared, so they could never be counted as open. This is the same
# truncation that once made `V288A` look locally authored.
CLAUSE_DEFINITION = re.compile(
    r"^(IRIS-V1-([A-Z]+)-C\d+[A-Z]?): (.*?)(?=\n\nIRIS-V1-|\n\n#|\Z)", re.M | re.S
)

# Clauses whose text only scopes a chapter, defers to another, or introduces a
# table are not obligations a vector can observe, so they are reported apart
# from the ones that are.
SCOPE = re.compile(r"^this chapter (defines|covers|specifies)", re.I)
EXCLUSION = re.compile(r"must not define|belongs to later chapters", re.I)
TABLE = re.compile(r"the following .*(table|inventory|list) is normative", re.I)
OBLIGATION = re.compile(r"\bMUST\b|\bSHALL\b")


def declared() -> dict[str, tuple[str, str]]:
    found: dict[str, tuple[str, str]] = {}
    for path in glob.glob(os.path.join(ROOT, "spec/iris-v1/*.md")):
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        for match in CLAUSE_DEFINITION.finditer(text):
            found[match.group(1)] = (match.group(2), match.group(3).strip())
    return found


def cited() -> set[str]:
    names: set[str] = set()
    pattern = os.path.join(ROOT, "conformance/iris-v1/vectors/**/*.json")
    for path in glob.glob(pattern, recursive=True):
        with open(path, encoding="utf-8") as handle:
            record = json.load(handle)
        names.update(record["source"].get("clauses", []))
    return names


def kind(text: str) -> str:
    if SCOPE.match(text):
        return "scope"
    if EXCLUSION.search(text):
        return "scope"
    if TABLE.search(text):
        return "table"
    return "obligation" if OBLIGATION.search(text) else "prose"


def clause_order(row: tuple) -> tuple[int, str]:
    """Sorts `C053` before `C053A` before `C053B` rather than failing on the letter."""
    tail = row[0].rsplit("C", 1)[1]
    digits = tail.rstrip("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
    return (int(digits), tail[len(digits) :])


def main() -> int:
    clauses, covered = declared(), cited()
    chapter = sys.argv[1] if len(sys.argv) > 1 and not sys.argv[1].startswith("-") else None

    if chapter:
        rows = [
            (name, body)
            for name, (owner, body) in sorted(clauses.items())
            if owner == chapter and name not in covered and kind(body) == "obligation"
        ]
        rows.sort(key=clause_order)
        if "--ids" in sys.argv:
            print(" ".join(name for name, _ in rows))
            return 0
        for name, body in rows:
            print(f"-- {name}\n   {body[:400]}\n")
        print(f"{len(rows)} uncited obligations in {chapter}")
        return 0

    # Reported per KIND rather than as one blended percentage. A single ratio
    # over every clause divided 728 by 947 and read as 76.9%, which understated
    # the work by counting chapter-scope headers, table lead-ins and
    # cross-referencing prose as though they were unmet obligations. Only a
    # clause carrying MUST or SHALL states something a vector can observe.
    counts: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    for name, (owner, body) in clauses.items():
        obligation = kind(body) == "obligation"
        counts[owner]["total"] += 1
        counts[owner]["duty"] += 1 if obligation else 0
        if name in covered:
            counts[owner]["cited"] += 1
            counts[owner]["duty_cited"] += 1 if obligation else 0
        elif obligation:
            counts[owner]["open"] += 1

    print(f"{'CHAPTER':<13}{'DUTIES':>7}{'MET':>6}{'OPEN':>6}{'OTHER':>7}{'CLAUSES':>9}")
    totals: collections.Counter = collections.Counter()
    for owner in sorted(counts):
        row = counts[owner]
        totals.update(row)
        other = row["total"] - row["duty"]
        print(
            f"{owner:<13}{row['duty']:>7}{row['duty_cited']:>6}"
            f"{row['open']:>6}{other:>7}{row['total']:>9}"
        )
    other = totals["total"] - totals["duty"]
    print(
        f"{'TOTAL':<13}{totals['duty']:>7}{totals['duty_cited']:>6}"
        f"{totals['open']:>6}{other:>7}{totals['total']:>9}"
    )
    share = totals["duty_cited"] / totals["duty"] * 100 if totals["duty"] else 0.0
    print(f"\nobligation coverage: {totals['duty_cited']}/{totals['duty']} ({share:.1f}%)")
    print("DUTIES carry MUST or SHALL. OTHER are chapter scope, table lead-ins")
    print("and cross-referencing prose, which state no separately observable rule.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
