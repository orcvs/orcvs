import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { check, slug } from '../check-requirements.ts';

const MARKER = [
  '> Snapshot: checked against `abc1234` (2026-11-15); incorporates ADRs through 0002.',
  '> Source ADRs: 0001.',
  '',
  '',
].join('\n');

/** A repository holding `docs/requirements/` with `files`, plus any other `extra` files. */
function repo(files: Record<string, string>, extra: Record<string, string> = {}): string {
  const root = mkdtempSync(join(tmpdir(), 'requirements-'));
  const all = { ...extra };
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

const ADR = { 'docs/adr/0001-a.md': '# A\n' };

test('slug follows GitHub heading anchors', () => {
  assert.equal(slug('LANG-001 A rule, with `code`'), 'lang-001-a-rule-with-code');
});

test('a consistent snapshot has no problems', () => {
  const text = `# T\n\n${MARKER}### LANG-001 A rule\n\nSources: [ADR 0001](../adr/0001-a.md). See [above](#lang-001-a-rule).\n`;
  assert.deepEqual(problems({ 'language.md': text, 'README.md': '# Index\n' }, ADR), []);
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
  const extra = { ...ADR, 'docs/adr/0002-b.md': '# B\n' };
  const found = problems({ 'a.md': `${MARKER}[ADR 0002](../adr/0002-b.md)\n` }, extra);
  assert.deepEqual(found, ['a.md: a rule cites ADR 0002, which the marker does not list']);
});

test('an Evidence file that is missing, or lacks its symbol, is reported', () => {
  const extra = { 'src/lib.rs': 'fn present() {}\n' };
  const text = `${MARKER}Evidence: \`src/lib.rs\` \`present\`, \`absent\`; \`src/gone.rs\`.\n`;
  assert.deepEqual(problems({ 'a.md': text }, extra), [
    'a.md: `absent` is not in the file named before it',
    'a.md: Evidence names missing src/gone.rs',
  ]);
});
