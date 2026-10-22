# 05 — Keep the web's missing inputs to one module each

**What to build:** The items only a Theme document uses move from `console/src/theme.rs` into `console/src/theme_document.rs`: `ThemeDocument`, `ThemeError`, `resolve`, `OptionalFill`, the key enums' `name` and the generated setters from ticket 02. The web reads no Theme document and no settings, and today that is said by the same `cfg_attr(…, expect(dead_code, reason = "only native discovery loads a Theme document…"))` written 13 times in `theme.rs`, 5 in `theme_registry.rs` and twice in `lib.rs`. After the move, `theme_document`'s one module-level expectation covers them, and `theme.rs` keeps the resolved `Theme` and the built-ins, which is all the web uses. The same root cause appears under its own reason on `ThemeSelection::new` (`console/src/theme_selection.rs`), which only the native settings file (`config.rs`) calls: it is compiled only where settings are read (`#[cfg(any(not(target_arch = "wasm32"), test))]`), so it needs no expectation. The `mod contrast` expectation in `lib.rs` stays: one attribute on a module the web never reaches is already the narrowest form. Native discovery and every item's behaviour are unchanged.

**Blocked by:** 02 — Route Theme properties from the catalogue.

**Status:** resolved

- [x] `ThemeSelection::new` carries no `dead_code` expectation.
- [x] No item in `theme.rs` carries the web `dead_code` expectation, and the attributes left in `theme_registry.rs` and `lib.rs` are only those its items still need, each with its reason.
- [x] `theme.rs` holds the resolved `Theme`, its accessors the console paints from and the built-ins; `theme_document` holds what reads, resolves and refuses a document.
- [x] `mise run check_wasm` passes, so the web target still compiles with no unexpected dead code and no unfulfilled expectation.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and `cargo nextest run --workspace --tests --no-default-features --locked` pass.

## Comments

- The key enums moved too, along with their `name` and the setters, and so did `ThemeIdentity`'s `from_stem`, `is_reserved` and `as_str`. To make that move possible, `ThemeIdentity`'s field is `pub(crate)`. An identity carries no invariant beyond `from_stem`'s non-empty check, and `configured` already skips that check.
- The width keys' `name` is read only by tests, so it compiles only under `cfg(test)`.
- The five `theme_registry.rs` expectations and the two in `lib.rs` stay. Each sits on an item the web compiles and never reaches. So the spec's "twenty become one" comes to thirteen fewer, plus `ThemeSelection::new`.
