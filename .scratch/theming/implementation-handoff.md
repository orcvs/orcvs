# Orcvs theming implementation handoff

Updated 2026-09-22 after the design interview. This supersedes the Base16-centred
temporary audit as an implementation entry point. Product decisions are in
ADR 0053; `schema.md` defines the complete format, dark values, parser and rendering mappings.
No runtime implementation has been performed in this preparation branch.

## Settled scope

One YAML document format: `format: orcvs-theme`, `version: 1`, `name`, `inherits`
and `style`. No Base16 palette, importer or conversion work is required now.
Named properties cover Source colours, console colours, alpha, fills and bounded
border widths. Custom files inherit exactly one built-in and its appearance.
Omission inherits; only the two optional Cursor fills also support explicit clear.
Transparent is a supplied colour, not an absent optional fill.

Native files live in `~/.orcvs/themes/` and are read at startup only. Web uses
file imports and retains documents with persistence. Filename stems are identities;
native duplicate identities are errors, successful web reimports replace a document.
Built-in identities are reserved. Invalid input is rejected whole. Fallback keeps
the saved selection intact and uses the built-in of the same appearance.

Widths stay constant in display points across zoom: Grid/Cell 0–1, chrome 0–2.
Zero hides strokes. Cursor/Region animation modulates nominal width without zoom
scaling. The application backdrop is opaque; surfaces above it composite alpha.
Uniform `cell.background` sits beneath highlighting and does not block Region fallback.
Fact precedence stays fixed. Motion remains a separate setting.

## Delivery order and ownership

1. Use `schema.md` and the files in `examples/` as the implementation contract.
   The schema preparation is complete; implementation must verify the values
   through shipped rendering and loading paths.
2. Satisfy the existing Paint recovery and recorded benchmark-floor prerequisites.
   Read the floor comparison in CI; do not substitute a local benchmark number.
3. `06`: resolved named Theme, complete dark definition, pure inheritance resolver,
   Source painting and alpha/width fixes. Resolve finite paint combinations outside
   the per-Cell loop. Preserve the current normal-zoom dark appearance.
4. `03`: derive chrome, including previously inherited toolkit colours. Prepare
   pickers but retain `02`'s shared presentation: the same resolved Okabe–Ito
   style in both egui appearance slots, with matching Source rendering and
   stored preferences preserved. Distinct registration belongs to `04`.
5. `08`: contrast report using the same effective composites as painting. Keep
   Sequence's accepted failure visible. Invalid Number and Note Diagnostic
   states are known pending failures (4.446944:1 and 4.053689:1); confirm them
   through shipped rendering and obtain explicit acceptance before the gate passes.
6. `04`: complete the light proposal and produce actual console captures plus
   contrast results. Obtain user acceptance before distinct light/dark style
   registration and exposing mode/Theme pickers. This issue owns switching
   acceptance, including startup and OS appearance changes.
7. `07`: one parser and native/web transport, using `06`'s resolver. `10` completes
   the documented custom-authoring workflow and end-to-end acceptance on both
   loading paths. It adds neither a second parser nor an in-app editor.
8. `09`: preserve confirmed zero-amount, zero-frequency and reduced-motion
   behaviour, with no effect-only repaint deadline when the effect is stationary.

Steps are dependency order, not a requirement for one PR each. No usable
intermediate state may theme Source and chrome differently, including startup
restoration and OS appearance changes.

## Preparation complete; remaining implementation evidence

The named-property catalogue, exact dark values, weak/IME mappings, explicit
role backgrounds, parser subset, format/version handling, optional-fill encoding,
resource limits and native/web error behaviour are specified in `schema.md`.
`examples/okabe-ito-copy.yaml` supplies every property; `examples/my-dark.yaml`
is a small inherited custom Theme. The legacy schema-draft link redirects there.

The implementation must still verify actual paint/composite results, native/web
loading, feature combinations and error recovery. No runtime tests were run for
these documentation changes. The light palette is not approved: complete its
values and obtain acceptance of actual console captures and contrast results
under `04` before exposing switching. The two known dark Diagnostic contrast
failures also need explicit acceptance before all criteria can pass; they are
recorded in the schema and issue `08`, not automatically accepted. No further product-scope interview is
required to begin the implementation work described here.

## Verification for implementation

Follow the current repository contract and the Rust/egui skills at implementation
time. For console changes: formatting, console clippy and nextest; persistence-off
workspace tests for storage work; `mise run check_wasm` for platform/rendering.
Use the inspection feature gate only if touched. Run dependency audit if manifests,
features or lockfile change. Run workspace and doctest gates once before a PR.

Regression cases must cover inheritance, invalid documents, duplicate identities,
web replacement, fallback/autosave/repair, alpha composition, optional fills,
uniform Cell fill with Region fallback, fixed stroke widths at supported zoom/DPI,
motion scheduling and coherent Source/chrome switching. Use shipped rendering and
loading paths, not a separate demonstration UI. Capture light and dark through the
egui workflow; broad platform/browser/benchmark comparison work remains with CI.

## Workspace

Preparation lives in `.worktrees/one-theme-adr-0053`, on `one-theme-adr-0053`.
The main checkout has unrelated user changes; do not overwrite or absorb them.
Keep existing ticket filenames stable despite updated titles so references survive.
Historical Comments record superseded decisions and are not current requirements.
