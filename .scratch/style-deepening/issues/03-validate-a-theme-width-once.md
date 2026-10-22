# 03 — Validate a Theme width once

**What to build:** The Theme document decoder (`console/src/theme_document.rs`, the width reads) produces `GridWidth` and `ChromeWidth` values, and `ThemeDocument` holds those typed widths instead of bare `f32`s. `resolve` (`console/src/theme.rs`) then assigns them and can no longer fail on a width, so `WidthError`, `width_error` and the `NonFiniteWidth` and `WidthOutOfRange` variants of `ThemeError` are deleted. The decoder's own finiteness and range check stays and keeps naming the key it read; an `f64` in range narrows to an `f32` still in range, so the typed constructor cannot refuse after it.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] A width's bound is checked in one place, and `resolve` has no width error path.
- [x] `WidthError`, `width_error`, `ThemeError::NonFiniteWidth` and `ThemeError::WidthOutOfRange` are gone, and `theme_document`'s module doc no longer says the bound is checked twice.
- [x] A non-finite, negative or over-range width in a Theme document is still refused by the decoder with the key it read, for a Grid width and a chrome width alike; the existing decoder refusal tests pass unchanged or move with their assertions intact.
- [x] Every shipped and test Theme document resolves exactly as before.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and `cargo nextest run --workspace --tests --no-default-features --locked` pass.

## Comments

- `GridWidth`/`ChromeWidth::from_points` still check their bound, now answering `Option`, because the built-ins construct widths. On the document path the decoder's `f64` check comes first, and `from_points(..).expect(..)` after it cannot fail, since an in-range `f64` narrows to an in-range `f32`.
