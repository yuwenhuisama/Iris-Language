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
CLAUSE_DEFINITION = re.compile(
    r"^(IRIS-V1-([A-Z]+)-C\d+): (.*?)(?=\n\nIRIS-V1-|\n\n#|\Z)", re.M | re.S
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


def main() -> int:
    clauses, covered = declared(), cited()
    chapter = sys.argv[1] if len(sys.argv) > 1 and not sys.argv[1].startswith("-") else None

    if chapter:
        rows = [
            (name, body)
            for name, (owner, body) in sorted(clauses.items())
            if owner == chapter and name not in covered and kind(body) == "obligation"
        ]
        rows.sort(key=lambda row: int(row[0].rsplit("C", 1)[1]))
        if "--ids" in sys.argv:
            print(" ".join(name for name, _ in rows))
            return 0
        for name, body in rows:
            print(f"-- {name}\n   {body[:400]}\n")
        print(f"{len(rows)} uncited obligations in {chapter}")
        return 0

    counts: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    for name, (owner, body) in clauses.items():
        counts[owner]["total"] += 1
        if name in covered:
            counts[owner]["cited"] += 1
        elif kind(body) == "obligation":
            counts[owner]["open"] += 1

    print(f"{'CHAPTER':<13}{'CLAUSES':>8}{'CITED':>7}{'OPEN':>6}")
    totals: collections.Counter = collections.Counter()
    for owner in sorted(counts):
        row = counts[owner]
        totals.update(row)
        print(f"{owner:<13}{row['total']:>8}{row['cited']:>7}{row['open']:>6}")
    print(f"{'TOTAL':<13}{totals['total']:>8}{totals['cited']:>7}{totals['open']:>6}")
    share = totals["cited"] / totals["total"] * 100 if totals["total"] else 0.0
    print(f"\nclause citation coverage: {share:.1f}%")
    print("OPEN counts uncited clauses carrying MUST or SHALL.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
