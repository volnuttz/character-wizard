---
name: add-srd-content
description: Extend or correct character-wizard game data and character-creation behavior from assets/SRD_CC_v5.2.1.pdf. Use for classes, backgrounds, species, feats, spells, equipment, proficiencies, ability scores, derived rules, or wizard choices sourced from the supplied SRD; do not use for PDF field mapping alone.
---

# Add SRD content

Keep implementation traceable to the supplied SRD and consistent across rules,
models, prompts, serialization, rendering, and tests.

## Workflow

1. Read `references/content-map.md` to identify affected modules and invariants.
2. Locate the rule in `assets/SRD_CC_v5.2.1.pdf`; record the PDF page in
   working notes. Do not rely on memory or another D&D edition.
3. Inspect existing structures in `src/srd_data`, domain derivations, creation prompts,
   PDF output, and tests before editing.
4. Model structured choices explicitly. Keep display text separate from numeric
   mechanics when calculations depend on a choice.
5. Treat character JSON as current-schema-only. Do not add schema-version fields,
   migration code, compatibility aliases, or legacy-shape fallbacks. Update the
   canonical fixture and validation tests when the schema changes.
6. Add focused rule/model tests and prompt tests where interaction changes. Add a
   PDF assertion if the new value is rendered.
7. Invoke `$verify-pc-wizard` after implementation.

## Boundaries

- Treat `assets/SRD_CC_v5.2.1.pdf` as authoritative.
- Do not edit either source PDF.
- Do not add non-SRD content without an explicit product decision and clear
  labeling.
- Avoid embedding calculations in terminal prompt or CLI presentation code.
- For opaque PDF fields, invoke `$maintain-pdf-mapping`.
