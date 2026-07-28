from __future__ import annotations

import re
import shutil
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VECTOR_ID = re.compile(r"IRIS-V1-[A-Z]+-V\d{3}")
CLAUSE_ID = re.compile(r"IRIS-V1-[A-Z]+-C\d+[A-Z]?")
DECISION_ID = re.compile(r"D-\d{3}")
MATRIX_ROW = re.compile(r"^\|\s*`(D-\d{3})`\s*\|")
VECTOR_ROW = re.compile(r"^\|\s*`(IRIS-V1-[A-Z]+-V\d{3})`\s*\|")
VECTOR_HEADER = "| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |"
DELETED_BROAD_VECTORS = {
    "IRIS-V1-GRAMMAR-V008", "IRIS-V1-GRAMMAR-V009", "IRIS-V1-GRAMMAR-V012",
    "IRIS-V1-GRAMMAR-V014", "IRIS-V1-COLLECTIONS-V040", "IRIS-V1-COLLECTIONS-V041",
    "IRIS-V1-COLLECTIONS-V043",
}
METADATA = {
    "D-000": ("normative", "specification artifact location and inventory metadata"),
    "D-004": ("superseded wording consolidated", "superseded by exact Float32 and Float64 type clauses"),
    "D-009": ("superseded wording consolidated", "consolidated by directional heterogeneous numeric method clauses"),
    "D-472": ("DEFERRED V1", "explicit DEFERRED V1 cancellation has no executable v1 behavior"),
}
NON_SEMANTIC = {
    "IRIS-V1-RUNTIME-C156",
    "IRIS-V1-CONTROL-C070",
    "IRIS-V1-COLLECTIONS-C092",
    "IRIS-V1-META-C113",
}
PLACEHOLDERS = (
    "Concrete D-",
    "SET [",
    "Add ",
    "ADD ",
    "directly covers",
    "audit confirmation",
    "The specified value, Type, error, diagnostic, status, or side effect",
    "The named direct scenario produces",
)
CATEGORIES = {"positive", "negative", "diagnostic", "differential"}
VALID_STATUSES = {
    "normative",
    "superseded wording consolidated",
    "informative implementation freedom",
    "DEFERRED V1",
}
CHAPTER_FILES = {
    "IDENTITY": "01-language-identity.md", "GRAMMAR": "02-lexical-grammar.md",
    "RUNTIME": "03-runtime-object-model.md", "CONTROL": "04-bindings-callables-control-flow.md",
    "TYPES": "05-types-contracts-generics.md", "COLLECTIONS": "06-collections-text-regex.md",
    "ASYNC": "07-async-resources-diagnostics.md", "META": "08-modules-metaprogramming.md",
    "FFI": "09-native-host-ffi.md", "LIBRARY": "10-serialization-standard-library.md",
    "MIGRATION": "11-migration-divergence.md", "CONFORMANCE": "12-conformance.md",
    "TRACE": "README.md",
}


@dataclass(frozen=True, slots=True)
class VectorDefinition:
    identifier: str
    decisions: frozenset[str]
    path: Path
    line: int


class ValidationFailure(Exception):
    pass


def fail(message: str) -> None:
    raise ValidationFailure(message)


def cells(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split(" | ")]


def matrix_cells(line: str) -> list[str]:
    content = line.strip().strip("|")
    decision, remainder = content.split(" | ", 1)
    heading, anchors_and_tail = remainder.split(" | [", 1)
    anchors, coverage, migration, status = ("[" + anchors_and_tail).rsplit(" | ", 3)
    return [decision.strip(), heading.strip(), anchors.strip(), coverage.strip(), migration.strip(), status.strip()]


def require_concrete(cell: str, label: str, path: Path, line: int) -> None:
    if not cell or any(pattern in cell for pattern in PLACEHOLDERS):
        fail(f"{path.name}:{line}: placeholder or missing {label}")


def has_decision_range(decisions: str) -> bool:
    return bool(re.search(r"D-\d{3}`?\s*(?:-|–|—)\s*`?(?:D-)?\d{3}", decisions))


def parse_vectors(spec: Path) -> dict[str, VectorDefinition]:
    definitions: dict[str, VectorDefinition] = {}
    for path in sorted(spec.glob("*.md")):
        table_active = False
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if line == VECTOR_HEADER:
                table_active = True
                continue
            if table_active and not line.startswith("|"):
                table_active = False
                continue
            match = VECTOR_ROW.match(line) if table_active else None
            if match is None:
                continue
            row = cells(line)
            if len(row) != 6:
                fail(f"{path.name}:{number}: vector definition must have six cells")
            identifier, category, applicability, source, expected, decisions = row
            if identifier.strip("`") != match.group(1):
                fail(f"{path.name}:{number}: malformed vector identifier")
            identifier = match.group(1)
            if category not in CATEGORIES:
                fail(f"{path.name}:{number}: invalid category {category!r}")
            require_concrete(applicability, "Applicability", path, number)
            require_concrete(source, "Source/Input", path, number)
            require_concrete(expected, "Expected observable", path, number)
            decision_ids = frozenset(DECISION_ID.findall(decisions))
            if not decision_ids or "..." in decisions or re.search(r"\bthrough\b", decisions, re.I) or has_decision_range(decisions):
                fail(f"{path.name}:{number}: explicit, non-range D-ID coverage required")
            if identifier in definitions:
                first = definitions[identifier]
                fail(f"duplicate vector definition {identifier}: {first.path.name}:{first.line} and {path.name}:{number}")
            definitions[identifier] = VectorDefinition(identifier, decision_ids, path, number)
    if not definitions:
        fail("no recognized chapter vector definition tables")
    return definitions


def parse_clauses(spec: Path) -> set[str]:
    return {
        clause
        for path in spec.glob("*.md")
        for clause in CLAUSE_ID.findall(path.read_text(encoding="utf-8"))
    }


def verify_links(spec: Path) -> int:
    count = 0
    link = re.compile(r"\[[^]]+\]\(([^)#]+)(?:#([^)]+))?\)")
    for path in spec.glob("*.md"):
        for target, anchor in link.findall(path.read_text(encoding="utf-8")):
            if target.startswith(("http:", "https:", "mailto:")):
                continue
            destination = (path.parent / target).resolve()
            if not destination.exists():
                fail(f"{path.name}: unresolved link {target}")
            if anchor:
                text = destination.read_text(encoding="utf-8")
                if anchor not in text and f"#{anchor}" not in text:
                    fail(f"{path.name}: unresolved anchor #{anchor} in {target}")
            count += 1
    return count


def rebuild_matrix(root: Path) -> None:
    """Build the matrix only from manifests, draft headings, and vector definitions."""
    spec = root / "spec" / "iris-v1"
    definitions = parse_vectors(spec)
    decisions_to_vectors: dict[str, list[str]] = {}
    for identifier, definition in definitions.items():
        for decision in definition.decisions:
            decisions_to_vectors.setdefault(decision, []).append(identifier)
    manifest_clauses: dict[str, list[str]] = {}
    evidence = root / ".omo" / "evidence" / "iris-v1-spec"
    shorthand = re.compile(r"(?<!IRIS-V1-)(IDENTITY|GRAMMAR|RUNTIME|CONTROL|TYPES|COLLECTIONS|ASYNC|META|FFI|LIBRARY|MIGRATION|CONFORMANCE|TRACE)-C(\d+[A-Z]?)")
    for path in sorted(evidence.glob("trace-audit-d*.md")):
        for line in path.read_text(encoding="utf-8").splitlines():
            match = re.match(r"^\|\s*`?(D-\d{3})`?\s*\|", line)
            if match is None:
                continue
            found = [(clause.split("-")[2], clause.rsplit("-C", 1)[1]) for clause in CLAUSE_ID.findall(line)]
            if not found:
                found = shorthand.findall(" | ".join(cells(line)[1:3]))
            if found:
                manifest_clauses[match.group(1)] = [f"IRIS-V1-{chapter}-C{number}" for chapter, number in found]
    draft = (root / ".omo" / "drafts" / "iris-language-specification.md").read_text(encoding="utf-8")
    headings = {
        match.group(1): re.sub(r"\s+", " ", match.group(2)).strip()
        for match in re.finditer(r"- \*\*(D-\d{3}) [^*]*\*\* (.*?)(?= Owner decision|$)", draft, re.M)
    }
    output = [
        "# Iris v1 Traceability Matrix", "", "Status: Iris v1 draft, frozen semantics.", "",
        "IRIS-V1-TRACE-C019: This matrix is normative for semantic decision coverage. Every frozen decision appears exactly once and maps to manifest-authoritative semantic clauses and only direct chapter vector definitions.", "",
        "## Decision Rows", "",
        "| Decision | Frozen draft heading | Semantic clause anchors | Exact example or vector coverage | Migration row | Status |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for number in range(515):
        decision = f"D-{number:03d}"
        clauses = manifest_clauses.get(decision)
        if clauses is None:
            fail(f"cannot rebuild: manifest lacks clause mapping for {decision}")
        semantic = list(dict.fromkeys(clause for clause in clauses if clause not in NON_SEMANTIC))
        if not semantic:
            fail(f"cannot rebuild: manifest has only non-semantic anchors for {decision}")
        heading = headings.get(decision)
        if heading is None:
            fail(f"cannot rebuild: draft lacks heading for {decision}")
        anchors = ", ".join(
            f"[{clause}]({CHAPTER_FILES[clause.split('-')[2]]})" for clause in semantic
        )
        if decision in METADATA:
            status, reason = METADATA[decision]
            coverage = f"metadata-only: {reason}"
        else:
            vectors = sorted(decisions_to_vectors.get(decision, []))
            if not vectors:
                fail(f"cannot rebuild: no genuine vector definition explicitly covers {decision}")
            coverage = ", ".join(f"`{identifier}`" for identifier in vectors)
            status = "normative"
        output.append(f"| `{decision}` | {heading} | {anchors} | {coverage} | none | `{status}` |")
    (spec / "traceability-matrix.md").write_text("\n".join(output) + "\n", encoding="utf-8")


def validate(root: Path) -> tuple[int, int, int]:
    spec = root / "spec" / "iris-v1"
    matrix = spec / "traceability-matrix.md"
    documents = sorted(spec.glob("*.md"))
    if len(documents) != 14:
        fail(f"expected exactly 14 spec artifacts, found {len(documents)}")
    definitions = parse_vectors(spec)
    clauses = parse_clauses(spec)
    rows = [line for line in matrix.read_text(encoding="utf-8").splitlines() if MATRIX_ROW.match(line)]
    if len(rows) != 515:
        fail(f"expected 515 matrix rows, found {len(rows)}")
    seen: set[str] = set()
    for line in rows:
        row = matrix_cells(line)
        if len(row) != 6:
            fail(f"matrix row must have six cells: {line}")
        decision, heading, anchors, coverage, migration, status = row
        decision = decision.strip("`")
        if decision in seen:
            fail(f"duplicate matrix decision {decision}")
        seen.add(decision)
        if not heading:
            fail(f"missing draft heading for {decision}")
        if status.strip("`") not in VALID_STATUSES:
            fail(f"invalid status for {decision}: {status}")
        anchor_ids = set(CLAUSE_ID.findall(anchors))
        if not anchor_ids or not anchor_ids <= clauses:
            fail(f"unresolved or absent clause anchor for {decision}")
        if anchor_ids <= NON_SEMANTIC:
            fail(f"coverage-only anchor used as sole semantic anchor for {decision}")
        if decision in METADATA:
            required_status, reason = METADATA[decision]
            if "metadata-only:" not in coverage or reason not in coverage or status.strip("`") != required_status:
                fail(f"incorrect metadata-only treatment for {decision}")
            continue
        if "metadata-only" in coverage or "..." in coverage or re.search(r"\bthrough\b", coverage, re.I):
            fail(f"invalid coverage syntax for {decision}")
        vector_ids = set(VECTOR_ID.findall(coverage))
        if not vector_ids:
            fail(f"missing vector definition mapping for {decision}")
        if vector_ids & DELETED_BROAD_VECTORS:
            fail(f"{decision} references deleted broad vector {sorted(vector_ids & DELETED_BROAD_VECTORS)}")
        for identifier in vector_ids:
            definition = definitions.get(identifier)
            if definition is None:
                fail(f"{decision} references non-definition vector {identifier}")
            if decision not in definition.decisions:
                fail(f"{decision} is not explicitly covered by {identifier}")
    expected = {f"D-{number:03d}" for number in range(515)}
    if seen != expected:
        fail("matrix D-ID set is not exactly D-000 through D-514")
    for decision in ("D-491", "D-498"):
        row = next(row for row in rows if row.startswith(f"| `{decision}` |"))
        if matrix_cells(row)[5] != "`normative`":
            fail(f"{decision} must be normative")
    links = verify_links(spec)
    return len(definitions), len(rows), links


def mutate_case(root: Path, name: str) -> None:
    spec = root / "spec" / "iris-v1"
    matrix = spec / "traceability-matrix.md"
    text = matrix.read_text(encoding="utf-8")
    if name == "missing-d-row":
        text = text.replace("| `D-001` |", "| `D-515` |", 1)
        matrix.write_text(text, encoding="utf-8")
    elif name == "unrelated-vector":
        text = text.replace("IRIS-V1-RUNTIME-V052", "IRIS-V1-RUNTIME-V053", 1)
        matrix.write_text(text, encoding="utf-8")
    elif name == "coverage-only-anchor":
        text = text.replace("IRIS-V1-RUNTIME-C101](03-runtime-object-model.md), [IRIS-V1-RUNTIME-C135", "IRIS-V1-RUNTIME-C156](03-runtime-object-model.md), [IRIS-V1-RUNTIME-C156", 1)
        matrix.write_text(text, encoding="utf-8")
    elif name == "wrong-metadata-only":
        text = text.replace("`IRIS-V1-RUNTIME-V052`", "metadata-only: not permitted", 1)
        matrix.write_text(text, encoding="utf-8")
    elif name == "broken-link":
        text = text.replace("(03-runtime-object-model.md)", "(missing.md)", 1)
        matrix.write_text(text, encoding="utf-8")
    elif name == "reference-only-vector":
        text = text.replace("IRIS-V1-RUNTIME-V052", "IRIS-V1-RUNTIME-V999", 1)
        matrix.write_text(text, encoding="utf-8")
    elif name == "deleted-broad-vector":
        text = text.replace("IRIS-V1-RUNTIME-V052", "IRIS-V1-GRAMMAR-V008", 1)
        matrix.write_text(text, encoding="utf-8")
    else:
        path = spec / "03-runtime-object-model.md"
        content = path.read_text(encoding="utf-8")
        if name == "placeholder-source":
            content = content.replace("`((2 ** 200) + 1) - (2 ** 200)`", "Concrete D-003 scenario", 1)
        elif name == "missing-expected":
            content = content.replace("`Integer(1)` with Type `Integer`; interpreter and JIT agree; no representation Type split.", "", 1)
        elif name == "duplicate-vector-definition":
            row = next(line for line in content.splitlines() if "IRIS-V1-RUNTIME-V052" in line)
            content = content.replace(row, f"{row}\n{row}", 1)
        elif name == "range-decision-metadata":
            content = content.replace("`D-003`", "`D-003`-`D-004`", 1)
        path.write_text(content, encoding="utf-8")


def self_test() -> None:
    cases = (
        "missing-d-row",
        "placeholder-source",
        "missing-expected",
        "unrelated-vector",
        "reference-only-vector",
        "range-decision-metadata",
        "coverage-only-anchor",
        "duplicate-vector-definition",
        "deleted-broad-vector",
        "wrong-metadata-only",
        "broken-link",
    )
    with tempfile.TemporaryDirectory(prefix="iris-traceability-") as temporary:
        root = Path(temporary) / "repo"
        shutil.copytree(ROOT / "spec", root / "spec")
        for name in cases:
            case_root = root.parent / name
            shutil.copytree(root, case_root)
            mutate_case(case_root, name)
            try:
                validate(case_root)
            except ValidationFailure:
                shutil.rmtree(case_root)
                continue
            fail(f"self-test unexpectedly passed: {name}")
    print("PASS self_test=missing-d-row,placeholder-source,missing-expected,unrelated-vector,reference-only-vector,range-decision-metadata,coverage-only-anchor,duplicate-vector-definition,deleted-broad-vector,wrong-metadata-only,broken-link")


def main() -> None:
    try:
        if "--self-test" in sys.argv:
            self_test()
            return
        definitions, rows, links = validate(ROOT)
    except ValidationFailure as error:
        print(f"FAIL: {error}")
        raise SystemExit(1) from error
    print(f"PASS artifacts=14 rows={rows} vector_definitions={definitions} links={links} coverage_gaps=0")


if __name__ == "__main__":
    main()
