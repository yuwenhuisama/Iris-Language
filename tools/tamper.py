"""Tamper one committed vector and confirm the runner catches it.

A vector that cannot fail proves nothing, so every authored row is checked by
breaking its expectation and requiring the chapter run to report it. This
exists as a file because the earlier shell version wrapped mutations in
triple-quoted strings, and any mutation ending in a quote silently became a
syntax error that LOOKED like a passing tamper.

Usage:
    python3 tools/tamper.py CHAPTER VECTOR_ID 'python-expression-on-d' ...
"""

from __future__ import annotations

import copy
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def run_chapter(chapter: str) -> str:
    completed = subprocess.run(
        ["cargo", "run", "-q", "-p", "iris-conformance", "--", "--chapter", chapter],
        capture_output=True,
        text=True,
        cwd=ROOT,
        check=False,
    )
    return completed.stdout


def main() -> int:
    if len(sys.argv) < 4:
        print(__doc__)
        return 2
    chapter, vector_id, mutations = sys.argv[1], sys.argv[2], sys.argv[3:]
    path = os.path.join(ROOT, f"conformance/iris-v1/vectors/{chapter}/{vector_id}.json")
    with open(path, encoding="utf-8") as handle:
        original = json.load(handle)

    failures = 0
    for mutation in mutations:
        record = copy.deepcopy(original)
        namespace = {"d": record}
        try:
            exec(mutation, namespace)  # noqa: S102 - a deliberate local mutation
        except SyntaxError as error:
            print(f"  {mutation[:52]:<54} -> MUTATION IS INVALID: {error}")
            failures += 1
            continue
        with open(path, "w", encoding="utf-8") as handle:
            json.dump(record, handle, ensure_ascii=False)
        caught = f"failed: {vector_id}" in run_chapter(chapter)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(json.dumps(original, separators=(",", ":"), ensure_ascii=False) + "\n")
        print(f"  {mutation[:52]:<54} -> {'caught' if caught else 'NOT CAUGHT'}")
        failures += 0 if caught else 1

    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
