//!
//! Which Theme the console presents for each appearance: the viewer's dark
//! and light Theme selections, and the Themes they are chosen from.
//!
//! ADR 0053's settings are a dark Theme, a light Theme and a mode. The mode
//! is egui's own `ThemePreference` (`.scratch/theming/issues/02`'s "one
//! owner" comment), which egui memory already stores and eframe already
//! restores. The two Theme selections are this module's: identities, never
//! values, so an improved built-in reaches every install.
//!

use crate::theme::{
    Appearance, OKABE_ITO_IDENTITY, ORCVS_LIGHT_IDENTITY, Theme, okabe_ito, orcvs_light,
};

///
/// One value per appearance, read and written by the appearance's name
/// rather than by a `match` at every use.
///
#[derive(Clone, Debug, PartialEq, Eq)]
struct ByAppearance<T> {
    dark: T,
    light: T,
}

impl<T> ByAppearance<T> {
    fn from_fn(mut value: impl FnMut(Appearance) -> T) -> Self {
        Self {
            dark: value(Appearance::Dark),
            light: value(Appearance::Light),
        }
    }

    fn get(&self, appearance: Appearance) -> &T {
        match appearance {
            Appearance::Dark => &self.dark,
            Appearance::Light => &self.light,
        }
    }

    fn get_mut(&mut self, appearance: Appearance) -> &mut T {
        match appearance {
            Appearance::Dark => &mut self.dark,
            Appearance::Light => &mut self.light,
        }
    }
}

///
/// The viewer's dark and light Theme selections, as Theme identities.
///
/// A selection is kept as chosen even when nothing presents it — a restored
/// identity no available Theme of that appearance answers to still saves back
/// unchanged (ADR 0053: the fallback "never replaces the saved Theme
/// selection, including on autosave").
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ThemeSelection(ByAppearance<String>);

impl Default for ThemeSelection {
    ///
    /// Each appearance's default built-in: `okabe-ito` for dark, and
    /// `orcvs-light` for light.
    ///
    fn default() -> Self {
        Self(ByAppearance::from_fn(|appearance| {
            default_identity(appearance).to_owned()
        }))
    }
}

impl ThemeSelection {
    ///
    /// A restored selection, holding whatever identities storage held.
    ///
    #[cfg_attr(
        not(any(feature = "persistence", test)),
        expect(
            dead_code,
            reason = "only a restore constructs a selection other than the default, and \
                      without `persistence` nothing is restored"
        )
    )]
    pub(crate) fn new(dark: String, light: String) -> Self {
        Self(ByAppearance { dark, light })
    }

    /// The identity selected for `appearance`.
    pub(crate) fn identity(&self, appearance: Appearance) -> &str {
        self.0.get(appearance)
    }
}

///
/// The built-in Theme each appearance falls back to when its selection names
/// no available Theme of that appearance.
///
pub(crate) fn default_identity(appearance: Appearance) -> &'static str {
    match appearance {
        Appearance::Dark => OKABE_ITO_IDENTITY,
        Appearance::Light => ORCVS_LIGHT_IDENTITY,
    }
}

///
/// The Themes a viewer chooses from, the current selection, and the Theme
/// that selection presents for each appearance.
///
/// The presented pair is resolved when the selection changes, not per
/// frame, so a Render Frame only borrows the one its appearance names.
///
#[derive(Clone, Debug)]
pub(crate) struct SelectedThemes {
    /// Every selectable Theme: the built-ins, which hold both appearances'
    /// defaults and so make the fallback in `resolve_slot` total.
    available: Vec<Theme>,
    selection: ThemeSelection,
    presented: ByAppearance<Theme>,
}

impl SelectedThemes {
    ///
    /// The built-in Themes under `selection`.
    ///
    /// Only the built-ins are available: `.scratch/theming/issues/07` is what
    /// adds loaded Themes beside them, and until it does each picker lists
    /// one Theme. Tests stand loaded Themes in through the test-only
    /// `with_stand_ins`, beside this rather than through it.
    ///
    pub(crate) fn new(selection: ThemeSelection) -> Self {
        Self::resolved(vec![okabe_ito(), orcvs_light()], selection)
    }

    ///
    /// `available` under `selection`. `available` holds both appearances'
    /// default built-ins.
    ///
    fn resolved(available: Vec<Theme>, selection: ThemeSelection) -> Self {
        let presented =
            ByAppearance::from_fn(|appearance| resolve_slot(&available, &selection, appearance));
        Self {
            available,
            selection,
            presented,
        }
    }

    /// The selection as the viewer made it, for saving.
    #[cfg_attr(
        not(any(feature = "persistence", test)),
        expect(
            dead_code,
            reason = "read only by a save, and without `persistence` nothing is saved"
        )
    )]
    pub(crate) fn selection(&self) -> &ThemeSelection {
        &self.selection
    }

    /// The Theme presented while the console's appearance is `appearance`.
    pub(crate) fn presented(&self, appearance: Appearance) -> &Theme {
        self.presented.get(appearance)
    }

    /// The Themes a picker for `appearance` lists: only that appearance's.
    pub(crate) fn listed(&self, appearance: Appearance) -> impl Iterator<Item = &Theme> {
        self.available
            .iter()
            .filter(move |theme| theme.appearance == appearance)
    }

    ///
    /// Selects `identity` for `appearance`, and presents the Theme it names.
    ///
    /// This is a viewer's choice, not a fallback, so it replaces whatever the
    /// selection held — a restored identity nothing answered to included.
    ///
    pub(crate) fn select(&mut self, appearance: Appearance, identity: &str) {
        identity.clone_into(self.selection.0.get_mut(appearance));
        *self.presented.get_mut(appearance) =
            resolve_slot(&self.available, &self.selection, appearance);
    }

    ///
    /// Registers each appearance's presented Theme as egui's style for that
    /// appearance ([`crate::style::install`]).
    ///
    pub(crate) fn install(&self, ctx: &egui::Context) {
        crate::style::install(
            ctx,
            self.presented(Appearance::Dark),
            self.presented(Appearance::Light),
        );
    }
}

///
/// The Theme `selection` presents for `appearance`: the available Theme of
/// that appearance it names, or that appearance's default built-in when it
/// names none (spec: "use the default built-in Theme of the same
/// appearance while retaining the saved selection").
///
fn resolve_slot(available: &[Theme], selection: &ThemeSelection, appearance: Appearance) -> Theme {
    let wanted = selection.identity(appearance);
    let named = |identity: &str| {
        available
            .iter()
            .find(|theme| theme.appearance == appearance && theme.identity == identity)
    };
    named(wanted)
        .or_else(|| named(default_identity(appearance)))
        .expect("the available Themes hold every appearance's default built-in")
        .clone()
}

///
/// A second dark Theme, `my-dark`, standing in for one
/// `.scratch/theming/issues/07` will load: built here, because nothing
/// shipped makes one yet. Its window, panel, input and Grid backgrounds
/// differ from every other Theme's here, so a test can tell which Theme
/// painted a frame.
///
#[cfg(test)]
pub(crate) fn my_dark() -> Theme {
    use egui::Color32;

    Theme {
        identity: "my-dark".to_owned(),
        name: "My Dark".to_owned(),
        window_background: Color32::from_rgb(0x20, 0x10, 0x30),
        panel_background: Color32::from_rgb(0x20, 0x10, 0x30),
        grid_background: Color32::from_rgb(0x10, 0x00, 0x20),
        input_background: Color32::from_rgb(0x18, 0x08, 0x28),
        ..okabe_ito()
    }
}

///
/// A second light Theme, `my-light`, standing in for a loaded light Theme the
/// way [`my_dark`] does for a dark one.
///
#[cfg(test)]
pub(crate) fn my_light() -> Theme {
    use egui::Color32;

    Theme {
        identity: "my-light".to_owned(),
        name: "My Light".to_owned(),
        window_background: Color32::from_rgb(0xF4, 0xEC, 0xDC),
        panel_background: Color32::from_rgb(0xF4, 0xEC, 0xDC),
        grid_background: Color32::from_rgb(0xFE, 0xF8, 0xEE),
        input_background: Color32::from_rgb(0xFA, 0xF2, 0xE4),
        ..orcvs_light()
    }
}

///
/// The built-ins with [`my_dark`] and [`my_light`] beside them, under
/// `selection`: what `.scratch/theming/issues/07`'s loaded Themes will make
/// available, built beside [`SelectedThemes::new`] rather than through it.
///
#[cfg(test)]
pub(crate) fn with_stand_ins(selection: ThemeSelection) -> SelectedThemes {
    SelectedThemes::resolved(
        vec![okabe_ito(), my_dark(), orcvs_light(), my_light()],
        selection,
    )
}

#[cfg(test)]
mod tests {
    use super::{SelectedThemes, ThemeSelection, my_dark, my_light, with_stand_ins};
    use crate::theme::{Appearance, okabe_ito, orcvs_light};

    #[test]
    fn a_default_selection_presents_each_appearances_built_in() {
        let themes = SelectedThemes::new(ThemeSelection::default());
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert_eq!(*themes.presented(Appearance::Light), orcvs_light());
        assert_eq!(themes.selection().identity(Appearance::Dark), "okabe-ito");
        assert_eq!(
            themes.selection().identity(Appearance::Light),
            "orcvs-light"
        );
    }

    #[test]
    fn each_picker_lists_only_the_themes_of_its_appearance() {
        let themes = with_stand_ins(ThemeSelection::default());
        let listed = |appearance| {
            themes
                .listed(appearance)
                .map(|theme| theme.identity.as_str())
                .collect::<Vec<_>>()
        };
        assert_eq!(listed(Appearance::Dark), ["okabe-ito", "my-dark"]);
        assert_eq!(listed(Appearance::Light), ["orcvs-light", "my-light"]);
    }

    #[test]
    fn a_restored_selection_presents_the_themes_it_names() {
        let themes = with_stand_ins(ThemeSelection::new(
            "my-dark".to_owned(),
            "my-light".to_owned(),
        ));
        assert_eq!(*themes.presented(Appearance::Dark), my_dark());
        assert_eq!(*themes.presented(Appearance::Light), my_light());
    }

    ///
    /// An identity no available Theme answers to, and one whose Theme is of
    /// the other appearance — the `okabe-ito` every earlier build saved into
    /// the light key — both present that appearance's default built-in, and
    /// neither is rewritten.
    ///
    #[test]
    fn an_unavailable_or_wrong_appearance_selection_falls_back_and_is_kept() {
        let selection = ThemeSelection::new("missing".to_owned(), "okabe-ito".to_owned());
        let themes = SelectedThemes::new(selection.clone());
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert_eq!(*themes.presented(Appearance::Light), orcvs_light());
        assert_eq!(*themes.selection(), selection);
    }

    #[test]
    fn selecting_a_theme_changes_only_its_own_appearance() {
        let mut themes = with_stand_ins(ThemeSelection::default());

        themes.select(Appearance::Dark, "my-dark");
        assert_eq!(*themes.presented(Appearance::Dark), my_dark());
        assert_eq!(*themes.presented(Appearance::Light), orcvs_light());
        assert_eq!(themes.selection().identity(Appearance::Dark), "my-dark");

        themes.select(Appearance::Light, "my-light");
        assert_eq!(*themes.presented(Appearance::Dark), my_dark());
        assert_eq!(*themes.presented(Appearance::Light), my_light());
        assert_eq!(themes.selection().identity(Appearance::Light), "my-light");

        themes.select(Appearance::Dark, "okabe-ito");
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert_eq!(*themes.presented(Appearance::Light), my_light());
    }

    ///
    /// Picking a Theme is the viewer's choice, not a fallback: picking the
    /// built-in a kept, unresolvable selection falls back to replaces that
    /// selection, even though what is presented does not change.
    ///
    #[test]
    fn picking_the_fallback_theme_replaces_a_kept_selection() {
        let mut themes = SelectedThemes::new(ThemeSelection::new(
            "missing".to_owned(),
            "okabe-ito".to_owned(),
        ));

        themes.select(Appearance::Dark, "okabe-ito");
        themes.select(Appearance::Light, "orcvs-light");

        assert_eq!(*themes.selection(), ThemeSelection::default());
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert_eq!(*themes.presented(Appearance::Light), orcvs_light());
    }
}
