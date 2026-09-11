//!
//! The console's error channel: one report, on whichever channel the target
//! being built actually reads.
//!
//! What is reported is the same on both targets; what reads a report is not.
//! The native binary installs a `tracing` subscriber (`console/src/main.rs`) and
//! reads `tracing`. The browser build installs `eframe::WebLogger` instead,
//! which forwards `log` records to the developer console and knows nothing of
//! `tracing`; with no `tracing` subscriber on that target a `tracing` event is
//! dropped where nobody sees it. Putting the report on both channels is what
//! makes "reported" true in the browser as well.
//!
//! `log` is already a `wasm32`-only dependency of this crate, so the second
//! line costs no manifest change. The alternative — enabling `tracing`'s `log`
//! feature in the workspace manifest, which bridges every `tracing` event to
//! `log` wherever no subscriber is installed — is one line but a much wider
//! change: it routes every event in the workspace to the browser console at
//! `WebLogger`'s `Debug` filter, and `orcvs/src/source/model.rs` emits `debug!`
//! on every Cell write, which is the Tick and edit path. Turning a handful of
//! error reports into per-Cell console traffic in the browser is an unmeasured
//! cost on a hot path, so the second channel is carried here, where only the
//! sites that ask for it pay for it.
//!
//! This module is the one place that knows any of that. A caller reports an
//! error; which channels that reaches is not its concern, and no call site
//! carries a target `cfg` of its own.
//!

///
/// Reports an error on every channel this target reads.
///
/// The arguments are `format!`'s, so a caller writes the message it would have
/// written to either logger and nothing is formatted unless a channel is
/// there to take it — the report costs no allocation of its own, which matters
/// at the `Console::ui` site, where it sits on a path that runs many times per
/// second.
///
/// It is a macro rather than a function for that reason: a function would take
/// either a `String`, which allocates on a path that has none today, or a
/// `std::fmt::Arguments`, which puts `format_args!` in front of every message
/// and hands the caller back the ceremony this module exists to remove.
///
/// One message and no structured fields: `log` has no counterpart to
/// `tracing`'s fields, and a report that carries less on one target than the
/// other is the difference this module exists to close. A value worth
/// recording goes in the message.
///
/// Every call site is conditional — the storage seam's on the `persistence`
/// feature, the Playback failure report's on the target — so a native build
/// with `--no-default-features` compiles no caller at all. That is a fact
/// about which sites that build contains, not about the channel: this is the
/// console's error channel on every target, and the next site to need it must
/// not have to re-derive what it knows. `unused_macros`, and the
/// `unused_imports` the re-export below draws with it, cannot see that, so
/// they are silenced on these two items and nowhere else.
#[allow(unused_macros)]
macro_rules! error {
    ($($argument:tt)*) => {{
        ::tracing::error!($($argument)*);
        // The browser reads `log`, and only the browser: on native the
        // `tracing` line above is the whole report.
        #[cfg(target_arch = "wasm32")]
        ::log::error!($($argument)*);
    }};
}

#[allow(unused_imports)]
pub(crate) use error;
