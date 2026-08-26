#!/usr/bin/env python3
"""Emit the RUNNABLE source corpus the VM coverage probe measures against.

The probe reads a TSV of `id<TAB>base64(source)` rather than the corpus
directly, because `iris-vm` does not depend on the conformance crate and a
test that walked the vector tree would duplicate its schema handling.

That file used to be produced by hand into /tmp, which made the headline
coverage number UNREPRODUCIBLE: once the file aged out, neither the count nor
the denominator behind it could be recovered, and a reported gap could not be
re-measured. Writing it from the frozen corpus with the selection rules stated
here is what makes the measurement auditable.

The denominator excludes three kinds of vector, for reasons that are not
interchangeable:

  - `input.malformed` vectors are SUPPOSED to be rejected, so counting them as
    coverage gaps measures the backend against programs it is right to refuse.
  - `bucket:documentation` vectors carry prose rather than a program.
  - `bucket:record-validation` vectors carry a conformance RECORD as their
    source text - the `source_text` is a vector's own JSON, not Iris - so the
    compiler is right to reject it and a rejection is not a gap.

Dropping the last rule is what a reconstruction of this file first did, and it
inflated `rejected source` from 25 to 44 while moving the denominator to 755:
19 JSON records counted as programs the backend had failed to compile.

Usage:
    python3 tools/vm-corpus.py [output.tsv]
"""

from __future__ import annotations

import base64
import glob
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def runnable_vectors() -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    pattern = os.path.join(ROOT, "conformance/iris-v1/vectors/*/*.json")
    for path in sorted(glob.glob(pattern)):
        with open(path, encoding="utf-8") as handle:
            vector = json.load(handle)
        source = vector.get("input", {})
        if source.get("kind") != "source":
            continue
        text = source.get("source_text")
        if text is None:
            continue
        if source.get("malformed"):
            continue
        tags = vector.get("tags", [])
        if "bucket:documentation" in tags or "bucket:record-validation" in tags:
            continue
        rows.append((vector["id"], base64.b64encode(text.encode()).decode()))
    return rows


def main() -> int:
    destination = sys.argv[1] if len(sys.argv) > 1 else "/tmp/srcs.tsv"
    rows = runnable_vectors()
    with open(destination, "w", encoding="utf-8") as handle:
        for name, encoded in rows:
            handle.write(f"{name}\t{encoded}\n")
    print(f"wrote {len(rows)} runnable source vectors to {destination}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
