# Themes

The console ships `grayscale-dark` and `grayscale-light` as its default Themes and switches between
them with the light/dark mode. To change a colour, add a custom Theme that inherits a default and
lists only what it changes. [`console/src/theme.md`](../console/src/theme.md) lists every property.

## Changing the Bang colours

`source.bang` is the Bang glyph and `source.bang.border` its Cell border.

`~/.orcvs/themes/my-dark.toml`:

```toml
format = "orcvs-theme"
version = 1
name = "My Dark"
inherits = "grayscale-dark"

[style]
"source.bang" = "#FFAA00"
"source.bang.border" = "#FFAA00"
```

`~/.orcvs/themes/my-light.toml`:

```toml
format = "orcvs-theme"
version = 1
name = "My Light"
inherits = "grayscale-light"

[style]
"source.bang" = "#964B00"
"source.bang.border" = "#964B00"
```

Then select them by file name in `~/.orcvs/config.toml`, and restart the console:

```toml
[theme]
dark = "my-dark"
light = "my-light"
```

Every other property keeps the default's value.
