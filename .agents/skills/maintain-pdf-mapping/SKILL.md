---
name: maintain-pdf-mapping
description: Inspect, diagnose, test, or update character-wizard's lopdf AcroForm rendering and opaque character-sheet.pdf field mappings. Use for missing, misplaced, malformed, or new PDF values, checkbox states, appearance streams, and changes to src/pdf_renderer; do not use for game-rule changes with no PDF impact.
---

# Maintain PDF mapping

Map fields from evidence: form metadata, annotation rectangles, visible labels,
and write/read-back tests. The template's names are opaque and must not be guessed.

## Workflow

1. Read `references/acroform.md` before changing mappings.
2. Inspect the repository's development-only `assets/character-sheet.pdf` fixture,
   or the explicit user-supplied template under diagnosis, with the Rust inventory
   helpers. Capture field name, type, page, rectangle, parent field, current value,
   and checkbox appearance states.
3. Correlate annotation coordinates with extracted page labels. If ambiguity
   remains, create a temporary diagnostic PDF with unmistakable marker values and
   inspect it; never commit diagnostic output.
4. Update `render_character()` projections. Keep calculations on `Character`.
5. Handle buttons with their actual on-state name rather than assuming `/Yes`.
6. Keep the frozen template catalog and `validate_template()` aligned with any new
   mapped fields.
7. Render a sample through an explicit template path, reopen it with lopdf,
   and assert stored values. Preserve both pages and the AcroForm.
8. Add or update focused tests under `src/pdf_renderer/`, then invoke
   `$verify-pc-wizard`.

## Safety

- Never overwrite `assets/character-sheet.pdf`.
- Never copy the official template into package assets or release artifacts.
- Do not flatten the form unless explicitly requested.
- Do not infer semantic meaning from numeric field names alone.
- Keep mapping constants centralized in `src/pdf_renderer`.
