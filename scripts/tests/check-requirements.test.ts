import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { check, slug, unmovedMarkers } from '../check-requirements.ts';

const MARKER = [
  '> Snapshot: incorporates ADRs through 0002 and ADR changes dated through 2026-11-15.',
  '> Source ADRs: 0001.',
  '',
  '',
].join('\n');

/** The ADRs every fixture holds, both within the marker above. */
const ADRS = {
  'docs/adr/0001-a.md': '# A\n\nStatus: accepted on 2026-11-01.\n',
  'docs/adr/0002-b.md': '# B\n\nStatus: accepted.\n\n## Amendment, 2026-11-15: same day\n',
};

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
    'docs/adr/0003-c.md': '# C\n\nStatus: accepted by Toby on 2026-11-16.\n',
  };
  assert.deepEqual(problems({ 'a.md': MARKER }, extra), [
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
  const rejected = { 'docs/adr/0003-c.md': '# C\n\nStatus: rejected by Toby on 2026-11-17.\n' };
  assert.deepEqual(problems({ 'a.md': MARKER }, rejected), []);
  const accepted = { 'docs/adr/0003-c.md': '# C\n\nStatus: accepted by Toby on 2026-11-17.\n' };
  assert.deepEqual(problems({ 'a.md': MARKER }, accepted), [
    'a.md: ADR 0003 is newer than the snapshot marker, which does not incorporate it',
  ]);
});

test('a marker that incorporates an ADR number no file holds is reported, in a gap or above the highest', () => {
  const above = MARKER.replace('through 0002', 'through 0003');
  assert.deepEqual(problems({ 'a.md': above }), [
    'a.md: the snapshot marker incorporates ADRs through 0003, which no ADR file holds',
  ]);
  const gap = MARKER.replace('through 0002', 'through 0003');
  const later = { 'docs/adr/0004-d.md': '# D\n\nStatus: accepted on 2026-11-01.\n' };
  assert.deepEqual(problems({ 'a.md': gap }, later), [
    'a.md: the snapshot marker incorporates ADRs through 0003, which no ADR file holds',
    'a.md: ADR 0004 is newer than the snapshot marker, which does not incorporate it',
  ]);
});

test('an amendment heading without a readable date is reported', () => {
  const extra = {
    'docs/adr/0002-b.md': '# B\n\nStatus: accepted.\n\n## Amendment, 2026-11-15: same day\n\n## Amendment: undated\n',
  };
  assert.deepEqual(problems({ 'a.md': MARKER }, extra), [
    'docs/adr/0002-b.md: "## Amendment: undated" is not an `## Amendment, YYYY-MM-DD` heading',
  ]);
});

test('a change to docs/adr/ that leaves a marker behind the newest ADR is reported', () => {
  const marker = MARKER.split('\n')[0];
  const old = marker.replace('2026-11-15', '2026-11-14');
  const proposed = { 'docs/adr/0003-c.md': '# C\n\nStatus: proposed on 2026-11-16.\n' };
  const root = repo({ 'a.md': MARKER, 'b.md': MARKER.replace('2026-11-15', '2026-11-14') }, proposed);
  try {
    const adr = ['docs/adr/0002-b.md'];
    // a.md moved to the newest ADR; b.md stayed behind it.
    const before = new Map([['a.md', old], ['b.md', old]]);
    assert.deepEqual(unmovedMarkers(root, adr, before), [
      'b.md: the change touches docs/adr/ but leaves this snapshot marker where it was',
    ]);
    // A marker that already names the newest ADR and date has nowhere to move.
    assert.deepEqual(unmovedMarkers(root, adr, new Map([['a.md', marker]])), []);
    // A change outside docs/adr/ owes nothing.
    assert.deepEqual(unmovedMarkers(root, ['docs/requirements/b.md'], before), []);
    // Nor does a change to a proposed ADR, which decides nothing yet.
    assert.deepEqual(unmovedMarkers(root, ['docs/adr/0003-c.md'], before), []);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
