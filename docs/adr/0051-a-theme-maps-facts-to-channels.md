# A theme maps facts to channels

Status: accepted; partly superseded by [ADR 0053](0053-one-theme-styles-the-whole-console.md), which replaces its affordance namespace, its storage rule, its Sequence assignment, and its concession of chrome to egui's `Visuals`. Its facts-and-channels framing stands. `.scratch/theming/issues/01` carries it. It took [ADR 0050](0050-the-language-map-answers-source-paint-per-cell.md) as the decision on what a Render Frame Cell carries; ADR 0050 has since been rejected and superseded by [ADR 0052](0052-a-claim-answers-whether-its-slot-is-written.md), under which a Cell carries the parser's Claim and `RenderCell::source_paint` answers the same facts. This ADR decides only what the console does with those facts once it has them. Numbered 0051 rather than 0050 because ADR 0050 is the other decision on the pull request where the two landed. It supersedes the colour-authority arrangement `.scratch/syntax-highlighting/issues/01` shipped, which no ADR recorded.

A Cell carries independent **facts**. The console draws them through independent **channels**. A theme is the mapping between the two, and nothing else decides it.

The facts a Cell carries today are the parser's claim (ADR 0044) — the Cells a slot covers, the Token its signature declared, and the Atom those Cells bound or none — together with whether the slot holds written content, whether the Cell lies in a root Function's Output Portal Reservation, and whether it is selected. Facts are answered by the language and by the console's own derivation. They are not colours.

The channels a Cell has are its glyph colour, its fill, its border, and the overlays drawn across it. A channel is a visual property with room for exactly one value per Cell.

**Which channel shows which fact is a theme decision, not a code decision.** `style.rs::operand_paint` currently hardcodes it: an invalid operand's Diagnostic always takes the glyph colour and the declared Token always takes the fill. Both facts do survive — `an_invalid_operand_draws_diagnostic_but_keeps_its_declared_tint` pins exactly that — so this is not a lost fact. It is a mapping expressed as control flow, which no theme can restate. A theme that wanted Diagnostic on the fill and the Token on the glyph would have to edit the function.

**`CellVisuals` is below the theme, not above it.** It carries `{ background, border, foreground }` — three resolved colours. By the time a Cell reaches the renderer its facts are gone, so the mapping has to happen before it, and today that means inside `style.rs`. Putting the mapping in the theme means the theme resolves facts to `CellVisuals`, and `style.rs` stops choosing.

**Syntax and affordance are different fact kinds and take different theme namespaces.** A Token is a spelling the Language Map answers. Diagnostic and Output Portal are neither: a Diagnostic is a verdict on whether content bound, and the syntax scope document names the Output Portal "the one derived fact… which interprets no spelling". This is the split VS Code makes between `tokenColors` and the `editorError.*` workbench keys, and it is why Diagnostic does not compete with Sequence for a syntax slot.

**The syntax namespace is base16.** Sixteen slots, `base00`–`base0F`, assigned by the semantic roles [base16's styling guidelines](https://github.com/chriskempson/base16/blob/main/styling.md) state — not by colour names, which that document does not give. One template maps Orcvs's Tokens onto the slots, and every published scheme drops in unchanged:

| Orcvs | Slot | base16 role |
|---|---|---|
| Source background | `base00` | Default Background |
| Comment | `base03` | Comments |
| Ordinary, Char, Atom | `base05` | Default Foreground, Delimiters, Operators |
| Sequence | `base08` | Markup Lists |
| Number | `base09` | Integers, Boolean, Constants |
| Note | `base0B` | Strings |
| Function | `base0D` | Functions, Methods |
| Bang | `base0E` | Keywords, Storage |

`base0A`, `base0C` and `base0F` are unassigned. Orcvs has seven Tokens and base16 has more slots than that; filling them for the sake of it would mint distinctions the language does not have.

**The affordance namespace is named keys, not slots.** Diagnostic, Output Portal, Fill tint strength, Region, selection and the Cursor are console affordances. They take named keys the way `editorError.foreground` does, defaulting from the chrome palette. A base16 scheme supplies no value for them, which is correct: they are not that scheme's business.

**A theme stores what was chosen, never what was resolved.** A scheme is named or its sixteen values are stored; per-role overrides are stored beside it, keyed by scheme so an override does not follow a viewer to a different scheme. Nothing stores a resolved colour. This is the property `SourcePaintSettings` lacks — it holds ten resolved `Color32` fields that persistence writes in full on every autosave, so a moved default, a changed template, or a newly shipped scheme cannot reach an install that has saved once.

## Rejected alternatives

**Fold the Source Paint roles into the per-theme `Visuals` palette.** `.scratch/theming/issues/03` already carries the counter-claim: glyph colour is a language concept, so the toolkit's theme model cannot define it. The console's presentation tokens are conceded to `Visuals`; the Tokens are not.

**Key syntax colours by Token directly instead of through base16 slots.** More precise, and worth nothing. A format no published scheme targets cannot accept Solarized, Gruvbox or Nord without a hand translation per scheme, and the ecosystem's schemes are the reason to have a format at all. Seven Tokens onto sixteen slots loses no distinction Orcvs makes.

**Give Diagnostic a base16 slot.** It is not a spelling, so it would take a slot from a Token that is. Editor templates that do this put errors on `base08` by convention, which is exactly the collision to avoid: `base08` is where Sequence belongs by the spec's own "Markup Lists".

**Decide which channel Diagnostic draws on.** Not an architectural question. Glyph colour, fill or border are all legitimate, and the theme says which.

## Consequences

`style.rs::operand_paint` and `claim_paint` stop choosing channels. They answer facts; the theme resolves facts to a `CellVisuals`.

`style.rs`'s fixed hex assertions cannot survive an arbitrary scheme. They become a contrast validator over a scheme and the template, reporting each Token's measured ratio against `base00` and which floor it fails, plus a test that every shipped scheme passes. The named exceptions the current table pins — Sequence below the 4.5:1 floor, Bang, Function and Diagnostic below Comment — are facts about Okabe–Ito, not rules a scheme must satisfy.

`SourcePaintSettings` and the `source_paint` storage key are replaced rather than migrated. The encoding is ten `r,g,b` groups with no room for "unset", so an existing string would decode as ten deliberate overrides and preserve the behaviour being removed. A new key and an unread old one is the whole migration; the Orcvs language is pre-release and the console has no external installs.

`.scratch/theming/issues/05` — chrome seeding four `Visuals` fields from the restored Source colours — is answered by deletion. Chrome affordances take chrome keys, so nothing at startup needs the restored Source scheme, and the ordering defect it describes has nothing left to order.

The default scheme is Okabe–Ito expressed as base16, so the shipped appearance does not change when this lands. Okabe–Ito is a qualitative palette chosen for colour-vision deficiency, not a base16 scheme, and the two unassigned accent slots it has no value for take its nearest greyscale.
