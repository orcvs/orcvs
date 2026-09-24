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

use crate::theme::{Appearance, Theme, ThemeIdentity};
use crate::theme_registry::ThemeRegistry;

///
/// One value per appearance, read and written by the appearance's name
/// rather than by a `match` at every use.
///
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
pub(crate) struct ThemeSelection(ByAppearance<ThemeIdentity>);

impl Default for ThemeSelection {
    ///
    /// Each appearance's default built-in: `okabe-ito` for dark, and
    /// `orcvs-light` for light.
    ///
    fn default() -> Self {
        Self(ByAppearance::from_fn(ThemeIdentity::default_for))
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
    pub(crate) fn new(dark: ThemeIdentity, light: ThemeIdentity) -> Self {
        Self(ByAppearance { dark, light })
    }

    /// The identity selected for `appearance`.
    pub(crate) fn identity(&self, appearance: Appearance) -> &ThemeIdentity {
        self.0.get(appearance)
    }
}

///
/// The Themes a viewer chooses from, the current selection, and the Theme
/// that selection presents for each appearance.
///
/// The Themes are the [`ThemeRegistry`]'s: the built-ins and every loaded
/// Theme. The presented pair is resolved when the selection or the registry
/// changes, not per frame, so a Render Frame only borrows the one its
/// appearance names.
///
pub(crate) struct SelectedThemes {
    registry: ThemeRegistry,
    selection: ThemeSelection,
    presented: ByAppearance<Theme>,
    /// Why an appearance's selection is not what it presents, while it is
    /// not and until the viewer dismisses it.
    selection_notices: ByAppearance<Option<String>>,
}

impl SelectedThemes {
    ///
    /// The Themes `registry` holds, under `selection`.
    ///
    pub(crate) fn new(registry: ThemeRegistry, selection: ThemeSelection) -> Self {
        let presented = ByAppearance::from_fn(|appearance| registry.fallback(appearance).clone());
        let mut themes = Self {
            registry,
            selection,
            presented,
            selection_notices: ByAppearance::default(),
        };
        for appearance in [Appearance::Dark, Appearance::Light] {
            themes.resolve(appearance);
        }
        themes
    }

    ///
    /// Presents the Theme `appearance`'s selection names: the available
    /// Theme of that appearance, or that appearance's default built-in when
    /// it names none (spec: "use the default built-in Theme of the same
    /// appearance while retaining the saved selection"). A fallback raises a
    /// notice saying why; a notice that is new is also reported.
    ///
    fn resolve(&mut self, appearance: Appearance) {
        let identity = self.selection.identity(appearance);
        let (theme, notice) = match self.registry.select(appearance, identity) {
            Ok(theme) => (theme, None),
            Err(unavailable) => {
                let fallback = self.registry.fallback(appearance);
                let notice = format!(
                    "The selected {} Theme \"{identity}\" is unavailable: {unavailable}. {} is \
                     shown in its place. The saved selection is kept, and the Theme is shown \
                     once its file loads.",
                    appearance.word(),
                    fallback.name,
                );
                (fallback, Some(notice))
            }
        };
        *self.presented.get_mut(appearance) = theme.clone();
        if let Some(message) = &notice
            && self.selection_notices.get(appearance).as_ref() != Some(message)
        {
            crate::report::error!("{message}");
        }
        *self.selection_notices.get_mut(appearance) = notice;
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

    ///
    /// The Themes a picker for `appearance` lists: only that appearance's,
    /// built-ins first. A custom Theme's appearance is its parent's.
    ///
    pub(crate) fn listed(&self, appearance: Appearance) -> impl Iterator<Item = &Theme> {
        self.registry
            .themes()
            .filter(move |theme| theme.appearance == appearance)
    }

    ///
    /// Selects `identity` for `appearance`, and presents the Theme it names.
    ///
    /// This is a viewer's choice, not a fallback, so it replaces whatever the
    /// selection held — a restored identity nothing answered to included.
    ///
    pub(crate) fn select(&mut self, appearance: Appearance, identity: &ThemeIdentity) {
        identity.clone_into(self.selection.0.get_mut(appearance));
        self.resolve(appearance);
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

    /// Every notice the viewer has not dismissed: the selection notices
    /// first, then the registry's in the order they arose.
    pub(crate) fn notices(&self) -> impl Iterator<Item = &String> {
        [&self.selection_notices.dark, &self.selection_notices.light]
            .into_iter()
            .flatten()
            .chain(self.registry.notices())
    }

    pub(crate) fn notice_count(&self) -> usize {
        self.notices().count()
    }

    pub(crate) fn dismiss_notices(&mut self) {
        self.selection_notices = ByAppearance::default();
        self.registry.dismiss_notices();
    }

    /// Every notice, owned, so a test can index, compare and print them.
    #[cfg(test)]
    pub(crate) fn notice_list(&self) -> Vec<String> {
        self.notices().cloned().collect()
    }
}

// === Web import ===

#[cfg(any(target_arch = "wasm32", test))]
impl SelectedThemes {
    ///
    /// Imports one Theme file into the registry
    /// ([`ThemeRegistry::import`]). A valid document may be the Theme a
    /// selection names, or replace the one presented, so each appearance
    /// whose selection names it is resolved again; the caller installs the
    /// result. An appearance selecting another Theme is left alone, so a
    /// notice about it the viewer dismissed stays dismissed.
    ///
    pub(crate) fn import(&mut self, file_name: &str, bytes: &[u8]) -> Result<(), String> {
        let imported = self.registry.import(file_name, bytes)?;
        for appearance in [Appearance::Dark, Appearance::Light] {
            if *self.selection.identity(appearance) == imported {
                self.resolve(appearance);
            }
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl SelectedThemes {
    /// [`ThemeRegistry::refuse_conflicting_drops`].
    pub(crate) fn refuse_conflicting_drops(&mut self, file_names: Vec<String>) -> Vec<String> {
        self.registry.refuse_conflicting_drops(file_names)
    }

    /// [`ThemeRegistry::refuse_unreadable`].
    pub(crate) fn refuse_unreadable(&mut self, file_name: &str, error: &str) {
        self.registry.refuse_unreadable(file_name, error);
    }

    /// [`ThemeRegistry::store_imported`].
    #[cfg(feature = "persistence")]
    pub(crate) fn store_imported(&mut self, storage: &mut dyn eframe::Storage) {
        self.registry.store_imported(storage);
    }
}

#[cfg(test)]
mod tests {
    use super::{SelectedThemes, ThemeSelection};
    use crate::theme::{
        Appearance, OKABE_ITO_IDENTITY, ORCVS_LIGHT_IDENTITY, okabe_ito, orcvs_light,
    };
    use crate::theme_registry::ThemeRegistry;
    use crate::theme_registry::tests_support::{dark, id, my_dark, my_light, with_my_themes};

    fn mentions(themes: &SelectedThemes, needle: &str) -> bool {
        themes.notices().any(|notice| notice.contains(needle))
    }

    #[test]
    fn a_default_selection_presents_each_appearances_built_in_and_raises_no_notice() {
        let themes = SelectedThemes::new(ThemeRegistry::built_in(), ThemeSelection::default());
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert_eq!(*themes.presented(Appearance::Light), orcvs_light());
        assert_eq!(
            *themes.selection().identity(Appearance::Dark),
            OKABE_ITO_IDENTITY
        );
        assert_eq!(
            *themes.selection().identity(Appearance::Light),
            ORCVS_LIGHT_IDENTITY
        );
        assert_eq!(themes.notice_count(), 0, "{:?}", themes.notice_list());
    }

    ///
    /// `.scratch/theming/issues/07`: a loaded Theme is listed only in the
    /// picker matching its parent's appearance, after the built-ins.
    ///
    #[test]
    fn each_picker_lists_only_the_themes_of_its_appearance() {
        let themes = SelectedThemes::new(with_my_themes(), ThemeSelection::default());
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
    fn a_restored_selection_presents_the_loaded_themes_it_names() {
        let themes = SelectedThemes::new(
            with_my_themes(),
            ThemeSelection::new(id("my-dark"), id("my-light")),
        );
        assert_eq!(*themes.presented(Appearance::Dark), my_dark());
        assert_eq!(*themes.presented(Appearance::Light), my_light());
        assert!(
            !mentions(&themes, "is unavailable"),
            "{:?}",
            themes.notice_list()
        );
    }

    ///
    /// An identity no available Theme answers to, and one whose Theme is of
    /// the other appearance — the `okabe-ito` every earlier build saved into
    /// the light key — both present that appearance's default built-in, are
    /// reported, and neither is rewritten.
    ///
    #[test]
    fn an_unavailable_or_wrong_appearance_selection_falls_back_and_is_kept() {
        let selection = ThemeSelection::new(id("missing"), OKABE_ITO_IDENTITY);
        let themes = SelectedThemes::new(ThemeRegistry::built_in(), selection.clone());
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert_eq!(*themes.presented(Appearance::Light), orcvs_light());
        assert_eq!(*themes.selection(), selection);
        assert!(mentions(&themes, "selected dark Theme \"missing\""));
        assert!(mentions(&themes, "selected light Theme \"okabe-ito\""));
        assert!(mentions(&themes, "it is a dark Theme"));
        assert!(mentions(&themes, "Orcvs Light is shown in its place"));
    }

    #[test]
    fn selecting_a_theme_changes_only_its_own_appearance() {
        let mut themes = SelectedThemes::new(with_my_themes(), ThemeSelection::default());

        themes.select(Appearance::Dark, &id("my-dark"));
        assert_eq!(*themes.presented(Appearance::Dark), my_dark());
        assert_eq!(*themes.presented(Appearance::Light), orcvs_light());
        assert_eq!(
            *themes.selection().identity(Appearance::Dark),
            id("my-dark")
        );

        themes.select(Appearance::Light, &id("my-light"));
        assert_eq!(*themes.presented(Appearance::Dark), my_dark());
        assert_eq!(*themes.presented(Appearance::Light), my_light());
        assert_eq!(
            *themes.selection().identity(Appearance::Light),
            id("my-light")
        );

        themes.select(Appearance::Dark, &OKABE_ITO_IDENTITY);
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert_eq!(*themes.presented(Appearance::Light), my_light());
    }

    ///
    /// Picking a Theme is the viewer's choice, not a fallback: picking the
    /// built-in a kept, unresolvable selection falls back to replaces that
    /// selection, even though what is presented does not change, and its
    /// notice goes. The other appearance's notice stays: nothing about it
    /// changed.
    ///
    #[test]
    fn picking_the_fallback_theme_replaces_a_kept_selection() {
        let mut themes = SelectedThemes::new(
            ThemeRegistry::built_in(),
            ThemeSelection::new(id("missing"), OKABE_ITO_IDENTITY),
        );

        themes.select(Appearance::Dark, &OKABE_ITO_IDENTITY);
        assert!(!mentions(&themes, "\"missing\""));
        assert!(mentions(&themes, "\"okabe-ito\""));

        themes.select(Appearance::Light, &ORCVS_LIGHT_IDENTITY);
        assert_eq!(*themes.selection(), ThemeSelection::default());
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert_eq!(*themes.presented(Appearance::Light), orcvs_light());
        assert_eq!(themes.notice_count(), 0, "{:?}", themes.notice_list());
    }

    ///
    /// A dismissed selection notice stays dismissed while the viewer picks a
    /// Theme for the other appearance.
    ///
    #[test]
    fn picking_one_appearance_does_not_restore_the_others_dismissed_notice() {
        let mut themes = SelectedThemes::new(
            with_my_themes(),
            ThemeSelection::new(id("missing"), ORCVS_LIGHT_IDENTITY),
        );
        assert!(mentions(&themes, "\"missing\""));
        themes.dismiss_notices();

        themes.select(Appearance::Light, &id("my-light"));
        assert_eq!(themes.notice_count(), 0, "{:?}", themes.notice_list());
    }

    ///
    /// A web import of the selected Theme presents it, and clears the notice
    /// about it; a failed import of it changes neither.
    ///
    #[test]
    fn importing_the_selected_theme_presents_it_and_clears_its_notice() {
        let mut themes = SelectedThemes::new(
            ThemeRegistry::built_in(),
            ThemeSelection::new(id("my-dark"), ORCVS_LIGHT_IDENTITY),
        );
        assert!(mentions(&themes, "\"my-dark\" is unavailable"));
        assert!(
            mentions(&themes, "the Theme is shown once its file loads"),
            "{:?}",
            themes.notice_list()
        );

        themes
            .import("my-dark.toml", b"not a document")
            .expect_err("an invalid document");
        assert!(mentions(&themes, "\"my-dark\" is unavailable"));
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());

        themes
            .import("my-dark.toml", dark("Mine").as_bytes())
            .expect("a valid document");
        assert!(
            !mentions(&themes, "is unavailable"),
            "a stale unavailable notice survived the import that fixed it: {:?}",
            themes.notice_list()
        );
        assert_eq!(themes.presented(Appearance::Dark).name, "Mine");
    }

    ///
    /// A dismissed selection notice stays dismissed while the viewer imports
    /// a Theme neither selection names.
    ///
    #[test]
    fn importing_an_unrelated_theme_does_not_restore_a_dismissed_notice() {
        let mut themes = SelectedThemes::new(
            ThemeRegistry::built_in(),
            ThemeSelection::new(id("missing"), ORCVS_LIGHT_IDENTITY),
        );
        assert!(mentions(&themes, "\"missing\""));
        themes.dismiss_notices();

        themes
            .import("other.toml", dark("Other").as_bytes())
            .expect("a valid document");
        assert_eq!(themes.notice_count(), 0, "{:?}", themes.notice_list());
    }

    ///
    /// `schema.md`: a valid reimport that changes the selected Theme's
    /// appearance falls back to that appearance's default built-in, is
    /// reported, and the saved selection is kept.
    ///
    #[test]
    fn a_reimport_that_changes_appearance_falls_back_and_keeps_the_selection() {
        let mut themes = SelectedThemes::new(
            ThemeRegistry::built_in(),
            ThemeSelection::new(id("mine"), ORCVS_LIGHT_IDENTITY),
        );
        themes
            .import("mine.toml", dark("Mine").as_bytes())
            .expect("a valid document");
        assert_eq!(themes.presented(Appearance::Dark).name, "Mine");

        let light = "format = \"orcvs-theme\"\nversion = 1\nname = \"Mine\"\n\
                     inherits = \"orcvs-light\"\n";
        themes
            .import("mine.toml", light.as_bytes())
            .expect("a valid document");
        assert_eq!(*themes.presented(Appearance::Dark), okabe_ito());
        assert_eq!(*themes.presented(Appearance::Light), orcvs_light());
        assert_eq!(*themes.selection().identity(Appearance::Dark), id("mine"));
        assert!(
            mentions(&themes, "selected dark Theme \"mine\""),
            "{:?}",
            themes.notice_list()
        );
    }
}
