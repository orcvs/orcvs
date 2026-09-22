import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import type { BenchFloor } from '../check-bench-floors.ts';
import { checkFloors, parseBencherOutput, parseFloorsToml } from '../check-bench-floors.ts';

// A real fragment of what `cargo bench --output-format bencher` prints, taken
// from the shape criterion 0.8.2's `BencherReport` builds
// (`print!("test {} ... ")` then `println!("bench: {:>11} {}/iter (+/- {})")`
// in `criterion-0.8.2/src/report.rs`).
const SAMPLE_OUTPUT = [
  'test parse ... bench:         104 ns/iter (+/- 7)',
  'test parse_invalid ... bench:          62 ns/iter (+/- 4)',
  'test execute ... bench:          70 ns/iter (+/- 5)',
  'test parse_source ... bench:     1,204 ns/iter (+/- 934)',
  'test parse_records/63 ... bench:     650 ns/iter (+/- 20)',
  '',
].join('\n');

test('parseBencherOutput reads name, value, unit, and error from a bencher line', () => {
  const results = parseBencherOutput('test execute ... bench:          70 ns/iter (+/- 5)\n');

  assert.deepEqual(results.get('execute'), { name: 'execute', value: 70, unit: 'ns', errorValue: 5 });
});

test('parseBencherOutput strips comma thousands separators criterion prints for larger figures', () => {
  const results = parseBencherOutput('test parse_source ... bench:     1,204 ns/iter (+/- 934)\n');

  assert.deepEqual(results.get('parse_source'), {
    name: 'parse_source',
    value: 1204,
    unit: 'ns',
    errorValue: 934,
  });
});

test('parseBencherOutput reads a benchmark_group/parameter id unchanged', () => {
  const results = parseBencherOutput(SAMPLE_OUTPUT);

  assert.equal(results.get('parse_records/63')?.value, 650);
});

test('parseBencherOutput skips lines that are not bencher-format results', () => {
  const results = parseBencherOutput(
    ['Compiling lang v0.1.0', '', 'test parse ... bench:         104 ns/iter (+/- 7)', 'test result: ok'].join(
      '\n',
    ),
  );

  assert.deepEqual([...results.keys()], ['parse']);
});

test('parseBencherOutput keeps the last occurrence of a repeated name', () => {
  const results = parseBencherOutput(
    [
      'test execute ... bench:          70 ns/iter (+/- 5)',
      'test execute ... bench:          90 ns/iter (+/- 5)',
    ].join('\n'),
  );

  assert.equal(results.get('execute')?.value, 90);
});

test('parseBencherOutput parses every line of a real sample end to end', () => {
  const results = parseBencherOutput(SAMPLE_OUTPUT);

  assert.equal(results.size, 5);
  assert.equal(results.get('parse')?.value, 104);
  assert.equal(results.get('parse_invalid')?.value, 62);
});

const FLOORS_FIXTURE = [
  '# leading comment',
  '',
  '[[floor]]',
  'name = "execute"',
  'max_ns = 100',
  '# an inline comment line',
  'runner = "ubuntu-latest (GitHub Actions hosted runner)"',
  'source = "see PR #83"',
  '',
  '[[floor]]',
  'name = "parse_source"',
  'max_ns = 2000',
  'runner = "ubuntu-latest (GitHub Actions hosted runner)"',
  'source = "headroom over the settled figure"',
].join('\n');

test('parseFloorsToml reads every field of a well-formed file', () => {
  const floors = parseFloorsToml(FLOORS_FIXTURE);

  assert.deepEqual(floors, [
    {
      name: 'execute',
      maxNs: 100,
      runner: 'ubuntu-latest (GitHub Actions hosted runner)',
      source: 'see PR #83',
    },
    {
      name: 'parse_source',
      maxNs: 2000,
      runner: 'ubuntu-latest (GitHub Actions hosted runner)',
      source: 'headroom over the settled figure',
    },
  ]);
});

test('parseFloorsToml keeps a "#" inside a quoted string rather than treating it as a comment', () => {
  const floors = parseFloorsToml(FLOORS_FIXTURE);

  assert.equal(floors[0]?.source, 'see PR #83');
});

test('parseFloorsToml rejects a field with no preceding [[floor]] header', () => {
  assert.throws(() => parseFloorsToml('name = "execute"\n'), /expected \[\[floor\]\]/u);
});

test('parseFloorsToml rejects a floor missing a required field', () => {
  assert.throws(
    () => parseFloorsToml('[[floor]]\nname = "execute"\nrunner = "ci"\nsource = "note"\n'),
    /missing a positive number "max_ns"/u,
  );
});

test('parseFloorsToml rejects a non-positive max_ns', () => {
  assert.throws(
    () =>
      parseFloorsToml('[[floor]]\nname = "execute"\nmax_ns = 0\nrunner = "ci"\nsource = "note"\n'),
    /missing a positive number "max_ns"/u,
  );
});

test('parseFloorsToml rejects two floors sharing one name', () => {
  const duplicate = [
    '[[floor]]',
    'name = "execute"',
    'max_ns = 100',
    'runner = "ci"',
    'source = "a"',
    '',
    '[[floor]]',
    'name = "execute"',
    'max_ns = 200',
    'runner = "ci"',
    'source = "b"',
  ].join('\n');

  assert.throws(() => parseFloorsToml(duplicate), /duplicate \[\[floor\]\] name\(s\): execute/u);
});

test('parseFloorsToml rejects a line it cannot parse inside a floor', () => {
  assert.throws(
    () => parseFloorsToml('[[floor]]\nname = "execute"\n[nested]\n'),
    /unrecognised line inside \[\[floor\]\]/u,
  );
});

const floor = (overrides: Partial<BenchFloor> = {}): BenchFloor => ({
  name: 'execute',
  maxNs: 100,
  runner: 'ubuntu-latest (GitHub Actions hosted runner)',
  source: 'note',
  ...overrides,
});

test('checkFloors passes a benchmark strictly under its floor', () => {
  const results = new Map([['execute', { name: 'execute', value: 70, unit: 'ns', errorValue: 5 }]]);

  const check = checkFloors([floor()], results);

  assert.deepEqual(check, { ok: true, messages: [] });
});

test('checkFloors passes a benchmark exactly at its floor', () => {
  const results = new Map([['execute', { name: 'execute', value: 100, unit: 'ns', errorValue: 5 }]]);

  const check = checkFloors([floor()], results);

  assert.deepEqual(check, { ok: true, messages: [] });
});

test('checkFloors fails a benchmark over its floor, naming it, the figure, the result, and the excess', () => {
  const results = new Map([['execute', { name: 'execute', value: 130, unit: 'ns', errorValue: 5 }]]);

  const check = checkFloors([floor()], results);

  assert.equal(check.ok, false);
  assert.equal(check.messages.length, 1);
  assert.match(check.messages[0] ?? '', /^execute: measured 130 ns/u);
  assert.match(check.messages[0] ?? '', /exceeds the 100 ns floor/u);
  assert.match(check.messages[0] ?? '', /ubuntu-latest \(GitHub Actions hosted runner\)/u);
  assert.match(check.messages[0] ?? '', /by 30 ns \(30\.0%\)/u);
});

test('checkFloors ignores a benchmark absent from the floors file, regardless of its value', () => {
  const results = new Map([
    ['execute', { name: 'execute', value: 70, unit: 'ns', errorValue: 5 }],
    ['parse', { name: 'parse', value: 999_999, unit: 'ns', errorValue: 5 }],
  ]);

  const check = checkFloors([floor()], results);

  assert.deepEqual(check, { ok: true, messages: [] });
});

test('checkFloors fails a guarded benchmark missing from the run, so a rename cannot silently drop the guard', () => {
  const check = checkFloors([floor()], new Map());

  assert.equal(check.ok, false);
  assert.match(check.messages[0] ?? '', /^execute: guarded at a 100 ns floor .* but absent from the/u);
});

test('checkFloors fails a guarded benchmark reported in a unit other than ns', () => {
  const results = new Map([['execute', { name: 'execute', value: 1, unit: 'µs', errorValue: 0 }]]);

  const check = checkFloors([floor()], results);

  assert.equal(check.ok, false);
  assert.match(check.messages[0] ?? '', /reported in µs\/iter, not ns\/iter/u);
});

// End-to-end: the script as a CLI, over hand-made bencher output and a
// standalone floors file — never `mise run bench`, which is deferred to CI.
const SCRIPT_PATH = join(import.meta.dirname, '..', 'check-bench-floors.ts');

const runScript = (args: readonly string[]) =>
  execFileSync(process.execPath, [SCRIPT_PATH, ...args], { encoding: 'utf8' });

test('the script exits 0 and reports the guarded count when every floor holds', () => {
  const dir = mkdtempSync(join(tmpdir(), 'orcvs-bench-floors-'));
  try {
    const floorsPath = join(dir, 'floors.toml');
    const outputPath = join(dir, 'output.txt');
    writeFileSync(
      floorsPath,
      '[[floor]]\nname = "execute"\nmax_ns = 100\nrunner = "ci"\nsource = "note"\n',
    );
    writeFileSync(outputPath, 'test execute ... bench:          70 ns/iter (+/- 5)\n');

    const stdout = runScript([outputPath, floorsPath]);

    assert.match(stdout, /bench floors: 1 guarded benchmark\(s\) within their recorded floor/u);
  } finally {
    rmSync(dir, { recursive: true });
  }
});

test('the script exits non-zero and names the excess when a guarded benchmark exceeds its floor', () => {
  const dir = mkdtempSync(join(tmpdir(), 'orcvs-bench-floors-'));
  try {
    const floorsPath = join(dir, 'floors.toml');
    const outputPath = join(dir, 'output.txt');
    writeFileSync(
      floorsPath,
      '[[floor]]\nname = "execute"\nmax_ns = 100\nrunner = "ci"\nsource = "note"\n',
    );
    writeFileSync(outputPath, 'test execute ... bench:          130 ns/iter (+/- 5)\n');

    assert.throws(() => runScript([outputPath, floorsPath]), (error: unknown) => {
      const withOutput = error as { status?: number; stderr?: string };
      assert.equal(withOutput.status, 1);
      assert.match(withOutput.stderr ?? '', /execute: measured 130 ns, exceeds the 100 ns floor/u);
      assert.match(withOutput.stderr ?? '', /by 30 ns \(30\.0%\)/u);
      return true;
    });
  } finally {
    rmSync(dir, { recursive: true });
  }
});

test('the script passes an unguarded benchmark through untouched', () => {
  const dir = mkdtempSync(join(tmpdir(), 'orcvs-bench-floors-'));
  try {
    const floorsPath = join(dir, 'floors.toml');
    const outputPath = join(dir, 'output.txt');
    writeFileSync(
      floorsPath,
      '[[floor]]\nname = "execute"\nmax_ns = 100\nrunner = "ci"\nsource = "note"\n',
    );
    writeFileSync(
      outputPath,
      ['test execute ... bench:          70 ns/iter (+/- 5)', 'test parse ... bench:  999999999 ns/iter (+/- 5)'].join(
        '\n',
      ),
    );

    const stdout = runScript([outputPath, floorsPath]);

    assert.match(stdout, /1 guarded benchmark\(s\)/u);
  } finally {
    rmSync(dir, { recursive: true });
  }
});

test('the script fails when a guarded benchmark is missing from the output file', () => {
  const dir = mkdtempSync(join(tmpdir(), 'orcvs-bench-floors-'));
  try {
    const floorsPath = join(dir, 'floors.toml');
    const outputPath = join(dir, 'output.txt');
    writeFileSync(
      floorsPath,
      '[[floor]]\nname = "execute"\nmax_ns = 100\nrunner = "ci"\nsource = "note"\n',
    );
    writeFileSync(outputPath, 'test parse ... bench:         104 ns/iter (+/- 7)\n');

    assert.throws(() => runScript([outputPath, floorsPath]), (error: unknown) => {
      const withOutput = error as { status?: number; stderr?: string };
      assert.equal(withOutput.status, 1);
      assert.match(withOutput.stderr ?? '', /execute: guarded at a 100 ns floor .* but absent/u);
      return true;
    });
  } finally {
    rmSync(dir, { recursive: true });
  }
});

test('the script prints usage and exits non-zero when given no output file', () => {
  assert.throws(() => runScript([]), (error: unknown) => {
    const withOutput = error as { status?: number; stderr?: string };
    assert.equal(withOutput.status, 1);
    assert.match(withOutput.stderr ?? '', /usage: node scripts\/check-bench-floors\.ts/u);
    return true;
  });
});

test('the script reports a malformed floors file rather than crashing without a message', () => {
  const dir = mkdtempSync(join(tmpdir(), 'orcvs-bench-floors-'));
  try {
    const floorsPath = join(dir, 'floors.toml');
    const outputPath = join(dir, 'output.txt');
    writeFileSync(floorsPath, 'name = "execute"\n');
    writeFileSync(outputPath, 'test execute ... bench:          70 ns/iter (+/- 5)\n');

    assert.throws(() => runScript([outputPath, floorsPath]), (error: unknown) => {
      const withOutput = error as { status?: number; stderr?: string };
      assert.equal(withOutput.status, 1);
      assert.match(withOutput.stderr ?? '', /expected \[\[floor\]\]/u);
      return true;
    });
  } finally {
    rmSync(dir, { recursive: true });
  }
});
