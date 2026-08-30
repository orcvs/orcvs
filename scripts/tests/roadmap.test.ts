import assert from 'node:assert/strict';
import test from 'node:test';

import { taggedReference } from '../roadmap.ts';

test('taggedReference rejects an issue without a number', () => {
  const tagged = {
    feature: 'feature',
    issue: { number: null },
  } as Parameters<typeof taggedReference>[0];

  assert.throws(() => taggedReference(tagged), /Tagged issue feature must have a number/u);
});
