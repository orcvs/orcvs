// Reads the four built-in Themes out of console/src/theme.rs (pinned to theme.md by
// the record tests) and injects them as JSON into theme-workbench.html between the
// BUILTINS markers. Usage: node extract.mjs <repo-root>
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repo = process.argv[2] ?? process.cwd();
const src = readFileSync(join(repo, "console/src/theme.rs"), "utf8");
const here = dirname(fileURLToPath(import.meta.url));

// field -> dotted key, and the key's kind, from the style_catalogue! macro.
const catalogue = src.slice(src.indexOf("macro_rules! style_catalogue"), src.indexOf("pub(crate) use style_catalogue"));
const fields = {};
const order = [];
let kind = null;
for (const line of catalogue.split("\n")) {
  const group = line.match(/^\s*(OptionalColor|ColorKey|GridWidthKey|ChromeWidthKey)\s*\{/);
  if (group) kind = { OptionalColor: "fill", ColorKey: "color", GridWidthKey: "grid", ChromeWidthKey: "chrome" }[group[1]];
  const entry = line.match(/^\s*\w+\((\w+)\)\s*=>\s*"([^"]+)"/);
  if (entry) {
    fields[entry[1]] = { key: entry[2], kind };
    order.push({ field: entry[1], key: entry[2], kind });
  }
}

function hex(n) {
  return "#" + n.replace(/_/g, "").toUpperCase();
}

const builtins = [];
for (const fn of ["grayscale_dark", "grayscale_light", "okabe_ito", "orcvs_light"]) {
  const start = src.indexOf(`pub fn ${fn}() -> Theme {`);
  if (start < 0) throw new Error(`no fn ${fn}`);
  const body = src.slice(start, src.indexOf("\n}\n", start));
  const values = {};
  for (const m of body.matchAll(/^\s*(\w+):\s*(.+),$/gm)) {
    const [, field, expr] = m;
    const f = fields[field];
    if (!f) continue;
    let v;
    if (f.kind === "color") v = hex(expr.match(/straight_rgba\(0x([0-9A-Fa-f_]+)\)/)[1]);
    else if (f.kind === "fill") v = expr.trim() === "None" ? "none" : hex(expr.match(/0x([0-9A-Fa-f_]+)/)[1]);
    else v = Number(expr.match(/from_points\(([\d.]+)\)/)[1]);
    values[f.key] = v;
  }
  const missing = order.filter((o) => !(o.key in values)).map((o) => o.key);
  if (missing.length) throw new Error(`${fn} is missing ${missing.join(", ")}`);
  builtins.push({
    identity: fn.replace(/_/g, "-"),
    fn,
    name: body.match(/name:\s*"([^"]+)"/)[1],
    appearance: /Appearance::Dark/.test(body) ? "dark" : "light",
    values,
  });
}

const html = join(here, "theme-workbench.html");
const data = JSON.stringify({ catalogue: order, builtins });
const specimen = JSON.stringify(JSON.parse(readFileSync(join(here, "specimen.json"), "utf8")));
const font = readFileSync(join(repo, "console/assets/MonaspaceNeon-Regular.otf")).toString("base64");
const out = readFileSync(html, "utf8")
  .replace(/\/\*BUILTINS\*\/[\s\S]*?\/\*END BUILTINS\*\//, () => `/*BUILTINS*/${data}/*END BUILTINS*/`)
  .replace(/\/\*SPECIMEN\*\/[\s\S]*?\/\*END SPECIMEN\*\//, () => `/*SPECIMEN*/${specimen}/*END SPECIMEN*/`)
  .replace(/src: url\([^)]*\);/, () => `src: url(data:font/otf;base64,${font});`);
writeFileSync(html, out);
console.log(`injected ${builtins.length} Themes, ${order.length} keys and the specimen into ${html}`);
