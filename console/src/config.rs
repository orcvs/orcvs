//!
//! The console's settings, read from `~/.orcvs/config.toml` at startup on
//! native.
//!
//! ```toml
//! [theme]
//! dark = "okabe-ito"      # a dark Theme's identity
//! light = "orcvs-light"   # a light Theme's identity
//!
//! [cursor_effects]
//! glitch_amount = 60      # 0–100
//! glitch_frequency = 55   # 0–100
//! ```
//!
//! Every key is optional, and an absent key is the built-in default. A
//! missing file is an ordinary start. An unreadable or malformed file, an
//! unknown key, a value of the wrong type or out of range is a notice, and
//! only the keys it affects fall back to their defaults. `SelectedThemes`
//! reports a Theme identity no available Theme answers to.
//!
//! There is no settings UI and no reload: the file is read once, before the
//! console is built. The web reads no file and runs on the defaults.
//!

use crate::cursor_effects::CursorEffectSettings;
use crate::theme_selection::ThemeSelection;

///
/// The settings a console starts with, and why any of them is not what the
/// file said.
///
#[derive(Debug, Default)]
pub(crate) struct Config {
    /// The dark and light Theme identities, unchecked against the registry.
    pub(crate) theme_selection: ThemeSelection,
    /// Glitch amount and Glitch frequency, before reduced motion.
    pub(crate) cursor_effects: CursorEffectSettings,
    /// One message per problem, in the order found, for the Theme notice
    /// channel, which reports them.
    pub(crate) notices: Vec<String>,
}

impl Config {
    ///
    /// The shipped console's settings: `~/.orcvs/config.toml`. Without a home
    /// directory it runs on the defaults; `ThemeRegistry::start` reports that.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn start() -> Self {
        std::env::home_dir().map_or_else(Self::default, |home| {
            Self::read(&home.join(".orcvs").join(CONFIG_FILE))
        })
    }

    /// The web reads no config file and shows no config notice.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn start() -> Self {
        Self::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::CONFIG_FILE;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::io::Read as _;
    use std::path::Path;

    use super::Config;
    use crate::theme::{Appearance, ThemeIdentity};
    use crate::theme_selection::ThemeSelection;

    /// The settings file's name in `~/.orcvs/`.
    pub(crate) const CONFIG_FILE: &str = "config.toml";

    /// The most a settings file may hold. A file this size is not settings.
    const MAX_CONFIG_BYTES: u64 = 64 * 1024;

    impl Config {
        ///
        /// The settings the file at `path` holds. A missing file is the
        /// defaults and no notice; every other problem is a notice.
        ///
        pub(crate) fn read(path: &Path) -> Self {
            let origin = path.display().to_string();
            match read_text(path) {
                Ok(Some(text)) => parse(&text, &origin),
                Ok(None) => Self::default(),
                Err(problem) => Self {
                    notices: vec![format!(
                        "Could not read the settings file {origin}: {problem}; using the \
                         default settings"
                    )],
                    ..Self::default()
                },
            }
        }
    }

    ///
    /// The file's text, `None` when there is no file, or why it cannot be
    /// read as settings.
    ///
    fn read_text(path: &Path) -> Result<Option<String>, String> {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.to_string()),
        };
        let mut bytes = Vec::new();
        file.take(MAX_CONFIG_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        if bytes.len() as u64 > MAX_CONFIG_BYTES {
            return Err(format!("it is larger than {MAX_CONFIG_BYTES} bytes"));
        }
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| "it is not UTF-8 text".to_owned())
    }

    ///
    /// The settings `text` holds. `origin` names the file in a notice.
    ///
    fn parse(text: &str, origin: &str) -> Config {
        let mut config = Config::default();
        let table = match toml::from_str::<toml::Table>(text) {
            Ok(table) => table,
            Err(error) => {
                config.notices.push(format!(
                    "The settings file {origin} is not valid TOML, so the default settings are \
                     used: {}",
                    error.to_string().trim_end()
                ));
                return config;
            }
        };
        let mut notice = |message: String| config.notices.push(format!("{origin}: {message}"));
        let mut dark = None;
        let mut light = None;
        let mut amount = None;
        let mut frequency = None;
        for (section, value) in &table {
            let Some(keys) = value.as_table() else {
                if matches!(section.as_str(), "theme" | "cursor_effects") {
                    notice(format!(
                        "`{section}` must be a table; its keys use their defaults"
                    ));
                } else {
                    notice(format!("unknown key `{section}` is ignored"));
                }
                continue;
            };
            for (key, value) in keys {
                let name = format!("{section}.{key}");
                match (section.as_str(), key.as_str()) {
                    ("theme", "dark") => dark = identity(&name, value, &mut notice),
                    ("theme", "light") => light = identity(&name, value, &mut notice),
                    ("cursor_effects", "glitch_amount") => {
                        amount = percentage(&name, value, &mut notice);
                    }
                    ("cursor_effects", "glitch_frequency") => {
                        frequency = percentage(&name, value, &mut notice);
                    }
                    _ => notice(format!("unknown key `{name}` is ignored")),
                }
            }
        }
        config.theme_selection = ThemeSelection::new(
            dark.unwrap_or(ThemeIdentity::default_for(Appearance::Dark)),
            light.unwrap_or(ThemeIdentity::default_for(Appearance::Light)),
        );
        if let Some(amount) = amount {
            *config.cursor_effects.amount_mut() = amount;
        }
        if let Some(frequency) = frequency {
            *config.cursor_effects.frequency_mut() = frequency;
        }
        config
    }

    /// A Theme identity: any non-empty string. Whether a Theme answers to it
    /// is `SelectedThemes`' to report.
    fn identity(
        name: &str,
        value: &toml::Value,
        notice: &mut impl FnMut(String),
    ) -> Option<ThemeIdentity> {
        match value.as_str() {
            Some(identity) if !identity.is_empty() => {
                Some(ThemeIdentity::configured(identity.to_owned()))
            }
            _ => {
                notice(format!(
                    "`{name}` must be a Theme identity, a non-empty string; the default is used"
                ));
                None
            }
        }
    }

    /// A whole number from 0 to 100 inclusive.
    fn percentage(name: &str, value: &toml::Value, notice: &mut impl FnMut(String)) -> Option<u8> {
        let percentage = value
            .as_integer()
            .and_then(|number| u8::try_from(number).ok())
            .filter(|number| *number <= 100);
        if percentage.is_none() {
            notice(format!(
                "`{name}` must be a whole number from 0 to 100; the default is used"
            ));
        }
        percentage
    }

    #[cfg(test)]
    mod tests {
        use super::{CONFIG_FILE, Config};
        use crate::cursor_effects::CursorEffectSettings;
        use crate::theme::{Appearance, ThemeIdentity};
        use crate::theme_registry::tests_support::{TempDir, id};
        use crate::theme_selection::ThemeSelection;

        /// The settings a `config.toml` holding `text` gives, read from a
        /// directory of the test's own, never the viewer's home.
        fn read(text: &str) -> Config {
            let dir = TempDir::new();
            Config::read(&dir.write(CONFIG_FILE, text))
        }

        fn settings(amount: u8, frequency: u8) -> CursorEffectSettings {
            let mut settings = CursorEffectSettings::default();
            *settings.amount_mut() = amount;
            *settings.frequency_mut() = frequency;
            settings
        }

        fn assert_defaults(config: &Config) {
            assert_eq!(config.theme_selection, ThemeSelection::default());
            assert_eq!(config.cursor_effects, CursorEffectSettings::default());
        }

        fn mentions(config: &Config, needle: &str) -> bool {
            config.notices.iter().any(|notice| notice.contains(needle))
        }

        #[test]
        fn a_missing_file_is_the_defaults_and_no_notice() {
            let dir = TempDir::new();
            let config = Config::read(&dir.path().join(CONFIG_FILE));
            assert_defaults(&config);
            assert!(config.notices.is_empty(), "{:?}", config.notices);
        }

        #[test]
        fn an_empty_file_is_the_defaults_and_no_notice() {
            let config = read("");
            assert_defaults(&config);
            assert!(config.notices.is_empty(), "{:?}", config.notices);
        }

        #[test]
        fn every_key_is_read() {
            let config = read(
                "[theme]\ndark = \"my-dark\"\nlight = \"my-light\"\n\n[cursor_effects]\n\
                 glitch_amount = 0\nglitch_frequency = 100\n",
            );
            assert!(config.notices.is_empty(), "{:?}", config.notices);
            assert_eq!(
                config.theme_selection,
                ThemeSelection::new(id("my-dark"), id("my-light"))
            );
            assert_eq!(config.cursor_effects, settings(0, 100));
        }

        #[test]
        fn an_absent_key_is_its_default() {
            let config =
                read("[theme]\nlight = \"my-light\"\n[cursor_effects]\nglitch_amount = 7\n");
            assert!(config.notices.is_empty(), "{:?}", config.notices);
            assert_eq!(
                *config.theme_selection.identity(Appearance::Dark),
                ThemeIdentity::default_for(Appearance::Dark)
            );
            assert_eq!(
                *config.theme_selection.identity(Appearance::Light),
                id("my-light")
            );
            assert_eq!(
                config.cursor_effects,
                settings(7, CursorEffectSettings::default().frequency())
            );
        }

        #[test]
        fn a_malformed_file_is_the_defaults_and_a_notice_naming_the_file_and_line() {
            let dir = TempDir::new();
            let path = dir.write(CONFIG_FILE, "[theme]\ndark = \"my-dark\"\nlight = \n");
            let config = Config::read(&path);
            assert_defaults(&config);
            assert_eq!(config.notices.len(), 1, "{:?}", config.notices);
            assert!(mentions(&config, &path.display().to_string()));
            assert!(mentions(&config, "line 3"), "{:?}", config.notices);
        }

        #[test]
        fn an_unreadable_or_non_text_file_is_the_defaults_and_a_notice() {
            let dir = TempDir::new();
            std::fs::create_dir(dir.path().join(CONFIG_FILE)).expect("a directory");
            let config = Config::read(&dir.path().join(CONFIG_FILE));
            assert_defaults(&config);
            assert!(mentions(&config, "Could not read the settings file"));

            let other = TempDir::new();
            let path = other.path().join(CONFIG_FILE);
            std::fs::write(&path, [b'#', 0xFF, b'\n']).expect("a test file");
            let config = Config::read(&path);
            assert_defaults(&config);
            assert!(mentions(&config, "not UTF-8"), "{:?}", config.notices);
        }

        #[test]
        fn an_unknown_key_is_a_notice_and_leaves_the_known_keys_applied() {
            let config = read(
                "colour = 1\n[theme]\ndark = \"my-dark\"\naccent = \"x\"\n\
                 [cursor_effects]\nglitch_amount = 9\n[extra]\nkey = 1\n",
            );
            for key in ["`colour`", "`theme.accent`", "`extra.key`"] {
                assert!(
                    mentions(&config, key),
                    "{key} was not reported: {:?}",
                    config.notices
                );
            }
            assert_eq!(config.notices.len(), 3, "{:?}", config.notices);
            assert_eq!(
                *config.theme_selection.identity(Appearance::Dark),
                id("my-dark")
            );
            assert_eq!(config.cursor_effects.amount(), 9);
        }

        #[test]
        fn an_out_of_range_or_mistyped_value_falls_back_alone() {
            let config = read(
                "[theme]\ndark = 3\nlight = \"\"\n\
                 [cursor_effects]\nglitch_amount = 101\nglitch_frequency = -1\n",
            );
            for key in [
                "`theme.dark`",
                "`theme.light`",
                "`cursor_effects.glitch_amount`",
                "`cursor_effects.glitch_frequency`",
            ] {
                assert!(
                    mentions(&config, key),
                    "{key} was not reported: {:?}",
                    config.notices
                );
            }
            assert_defaults(&config);

            let config = read("[cursor_effects]\nglitch_amount = \"x\"\nglitch_frequency = 12\n");
            assert!(mentions(&config, "`cursor_effects.glitch_amount`"));
            assert_eq!(
                config.cursor_effects,
                settings(CursorEffectSettings::default().amount(), 12)
            );

            let config = read("theme = \"okabe-ito\"\n");
            assert!(
                mentions(&config, "`theme` must be a table"),
                "{:?}",
                config.notices
            );
            assert_defaults(&config);
        }
    }
}
