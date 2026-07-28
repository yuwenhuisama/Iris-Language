# Iris v1 Frozen Spec Defects

The frozen spec must never be edited. Defects are recorded here only.

Standing rule: chapter tables are authoritative for vector content, and chapter 12 is authoritative for record schema shape.

| defect ID | spec artifact | exact line | reading A | reading B | chosen implementation behavior | status |
| --- | --- | --- | --- | --- | --- | --- |
| `IRIS-V1-GRAMMAR-V008` | `spec/iris-v1/12-conformance.md` | `261` | Chapter 12 references `IRIS-V1-GRAMMAR-V008` inside the `IRIS-V1-CONFORMANCE-V011` JSON fixture. | `spec/iris-v1/02-lexical-grammar.md` has no defining row for `IRIS-V1-GRAMMAR-V008`. | The ID is out of scope for milestone 1 and must not be implemented or invented. | Open |
| `IRIS-V1-GRAMMAR-V003` | `spec/iris-v1/02-lexical-grammar.md`; `spec/iris-v1/12-conformance.md` | `551`; `166` | The chapter 02 row separates the three source forms with `;`: `2 ** 3 ** 2; -2 ** 2; 2 ** -3`. | The chapter 12 normative sample record uses `\n` separators in `source_text`: `2 ** 3 ** 2\n-2 ** 2\n2 ** -3`. | Chapter tables are authoritative for vector content, and chapter 12 is authoritative for record schema shape, so the implementation follows the chapter 02 content while keeping the chapter 12 record structure. | Open |
