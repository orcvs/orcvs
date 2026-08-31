import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import { readReleaseScope, taggedReference } from '../roadmap.ts';

test('taggedReference rejects an issue without a number', () => {
  const tagged = {
    feature: 'feature',
    issue: { number: null, path: '.scratch/feature/unnumbered.md' },
  } as Parameters<typeof taggedReference>[0];

  assert.throws(
    () => taggedReference(tagged),
    /Tagged issue \.scratch\/feature\/unnumbered\.md must have a number/u,
  );
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

test('readReleaseScope accepts emphasized release metadata', () => {
  const root = mkdtempSync(join(tmpdir(), 'orcvs-roadmap-'));
  try {
    writeFileSync(
      join(root, 'ROADMAP.md'),
      '# Release\n\n**Tag:** release\n**Goal:** ship\n**Definition:** docs/done.md\n**Gate:** feature/01\n',
    );

    assert.deepEqual(readReleaseScope(root), {
      title: 'Release',
      tag: 'release',
      goal: 'ship',
      definition: 'docs/done.md',
      gate: 'feature/01',
    });
  } finally {
    rmSync(root, { recursive: true });
  }
});

test('readReleaseScope treats whitespace-only metadata as absent', () => {
  const root = mkdtempSync(join(tmpdir(), 'orcvs-roadmap-'));
  try {
    writeFileSync(
      join(root, 'ROADMAP.md'),
      '# Release\n\nTag: release\nGoal: ship\nGate:   \n',
    );

    assert.equal(readReleaseScope(root)?.gate, null);
  } finally {
    rmSync(root, { recursive: true });
  }
});

test('readReleaseScope accepts markdown-emphasized release labels, same as Status', () => {
  const root = mkdtempSync(join(tmpdir(), 'orcvs-roadmap-'));
  try {
    writeFileSync(
      join(root, 'ROADMAP.md'),
      '# Release\n\n**Tag:** release\n**Goal:** ship\n**Definition:** docs/dod.md\n**Gate: 05**\n',
    );

    const scope = readReleaseScope(root);
    assert.equal(scope?.tag, 'release');
    assert.equal(scope?.goal, 'ship');
    assert.equal(scope?.definition, 'docs/dod.md');
    assert.equal(scope?.gate, '05');
  } finally {
    rmSync(root, { recursive: true });
  }
});
