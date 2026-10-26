//!
//! The native console window's size, position and state, stored beside the
//! Source so a relaunch returns the console to them.
//!
//! The console keeps this rather than eframe: eframe's own restore sizes the
//! first window before any of the console's code runs, and the first window
//! is the splash (ADR 0083).
//!

use egui::{NumExt as _, Pos2, Vec2, ViewportCommand};

///
/// Where the console window was and how big, in egui points.
///
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct WindowGeometry {
    /// The content area's size.
    pub(crate) inner_size: Vec2,
    /// The window frame's top-left corner. `None` where the platform reports no
    /// window position, as Wayland does.
    pub(crate) outer_position: Option<Pos2>,
    pub(crate) maximized: bool,
    pub(crate) fullscreen: bool,
}

impl WindowGeometry {
    ///
    /// The console window as egui last reported it.
    ///
    #[cfg(feature = "persistence")]
    pub(crate) fn of(ctx: &egui::Context) -> Self {
        let inner_size = ctx.content_rect().size();
        ctx.input(|input| {
            let viewport = input.viewport();
            Self {
                inner_size,
                outer_position: viewport.outer_rect.map(|rect| rect.min),
                maximized: viewport.maximized.unwrap_or(false),
                fullscreen: viewport.fullscreen.unwrap_or(false),
            }
        })
    }

    ///
    /// The commands that return a window to this geometry. The size is kept
    /// between `min_size` and the size of the monitor the window is on, where
    /// egui reports one, so a geometry stored on a larger monitor still fits.
    ///
    pub(crate) fn commands(
        self,
        min_size: Vec2,
        monitor_size: Option<Vec2>,
    ) -> Vec<ViewportCommand> {
        let mut size = self.inner_size.at_least(min_size);
        if let Some(monitor_size) = monitor_size {
            size = size.at_most(monitor_size);
        }
        let mut commands = vec![ViewportCommand::InnerSize(size)];
        if let Some(position) = self.outer_position {
            commands.push(ViewportCommand::OuterPosition(position));
        }
        if self.maximized {
            commands.push(ViewportCommand::Maximized(true));
        }
        if self.fullscreen {
            commands.push(ViewportCommand::Fullscreen(true));
        }
        commands
    }
}

///
/// The Storage key the console window's geometry lives under. eframe's own
/// `window` key is left unwritten: `persist_window` is off.
///
#[cfg(feature = "persistence")]
pub(crate) const WINDOW_KEY: &str = "orcvs_window";

///
/// The geometry the console window was last stored with.
///
#[cfg(feature = "persistence")]
pub(crate) fn stored(storage: Option<&dyn eframe::Storage>) -> Option<WindowGeometry> {
    eframe::get_value(storage?, WINDOW_KEY)
}

///
/// Without the `persistence` feature nothing is stored, so the console always
/// opens at its default size.
///
#[cfg(not(feature = "persistence"))]
pub(crate) fn stored(_storage: Option<&dyn eframe::Storage>) -> Option<WindowGeometry> {
    None
}

#[cfg(feature = "persistence")]
pub(crate) fn store(storage: &mut dyn eframe::Storage, geometry: &WindowGeometry) {
    eframe::set_value(storage, WINDOW_KEY, geometry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn geometry() -> WindowGeometry {
        WindowGeometry {
            inner_size: vec2(900.0, 640.0),
            outer_position: Some(pos2(120.0, 80.0)),
            maximized: false,
            fullscreen: false,
        }
    }

    #[test]
    fn a_geometry_returns_the_window_to_its_size_and_position() {
        assert_eq!(
            geometry().commands(vec2(300.0, 220.0), Some(vec2(1920.0, 1080.0))),
            vec![
                ViewportCommand::InnerSize(vec2(900.0, 640.0)),
                ViewportCommand::OuterPosition(pos2(120.0, 80.0)),
            ]
        );
    }

    #[test]
    fn a_maximized_or_fullscreen_window_is_returned_to_that_state() {
        let commands = WindowGeometry {
            maximized: true,
            fullscreen: true,
            ..geometry()
        }
        .commands(vec2(300.0, 220.0), None);
        assert!(commands.contains(&ViewportCommand::Maximized(true)));
        assert!(commands.contains(&ViewportCommand::Fullscreen(true)));
    }

    #[test]
    fn the_size_fits_between_the_minimum_and_the_monitor() {
        let large = WindowGeometry {
            inner_size: vec2(4000.0, 100.0),
            outer_position: None,
            ..geometry()
        };
        assert_eq!(
            large.commands(vec2(300.0, 220.0), Some(vec2(1920.0, 1080.0))),
            vec![ViewportCommand::InnerSize(vec2(1920.0, 220.0))]
        );
    }

    #[cfg(feature = "persistence")]
    #[test]
    fn a_stored_geometry_is_read_back() {
        let mut storage = crate::persistence::InMemoryStorage::default();
        assert_eq!(stored(Some(&storage)), None);
        store(&mut storage, &geometry());
        assert_eq!(stored(Some(&storage)), Some(geometry()));
    }
}
