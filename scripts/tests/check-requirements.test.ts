import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { check, fingerprint, markerLine, slug } from '../check-requirements.ts';

/** The ADRs every fixture holds, both within the marker below. */
const ADRS: Record<string, string> = {
  'docs/adr/0001-a.md': '# A\n\nStatus: accepted on 2026-11-01.\n',
  'docs/adr/0002-b.md': '# B\n\nStatus: accepted.\n\n## Amendment, 2026-11-15: same day\n',
};

/**
 * A topic's marker block naming ADR `through`, date `dated` and the fingerprint
 * of the accepted ADRs in `adrs` (the fixture's own unless given).
 */
function marker(through = '0002', dated = '2026-11-15', adrs: Record<string, string> = ADRS): string {
  const accepted = Object.entries(adrs)
    .filter(([, text]) => !/^Status: (?:proposed|rejected)\b/m.test(text))
    .map(([path, text]) => ({ name: path.slice('docs/adr/'.length), text }));
  return [
    `> Snapshot: incorporates ADRs through ${through} and ADR changes dated through ${dated} (adr ${fingerprint(accepted)}).`,
    '> Source ADRs: 0001.',
    '',
    '',
  ].join('\n');
}

const MARKER = marker();

/** A repository holding `docs/requirements/` with `files`, the ADRs above, and any other `extra` files. */
function repo(files: Record<string, string>, extra: Record<string, string> = {}): string {
  const root = mkdtempSync(join(tmpdir(), 'requirements-'));
  const all = { ...ADRS, ...extra };
  for (const [name, text] of Object.entries(files)) all[`docs/requirements/${name}`] = text;
  for (const [path, text] of Object.entries(all)) {
    mkdirSync(join(root, path, '..'), { recursive: true });
    writeFileSync(join(root, path), text);
  }
  return root;
}

function problems(files: Record<string, string>, extra: Record<string, string> = {}): string[] {
  const root = repo(files, extra);
  try {
    return check(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

test('slug follows GitHub heading anchors', () => {
  assert.equal(slug('LANG-001 A rule, with `code`'), 'lang-001-a-rule-with-code');
});

test('a consistent snapshot has no problems', () => {
  const text = `# T\n\n${MARKER}### LANG-001 A rule\n\nSources: [ADR 0001](../adr/0001-a.md). See [above](#lang-001-a-rule).\n`;
  assert.deepEqual(problems({ 'language.md': text, 'README.md': '# Index\n' }), []);
});

test('a missing marker and a duplicated ID are reported', () => {
  const found = problems({ 'a.md': '### LANG-001 One\n', 'b.md': `${MARKER}### LANG-001 Two\n` });
  assert.ok(found.includes('a.md: no snapshot marker'));
  assert.ok(found.includes('b.md: LANG-001 is also defined in a.md'));
});

test('a cited rule nobody defines is reported, unless an open decision reserves it', () => {
  const text = `${MARKER}See EVAL-002.\n\n## Open decisions\n\nEVAL-003 is reserved.\n`;
  assert.deepEqual(problems({ 'a.md': text }), ['a.md: cites EVAL-002, which no requirement defines']);
});

test('a missing link target or anchor is reported', () => {
  const found = problems({ 'a.md': `${MARKER}[x](gone.md) [y](#nowhere)\n` });
  assert.deepEqual(found, ['a.md: links to missing gone.md', 'a.md: links to missing anchor #nowhere']);
});

test('an ADR a rule cites but the marker omits is reported', () => {
  const found = problems({ 'a.md': `${MARKER}[ADR 0002](../adr/0002-b.md)\n` });
  assert.deepEqual(found, ['a.md: a rule cites ADR 0002, which its Source ADRs do not list']);
  const open = problems({ 'a.md': `${MARKER}\n## Open decisions\n\n[ADR 0002](../adr/0002-b.md)\n` });
  assert.deepEqual(open, []);
});

test('an Evidence file that is missing, or lacks its symbol, is reported', () => {
  const extra = { 'src/lib.rs': 'fn present() {}\n' };
  const text = `${MARKER}Evidence: \`src/lib.rs\` \`present\`, \`absent\`; \`src/gone.rs\`.\n`;
  assert.deepEqual(problems({ 'a.md': text }, extra), [
    'a.md: `absent` is not in the file named before it',
    'a.md: Evidence names missing src/gone.rs',
  ]);
});

test('an ADR numbered or dated past the marker is reported', () => {
  const extra = {
    'docs/adr/0001-a.md': '# A\n\nStatus: accepted on 2026-11-01.\n\n## Amendment, 2026-11-16: later\n',
    'docs/adr/0003-c.md': '# C\n\nStatus: accepted on 2026-11-16.\n',
  };
  const current = marker('0002', '2026-11-15', { ...ADRS, ...extra });
  assert.deepEqual(problems({ 'a.md': current }, extra), [
    'a.md: ADR 0001 is newer than the snapshot marker, which does not incorporate it',
    'a.md: ADR 0003 is newer than the snapshot marker, which does not incorporate it',
  ]);
});

test('a marker dated past the latest ADR change is reported', () => {
  const ahead = MARKER.replace('2026-11-15', '2099-01-01');
  assert.deepEqual(problems({ 'a.md': ahead }), [
    'a.md: the snapshot marker is dated 2099-01-01, after the latest ADR change on 2026-11-15',
  ]);
});

test('a proposed or rejected ADR is not newer than the marker; an accepted one is', () => {
  const proposed = { 'docs/adr/0003-c.md': '# C\n\nStatus: proposed on 2026-11-16.\n' };
  assert.deepEqual(problems({ 'a.md': MARKER }, proposed), []);
  const rejected = { 'docs/adr/0003-c.md': '# C\n\nStatus: rejected on 2026-11-17.\n' };
  assert.deepEqual(problems({ 'a.md': MARKER }, rejected), []);
  const accepted = { 'docs/adr/0003-c.md': '# C\n\nStatus: accepted on 2026-11-17.\n' };
  const current = marker('0002', '2026-11-15', { ...ADRS, ...accepted });
  assert.deepEqual(problems({ 'a.md': current }, accepted), [
    'a.md: ADR 0003 is newer than the snapshot marker, which does not incorporate it',
  ]);
});

test('a marker that incorporates an ADR number no file holds is reported, in a gap or above the highest', () => {
  const above = MARKER.replace('through 0002', 'through 0003');
  assert.deepEqual(problems({ 'a.md': above }), [
    'a.md: the snapshot marker incorporates ADRs through 0003, which no ADR file holds',
  ]);
  // The same marker becomes a gap once ADR 0004 exists above the missing 0003.
  const later = { 'docs/adr/0004-d.md': '# D\n\nStatus: accepted on 2026-11-01.\n' };
  const gap = marker('0003', '2026-11-15', { ...ADRS, ...later });
  assert.deepEqual(problems({ 'a.md': gap }, later), [
    'a.md: the snapshot marker incorporates ADRs through 0003, which no ADR file holds',
    'a.md: ADR 0004 is newer than the snapshot marker, which does not incorporate it',
  ]);
});

test('an amendment heading without a readable date is reported', () => {
  const extra = {
    'docs/adr/0002-b.md': '# B\n\nStatus: accepted.\n\n## Amendment, 2026-11-15: same day\n\n## Amendment: undated\n',
  };
  const current = marker('0002', '2026-11-15', { ...ADRS, ...extra });
  assert.deepEqual(problems({ 'a.md': current }, extra), [
    'docs/adr/0002-b.md: "## Amendment: undated" is not an `## Amendment, YYYY-MM-DD` heading',
  ]);
});

/** The fingerprint problem `check` reports for topic `name` once the ADRs are `adrs`. */
function stale(name: string, adrs: Record<string, string>): string {
  const was = /\(adr ([0-9a-f]{8})\)/.exec(MARKER)![1];
  const now = /\(adr ([0-9a-f]{8})\)/.exec(marker('0002', '2026-11-15', adrs))![1];
  return `${name}: the snapshot marker's fingerprint adr ${was} is not adr ${now}: an accepted ADR changed since it was set; reconcile the topic, then run \`node scripts/check-requirements.ts --marker\``;
}

test('any change to an accepted ADR the marker has not taken in is reported, whatever its date', () => {
  const cases: Record<string, Record<string, string>> = {
    'an amendment dated the marker\'s own day': {
      'docs/adr/0001-a.md': '# A\n\nStatus: accepted on 2026-11-01.\n\n## Amendment, 2026-11-15: also today\n',
    },
    'an edit to the body': { 'docs/adr/0001-a.md': '# A\n\nStatus: accepted on 2026-11-01.\n\nNew text.\n' },
    'an ADR filling a gap, dated before the marker': {
      'docs/adr/0000-z.md': '# Z\n\nStatus: accepted on 2026-11-01.\n',
    },
    'an accepted ADR rejected': { 'docs/adr/0001-a.md': '# A\n\nStatus: rejected on 2026-11-01.\n' },
  };
  for (const [what, extra] of Object.entries(cases)) {
    const found = problems({ 'a.md': MARKER, 'b.md': MARKER }, extra);
    const adrs = { ...ADRS, ...extra };
    assert.ok(found.includes(stale('a.md', adrs)) && found.includes(stale('b.md', adrs)), `${what}: ${found}`);
  }
});

test('a change to a proposed ADR or outside the numbered ADRs owes no marker', () => {
  const extra = {
    'docs/adr/0003-c.md': '# C\n\nStatus: proposed on 2026-11-16.\n',
    'docs/adr/README.md': '# ADRs\n',
  };
  assert.deepEqual(problems({ 'a.md': MARKER }, extra), []);
});

test('--marker prints the line the accepted ADRs call for', () => {
  const root = repo({ 'a.md': MARKER });
  try {
    assert.equal(markerLine(root), MARKER.split('\n')[0]);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
