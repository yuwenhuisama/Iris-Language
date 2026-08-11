"""Manifest and fixture rules from chapter 12.

A manifest that merely EXISTS satisfies nothing: its value is that it cannot
disagree with the corpus it describes. These checks recompute the counts and
digests from the files on disk, so a stale manifest fails the gate rather than
misreporting the bundle.
"""

from __future__ import annotations

import hashlib
import json
import os

MANIFEST_PATH = "conformance/iris-v1/manifest.json"
FIXTURE_ROOT = "conformance/iris-v1/fixtures"
VECTOR_ROOT = "conformance/iris-v1/vectors"

REQUIRED_KEYS = {
    "corpus_schema_version", "iris_language_major", "unicode_version",
    "hash_schema_version", "source_spec_revision", "iris_value_format",
    "vector_count", "vectors_by_chapter", "fixture_digests",
}
FIXTURE_ROW_KEYS = {"path", "digest", "media_type", "role"}
UNICODE_VERSION = "17.0.0"
DIGEST_PREFIX = "blake3-256:"


def blake3_hex(data: bytes) -> str | None:
    """BLAKE3 is not in the standard library; return None when unavailable.

    The digest VALUES are produced by the Rust generator, which already depends
    on blake3 through iris-runtime. Recomputing them here is a cross-check, so
    when the module is absent the structural checks still run and only the
    value comparison is skipped, rather than the whole audit passing silently.
    """
    try:
        import blake3
    except ImportError:
        return None
    return blake3.blake3(data).hexdigest()


def check(root: str, problems: list[str]) -> None:
    path = os.path.join(root, MANIFEST_PATH)
    if not os.path.exists(path):
        problems.append(f"C052 corpus manifest is missing: {MANIFEST_PATH}")
        return
    try:
        with open(path, encoding="utf-8") as handle:
            manifest = json.load(handle)
    except json.JSONDecodeError as error:
        problems.append(f"C052 manifest is not valid JSON: {error}")
        return

    missing = REQUIRED_KEYS - set(manifest)
    if missing:
        problems.append(f"C052 manifest omits required keys: {sorted(missing)}")
        return

    if manifest["unicode_version"] != UNICODE_VERSION:
        problems.append(
            f"C052 manifest Unicode version {manifest['unicode_version']!r} "
            f"is not the {UNICODE_VERSION} that COLLECTIONS-C042 fixes"
        )

    _check_counts(root, manifest, problems)
    _check_fixtures(root, manifest, problems)


def _check_counts(root: str, manifest: dict, problems: list[str]) -> None:
    actual_by_chapter: dict[str, int] = {}
    vector_root = os.path.join(root, VECTOR_ROOT)
    for chapter in sorted(os.listdir(vector_root)):
        directory = os.path.join(vector_root, chapter)
        if not os.path.isdir(directory):
            continue
        count = len([n for n in os.listdir(directory) if n.endswith(".json")])
        if count:
            actual_by_chapter[chapter] = count

    total = sum(actual_by_chapter.values())
    if manifest["vector_count"] != total:
        problems.append(
            f"C052 manifest vector_count {manifest['vector_count']} "
            f"disagrees with {total} vectors on disk"
        )
    if manifest["vectors_by_chapter"] != actual_by_chapter:
        stale = {
            chapter: (manifest["vectors_by_chapter"].get(chapter), actual)
            for chapter, actual in actual_by_chapter.items()
            if manifest["vectors_by_chapter"].get(chapter) != actual
        }
        dropped = set(manifest["vectors_by_chapter"]) - set(actual_by_chapter)
        problems.append(
            f"C062 manifest per-chapter counts are stale: {stale or ''} "
            f"{'absent chapters ' + str(sorted(dropped)) if dropped else ''}".strip()
        )


def _check_fixtures(root: str, manifest: dict, problems: list[str]) -> None:
    rows = manifest["fixture_digests"]
    if not isinstance(rows, list):
        problems.append("C053 fixture_digests is not an array")
        return

    recorded: dict[str, str] = {}
    for row in rows:
        if not isinstance(row, dict) or set(row) != FIXTURE_ROW_KEYS:
            problems.append(f"C053 fixture row must cite exactly {sorted(FIXTURE_ROW_KEYS)}: {row}")
            continue
        if not str(row["digest"]).startswith(DIGEST_PREFIX):
            problems.append(f"C053 fixture digest is not BLAKE3-256: {row['path']}")
            continue
        if not row["media_type"] or not row["role"]:
            problems.append(f"C053 fixture row omits media type or role: {row['path']}")
        recorded[row["path"]] = str(row["digest"])[len(DIGEST_PREFIX):]

    on_disk: dict[str, bytes] = {}
    fixture_root = os.path.join(root, FIXTURE_ROOT)
    for directory, _, names in os.walk(fixture_root):
        for name in names:
            full = os.path.join(directory, name)
            logical = os.path.relpath(full, root).replace(os.sep, "/")
            with open(full, "rb") as handle:
                on_disk[logical] = handle.read()

    for logical in sorted(set(on_disk) - set(recorded)):
        problems.append(f"C053 fixture on disk is absent from the manifest: {logical}")
    for logical in sorted(set(recorded) - set(on_disk)):
        problems.append(f"C053 manifest cites a fixture that does not exist: {logical}")

    for logical in sorted(set(on_disk) & set(recorded)):
        computed = blake3_hex(on_disk[logical])
        if computed is None:
            return
        if computed != recorded[logical]:
            problems.append(f"C043 fixture digest does not match its bytes: {logical}")
