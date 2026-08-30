import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { readReleaseScope, taggedReference } from '../roadmap.ts';

test('taggedReference rejects an issue without a number', () => {
  const tagged = {
    feature: 'feature',
    issue: { number: null },
  } as Parameters<typeof taggedReference>[0];

  assert.throws(() => taggedReference(tagged), /Tagged issue feature must have a number/u);
});

test('readReleaseScope rejects an executable Definition of Done URL', () => {
  const root = mkdtempSync(join(tmpdir(), 'orcvs-roadmap-'));
  try {
    writeFileSync(
      join(root, 'ROADMAP.md'),
      '# Release\n\nTag: release\nGoal: ship\nDefinition: javascript:alert(1)\n',
    );

    assert.throws(() => readReleaseScope(root), /Definition.*relative path or HTTPS URL/u);
  } finally {
    rmSync(root, { recursive: true });
  }
});
