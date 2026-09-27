//!
//! The console's own end of the storage seam: the two lines that wire the
//! running Console to `console::persistence`. Every other persistence test drives
//! the persistence interface directly and would still pass with the
//! Console unwired, so these construct a real `Console` and call the real
//! `eframe::App::save`.
//!

use eframe::App as _;
use orcvs::source::SourceCommander;

use super::Console;
use super::tests::{assert_chrome, start_console};
use crate::persistence::{InMemoryStorage, SOURCE_KEY, edited_source, save, starting_source};
#[cfg(not(target_arch = "wasm32"))]
use crate::persistence::{IsolatedRonDir, RonFileStorage};
use crate::theme::ThemeIdentity;
use crate::theme::{Appearance, okabe_ito, orcvs_light};
use crate::theme_registry::ThemeRegistry;
use crate::theme_registry::tests_support::{id, my_dark, with_my_themes};
use crate::theme_selection::ThemeSelection;

///
/// A Console started the way eframe starts it, over `storage`.
///
/// `_new_kittest` is eframe's own headless `CreationContext`, which is how
/// an `App` is constructed outside a window; it opens with no storage, and
/// the field is public precisely so a test can supply one.
///
fn console_over(storage: &dyn eframe::Storage) -> Console {
    let mut cc = eframe::CreationContext::_new_kittest(egui::Context::default());
    cc.storage = Some(storage);
    start_console(
        &cc,
        ThemeRegistry::built_in(),
        crate::config::Config::default(),
    )
}

///
/// A stored value that does not decode is discarded: the running Console
/// starts the empty Grid, and its own save writes that Grid over the value.
///
#[tokio::test]
async fn an_undecodable_value_is_discarded_and_overwritten_by_the_next_save() {
    let mut written = InMemoryStorage::default();
    save(&mut written, &edited_source());
    let undecodable = eframe::Storage::get_string(&written, SOURCE_KEY)
        .expect("the save call stored the revision")
        .replace("rows:256", "rows:255");
    let mut storage = InMemoryStorage::default();
    eframe::Storage::set_string(&mut storage, SOURCE_KEY, undecodable.clone());

    let mut console = console_over(&storage);
    assert_eq!(
        console.orcvs.source().snapshot(),
        crate::persistence::default_source().snapshot(),
        "an undecodable value did not start the empty Grid"
    );

    console.save(&mut storage);

    let saved = eframe::Storage::get_string(&storage, SOURCE_KEY).expect("the console saved");
    assert_ne!(saved, undecodable, "the save left the undecodable value");
    assert_eq!(
        starting_source(Some(&storage)).snapshot(),
        crate::persistence::default_source().snapshot()
    );
}

#[tokio::test]
async fn a_console_starts_the_revision_its_creation_storage_holds() {
    let saved = edited_source();
    let mut storage = InMemoryStorage::default();
    save(&mut storage, &saved);

    let console = console_over(&storage);

    assert_eq!(
        console.orcvs.source().snapshot(),
        saved.snapshot(),
        "the Console did not start the revision storage held"
    );
    assert_eq!(console.orcvs.source().grid().count(), 256 * 256);
}

///
/// A configured Theme reference no available Theme answers to presents
/// its appearance's default built-in with a notice, and the configured
/// Theme once a later launch loads it.
///
#[tokio::test]
async fn configured_theme_references_are_presented_or_fall_back() {
    let config = || crate::config::Config {
        theme_selection: ThemeSelection::new(
            id("my-dark"),
            ThemeIdentity::default_for(Appearance::Light),
        ),
        ..crate::config::Config::default()
    };

    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx.clone());
    let console = start_console(&cc, ThemeRegistry::built_in(), config());
    assert_eq!(*console.themes.presented(Appearance::Dark), okabe_ito());
    assert_eq!(*console.themes.presented(Appearance::Light), orcvs_light());
    assert_chrome(
        &ctx,
        &okabe_ito(),
        &orcvs_light(),
        "an unavailable dark reference beside the default light one",
    );
    let notices: Vec<&String> = console.themes.notices().collect();
    assert_eq!(notices.len(), 1, "{notices:?}");
    assert!(
        notices[0].contains("\"my-dark\"") && notices[0].contains("theme.dark"),
        "the unavailable dark selection should be reported: {notices:?}"
    );

    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx.clone());
    let console = start_console(&cc, with_my_themes(), config());
    assert_eq!(
        *console.themes.presented(Appearance::Dark),
        my_dark(),
        "a configured reference did not present the Theme it names once available"
    );
    assert_chrome(
        &ctx,
        &my_dark(),
        &orcvs_light(),
        "the next launch, with the configured dark Theme's file loaded",
    );
    // My Dark's own contrast notice is the only one.
    assert!(
        console
            .themes
            .notices()
            .all(|notice| notice.contains("contrast floor")),
        "{:?}",
        console.themes.notice_list()
    );
}

///
/// Settings are not read from or written to storage: the keys below
/// change nothing a console presents, and a save writes none of them.
///
#[tokio::test]
async fn the_retired_settings_keys_are_neither_read_nor_written() {
    let mut storage = InMemoryStorage::default();
    for (key, value) in [
        ("dark_theme", "my-dark"),
        ("light_theme", "okabe-ito"),
        ("cursor_effects", "0;0"),
    ] {
        eframe::Storage::set_string(&mut storage, key, value.to_owned());
    }

    let mut cc = eframe::CreationContext::_new_kittest(egui::Context::default());
    cc.storage = Some(&storage);
    let mut console = start_console(&cc, with_my_themes(), crate::config::Config::default());

    assert_eq!(*console.themes.presented(Appearance::Dark), okabe_ito());
    assert_eq!(*console.themes.presented(Appearance::Light), orcvs_light());
    // My Dark's own contrast notice is the only one.
    assert!(
        console
            .themes
            .notices()
            .all(|notice| notice.contains("contrast floor")),
        "{:?}",
        console.themes.notice_list()
    );
    assert_eq!(
        console.cursor_effects,
        crate::cursor_effects::CursorEffectSettings::default()
    );

    let mut written = InMemoryStorage::default();
    console.save(&mut written);
    for key in ["dark_theme", "light_theme", "cursor_effects"] {
        assert_eq!(
            eframe::Storage::get_string(&written, key),
            None,
            "a save wrote the retired {key} key"
        );
    }
}

///
/// `Console::load_function_reference` replaces the whole running Orcvs,
/// Grid included.
///
#[tokio::test]
async fn loading_the_function_reference_replaces_the_source_and_its_grid() {
    let mut storage = InMemoryStorage::default();
    save(&mut storage, &edited_source());
    let mut console = console_over(&storage);
    assert_eq!(
        console.orcvs.source().snapshot(),
        edited_source().snapshot(),
        "the console did not start the revision storage held"
    );

    console.load_function_reference();

    let reference = crate::function_reference::function_reference();
    assert_eq!(
        console.orcvs.source().snapshot(),
        reference.snapshot(),
        "loading the Function reference did not replace the running Source"
    );
    assert_eq!(
        console.orcvs.render_frame().grid().columns(),
        reference.grid().columns()
    );
    assert_eq!(
        console.orcvs.render_frame().grid().rows(),
        reference.grid().rows()
    );

    // With persistence on, the loaded reference then saves like any other
    // Source.
    let mut saved = InMemoryStorage::default();
    console.save(&mut saved);
    assert_eq!(
        starting_source(Some(&saved)).snapshot(),
        reference.snapshot(),
        "the loaded reference was not saved like any other Source"
    );
}

#[tokio::test]
async fn the_console_save_call_stores_the_current_revision() {
    let mut restored_from = InMemoryStorage::default();
    save(&mut restored_from, &edited_source());
    let mut console = console_over(&restored_from);
    let mut storage = InMemoryStorage::default();

    console.save(&mut storage);

    // A Console that never saves leaves storage empty, and the start that
    // reads it opens an empty Grid instead of this revision.
    let next_start = SourceCommander::with_source(starting_source(Some(&storage)));
    assert_eq!(
        next_start.snapshot(),
        edited_source().snapshot(),
        "the next start did not open the revision the Console saved"
    );
}

///
/// Native FileStorage is crate-private, so this drives the same RON kv file
/// the binary writes to `eframe::storage_dir("Orcvs")/app.ron`. Isolate
/// under a unique temp directory; never write Application Support.
///
#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn a_console_save_restarts_from_the_native_ron_file() {
    let dir = IsolatedRonDir::new();
    let saved = edited_source();
    {
        // A Console that already holds the 6×3 revision: FileStorage
        // cannot be constructed, so InMemoryStorage is only how this
        // session is primed. The write under test is `App::save` into the
        // RON file, then `flush`, which is what eframe does after save.
        let mut primed = InMemoryStorage::default();
        save(&mut primed, &saved);
        let mut console = console_over(&primed);
        let mut file = RonFileStorage::create(dir.path());
        console.save(&mut file);
        eframe::Storage::flush(&mut file);
    }

    let storage = RonFileStorage::from_file(dir.path());
    let restored = starting_source(Some(&storage));
    assert_eq!(restored.snapshot(), saved.snapshot());
    assert_eq!(restored.grid().count(), 256 * 256);
    assert!(restored.grid().position(255, 255).is_some());
    assert!(restored.grid().position(256, 255).is_none());

    let console = console_over(&storage);
    assert_eq!(
        console.orcvs.source().snapshot(),
        saved.snapshot(),
        "Console::new did not restore the revision the native RON file held"
    );
}
