import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

/**
 * Holds named benchmarks to a floor recorded in `benches/floors.toml`.
 *
 * `benchmark-action/github-action-benchmark`, wired into
 * `.github/workflows/bench.yml`, only ever compares a run against the
 * previous point stored for `main`. That makes an accepted regression
 * invisible after it lands: `main` republishes at the worse number, the next
 * pull request is judged against it, and nothing says so again. This answers
 * a different question — "is this benchmark still as good as we agreed it
 * should be" rather than "did this change make it worse" — by comparing the
 * same bencher-format output the ratio gate reads against a ceiling committed
 * in the repository, so raising the ceiling is a reviewed edit rather than a
 * silent side effect of merging. See
 * `.scratch/benchmarks/issues/07-hold-a-named-benchmark-to-a-recorded-floor.md`.
 *
 * This module runs no benchmark itself. It reads `output.txt`, the same file
 * `.github/workflows/bench.yml` produces with `mise run bench | tee output.txt`
 * and hands to the ratio-gate action.
 */

export const DEFAULT_FLOORS_PATH = 'benches/floors.toml';

export interface BenchResult {
  readonly name: string;
  readonly value: number;
  readonly unit: string;
  readonly errorValue: number;
}

export interface BenchFloor {
  readonly name: string;
  readonly maxNs: number;
  readonly runner: string;
  readonly source: string;
}

export interface FloorCheck {
  readonly ok: boolean;
  readonly messages: readonly string[];
}

/**
 * `cargo bench --output-format bencher` — which `mise run bench` always
 * passes — prints one line per benchmark:
 * `test <name> ... bench: <N> ns/iter (+/- <M>)`, and the numbers carry `,`
 * thousands separators (criterion's `format::integer`, `criterion-0.8.2/src/
 * format.rs`). `<name>` is matched as a single non-whitespace token: every
 * benchmark this workspace declares is a bare `bench_function` id or a
 * `group/parameter` id from `BenchmarkId`, neither of which contains a space.
 */
const BENCHER_LINE =
  /^test\s+(\S+)\s+\.\.\.\s+bench:\s+([\d,]+)\s+(\S+)\/iter\s+\(\+\/-\s+([\d,]+)\)\s*$/u;

const parseCount = (digits: string): number => Number(digits.replace(/,/gu, ''));

/**
 * Parses every bencher-format line in `text` into a result keyed by name.
 * Lines that do not match — a criterion progress line, a warning, a blank
 * line — are skipped rather than rejected, the same tolerance the action's
 * own regex-based parser gives the file. A name repeated in the input keeps
 * its last occurrence; `mise run bench` never prints one benchmark twice; a
 * concatenation of two runs is the caller's mistake, not this function's to
 * detect.
 */
export const parseBencherOutput = (text: string): ReadonlyMap<string, BenchResult> => {
  const results = new Map<string, BenchResult>();
  for (const rawLine of text.split('\n')) {
    const matched = BENCHER_LINE.exec(rawLine.trimEnd());
    if (matched === null) continue;
    const [, name, valueText, unit, errorText] = matched;
    if (name === undefined || valueText === undefined || unit === undefined || errorText === undefined) {
      continue;
    }
    results.set(name, {
      name,
      value: parseCount(valueText),
      unit,
      errorValue: parseCount(errorText),
    });
  }
  return results;
};

type TomlValue = string | number;
type TomlRecord = Record<string, TomlValue>;

const FLOOR_TABLE_HEADER = /^\[\[floor\]\]$/u;
const STRING_FIELD = /^([a-z_]+)\s*=\s*"((?:[^"\\]|\\.)*)"$/u;
const NUMBER_FIELD = /^([a-z_]+)\s*=\s*(-?\d+(?:\.\d+)?)$/u;

const unescapeTomlString = (value: string): string =>
  value.replace(/\\(.)/gu, (whole, escaped: string) => {
    switch (escaped) {
      case 'n':
        return '\n';
      case 't':
        return '\t';
      case '"':
        return '"';
      case '\\':
        return '\\';
      default:
        return whole;
    }
  });

/**
 * Strips a `#` comment, honouring a double-quoted string so a `#` inside a
 * `source` field (a PR reference such as `#83`) is not read as one. Escaped
 * quotes (`\"`) do not end the string.
 */
const stripComment = (line: string): string => {
  let inString = false;
  for (let index = 0; index < line.length; index += 1) {
    const char = line[index];
    if (char === '"' && line[index - 1] !== '\\') inString = !inString;
    else if (char === '#' && !inString) return line.slice(0, index);
  }
  return line;
};

const toBenchFloor = (record: TomlRecord, describe: string): BenchFloor => {
  const name = record.name;
  if (typeof name !== 'string' || name === '') {
    throw new Error(`${describe} is missing a string "name"`);
  }
  const maxNs = record.max_ns;
  if (typeof maxNs !== 'number' || !Number.isFinite(maxNs) || maxNs <= 0) {
    throw new Error(`${describe} (name "${name}") is missing a positive number "max_ns"`);
  }
  const runner = record.runner;
  if (typeof runner !== 'string' || runner === '') {
    throw new Error(`${describe} (name "${name}") is missing a string "runner"`);
  }
  const source = record.source;
  if (typeof source !== 'string' || source === '') {
    throw new Error(`${describe} (name "${name}") is missing a string "source"`);
  }
  return { name, maxNs, runner, source };
};

/**
 * Parses the restricted TOML subset `benches/floors.toml` is written in: flat
 * `key = "string"` or `key = 123` pairs inside `[[floor]]` array-of-table
 * blocks, comments, and nothing else. No nested tables, no multi-line
 * strings, no arrays — the shape the file's own header comment documents.
 * Anything wider than that is a parse error rather than a silent partial
 * read, because a floor this function misreads is a guard that does not hold.
 */
export const parseFloorsToml = (text: string, path = DEFAULT_FLOORS_PATH): readonly BenchFloor[] => {
  const records: TomlRecord[] = [];
  let current: TomlRecord | null = null;
  const lines = text.split('\n');
  for (let index = 0; index < lines.length; index += 1) {
    const raw = lines[index] ?? '';
    const line = stripComment(raw).trim();
    if (line === '') continue;
    if (FLOOR_TABLE_HEADER.test(line)) {
      current = {};
      records.push(current);
      continue;
    }
    if (current === null) {
      throw new Error(`${path}:${index + 1}: expected [[floor]] before any field, found: ${raw.trim()}`);
    }
    const stringMatch = STRING_FIELD.exec(line);
    if (stringMatch !== null) {
      const [, key, value] = stringMatch;
      if (key !== undefined && value !== undefined) current[key] = unescapeTomlString(value);
      continue;
    }
    const numberMatch = NUMBER_FIELD.exec(line);
    if (numberMatch !== null) {
      const [, key, value] = numberMatch;
      if (key !== undefined && value !== undefined) current[key] = Number(value);
      continue;
    }
    throw new Error(`${path}:${index + 1}: unrecognised line inside [[floor]]: ${raw.trim()}`);
  }

  const floors = records.map((record, recordIndex) =>
    toBenchFloor(record, `${path}: [[floor]] #${recordIndex + 1}`),
  );

  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const floor of floors) {
    if (seen.has(floor.name)) duplicates.add(floor.name);
    seen.add(floor.name);
  }
  if (duplicates.size > 0) {
    throw new Error(`${path}: duplicate [[floor]] name(s): ${[...duplicates].sort().join(', ')}`);
  }

  return floors;
};

const percentOver = (measured: number, maxNs: number): string =>
  (((measured - maxNs) / maxNs) * 100).toFixed(1);

/**
 * Compares each guarded floor against the run's results. A benchmark with no
 * floor entry is unguarded and never appears here, in either direction. A
 * floor whose name is absent from `results` fails rather than passing
 * silently: a rename that drops a benchmark from the bencher output would
 * otherwise drop its guard with it, unnoticed.
 */
export const checkFloors = (
  floors: readonly BenchFloor[],
  results: ReadonlyMap<string, BenchResult>,
): FloorCheck => {
  const messages: string[] = [];
  for (const floor of floors) {
    const result = results.get(floor.name);
    if (result === undefined) {
      messages.push(
        `${floor.name}: guarded at a ${floor.maxNs} ns floor (${floor.runner}) but absent from the ` +
          'benchmark output — a rename or removal would otherwise drop this guard silently',
      );
      continue;
    }
    if (result.unit !== 'ns') {
      messages.push(
        `${floor.name}: reported in ${result.unit}/iter, not ns/iter — benches/floors.toml only ` +
          'guards nanosecond figures',
      );
      continue;
    }
    if (result.value > floor.maxNs) {
      const over = result.value - floor.maxNs;
      messages.push(
        `${floor.name}: measured ${result.value} ns, exceeds the ${floor.maxNs} ns floor ` +
          `(${floor.runner}) by ${over} ns (${percentOver(result.value, floor.maxNs)}%)`,
      );
    }
  }
  return { ok: messages.length === 0, messages };
};

const entryPoint = process.argv[1];
if (entryPoint !== undefined && import.meta.url === pathToFileURL(resolve(entryPoint)).href) {
  const outputPath = process.argv[2];
  if (outputPath === undefined) {
    console.error(
      'usage: node scripts/check-bench-floors.ts <bencher-output-file> [floors-file]',
    );
    process.exitCode = 1;
  } else {
    const floorsPath = process.argv[3] ?? DEFAULT_FLOORS_PATH;
    try {
      const floors = parseFloorsToml(readFileSync(floorsPath, 'utf8'), floorsPath);
      const results = parseBencherOutput(readFileSync(outputPath, 'utf8'));
      const { ok, messages } = checkFloors(floors, results);
      for (const message of messages) console.error(message);
      if (ok) {
        console.log(`bench floors: ${floors.length} guarded benchmark(s) within their recorded floor`);
      } else {
        process.exitCode = 1;
      }
    } catch (error) {
      console.error(error instanceof Error ? error.message : String(error));
      process.exitCode = 1;
    }
  }
}
