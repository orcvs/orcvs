#!/usr/bin/env bash
# Opens the console on a Theme, with the Orca Tracker loaded. Press Space to play.
#
#   try-theme.sh theme.toml     # a Theme document file
#   try-theme.sh                # the Theme on the clipboard (the workbench's "Copy TOML"),
#                               # or else the newest one in ~/Downloads ("Download .toml")
#
# Runs in a throwaway home (target/theme-check), so ~/.orcvs is never read or changed.
# Each run replaces the previous one's window and starts the tracker afresh.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(git -C "$here" rev-parse --show-toplevel)"
home="$repo/target/theme-check"

file="${1:-}"
if [ -z "$file" ] && ! pbpaste | grep -q '^format = "orcvs-theme"'; then
  # Nothing copied: take the newest Theme document the workbench downloaded.
  file="$(ls -t "$HOME"/Downloads/*.toml 2>/dev/null | while read -r f; do
    grep -q '^format = "orcvs-theme"' "$f" && { echo "$f"; break; }; done || true)"
  [ -n "$file" ] && echo "Using $file"
fi
if [ -n "$file" ]; then
  document="$(cat "$file")"
  identity="$(basename "$file" .toml | sed -E 's/ \([0-9]+\)$//')"
else
  document="$(pbpaste)"
  identity="$(sed -nE 's/^# ([a-z0-9-]+)\.toml.*/\1/p' <<<"$document" | head -n 1)"
fi
if ! grep -q '^format = "orcvs-theme"' <<<"$document"; then
  echo "Not a Theme document. Pass a .toml file, or use the workbench's Copy TOML first." >&2
  exit 1
fi
identity="${identity:-tried-theme}"
appearance="$(sed -nE 's/^appearance = "(dark|light)"/\1/p' <<<"$document" | head -n 1)"
appearance="${appearance:-dark}"

# The console refuses a Theme whose window.background is not opaque, so force its alpha to FF.
if grep -qE '^"window\.background" = "#[0-9A-Fa-f]{6}[0-9A-Fa-f]{2}"' <<<"$document" \
  && ! grep -qiE '^"window\.background" = "#[0-9A-Fa-f]{6}FF"' <<<"$document"; then
  echo "note: window.background must be opaque; using its colour at full alpha."
  document="$(sed -E 's/^("window\.background" = "#[0-9A-Fa-f]{6})[0-9A-Fa-f]{2}"/\1"/' <<<"$document")"
fi

mkdir -p "$home/.orcvs/themes" "$home/Library/Application Support/Orcvs"
printf '%s\n' "$document" >"$home/.orcvs/themes/$identity.toml"
printf '[theme]\n%s = "%s"\n\n[cursor_effects]\nglitch_amount = 0\n' "$appearance" "$identity" \
  >"$home/.orcvs/config.toml"
cp "$here/tracker-app.ron" "$home/Library/Application Support/Orcvs/app.ron"

# One window at a time: stop the console a previous run started.
if [ -f "$home/console.pid" ] && kill -0 "$(cat "$home/console.pid")" 2>/dev/null; then
  kill "$(cat "$home/console.pid")" || true
  sleep 1
fi

# cargo finds its toolchain through the real HOME; only the console runs in the throwaway one.
cd "$repo"
console="$(cargo build --package console --features inspection --locked --message-format=json-render-diagnostics \
  | sed -n 's/.*"executable":"\([^"]*\)".*/\1/p' | tail -n 1)"
[ -n "$console" ] || { echo "cargo build produced no console executable" >&2; exit 1; }

echo "Opening the console on \"$identity\" ($appearance). The Orca Tracker is loaded: press Space to play."
[ "$appearance" = light ] && echo "It is a light Theme: pick Light (☼) in the top bar if the console shows dark."
EGUI_INSPECTION=127.0.0.1:5719 HOME="$home" XDG_DATA_HOME="$home" "$console" >"$home/console.log" 2>&1 &
echo $! >"$home/console.pid"
echo "Console log: $home/console.log"
