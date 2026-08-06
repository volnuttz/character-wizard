# AcroForm notes

The supported official template has two pages and hundreds of mostly opaque fields
such as `Text19` and `Check Box5`. The repository copy is a development/test
fixture only and is excluded from distributions. Rendering resolves an explicit
`--template`, a current-directory copy, or a validated user-cache copy before
downloading and caching the supported official sheet.

Recursive AcroForm fields and individual page widgets may differ, so inspect both
the catalog and page annotations.

For each `/Annots` entry, resolve `/T` directly or through `/Parent`, and record
`/FT`, `/Rect`, `/V`, `/AS`, and `/AP` `/N` keys. PDF coordinates start at the
bottom-left. Sort by descending lower Y coordinate and ascending X coordinate to
compare widgets with extracted labels from top to bottom.

Rendering uses lopdf object updates so the AcroForm is retained. Verify output by
reopening it with `read_field_values()`; also assert page count. Visual placement
still requires coordinate correlation or viewing a marked diagnostic PDF.

Existing mappings and compatibility checks live in `render_character()`,
projection helpers, template inventory validation, appearance generation, and
field read-back helpers under `src/pdf_renderer/`. Runtime template discovery and
download live in `src/template.rs`.
