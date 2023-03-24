import {jest} from '@jest/globals'

import { midify, toPulse, flipPulse, pulseOnBeat, merge } from '../src/library';

require('../src/globals');

describe('library', () => {

  describe('toPulse', () => {

    test('converts ▮▯', async () => {
      const result = toPulse('▮▯');
      expect(result).toEqual([1, 0]);
    });

    test('converts numeric string', async () => {
      const result = toPulse('10101010');
      expect(result).toEqual([1, 0, 1, 0, 1, 0, 1, 0]);
    });

    test('converts number', async () => {
      const result = toPulse(10101010);
      expect(result).toEqual([1, 0, 1, 0, 1, 0, 1, 0]);
    });

    test('strips whitespace', async () => {
      const result = toPulse('1 0 1 0 1 0 1 0');
      expect(result).toEqual([1, 0, 1, 0, 1, 0, 1, 0]);
    });

    test('strips punctuation', async () => {
      const result = toPulse('1.0.1.0.1.0.1.0');
      expect(result).toEqual([1, 0, 1, 0, 1, 0, 1, 0]);
    });

    test('allows mixing, even though it makes no sense', async () => {
      const result = toPulse(' ▮, ▯ 1,0. ▮▯');
      expect(result).toEqual([1, 0, 1, 0, 1, 0]);
    });
  });

  describe('flipPulse', () => {

    test('filps ▮▯ string', async () => {
      const result = flipPulse('▮▯');
      expect(result).toEqual([0, 1]);
    });

    test('flips numeric string', async () => {
      const result = flipPulse('10101010');
      expect(result).toEqual([0, 1, 0, 1, 0, 1, 0, 1]);
    });

    test('flips number', async () => {
      const result = flipPulse(10101010);
      expect(result).toEqual([0, 1, 0, 1, 0, 1, 0, 1]);
    });

    test('flips array', async () => {
      const result = flipPulse([0, 1, 0, 1, 0, 1, 0, 1]);
      expect(result).toEqual([1, 0, 1, 0, 1, 0, 1, 0]);
    });
  });

  describe('pulseOnBeat', () => {

    test('beat ', async () => {
      const result = pulseOnBeat();
      expect(result.length).toBe(4);
      expect(result).toEqual([1, 0, 0, 0]);
    });

    test('every 4th beat', async () => {
      const result = pulseOnBeat(4);
      expect(result.length).toBe(16);
      expect(result).toEqual([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    });

  });

  describe('merge', () => {

    test('parameters with default', async () => {
      const result = merge({d: 1, a: 2, r: 3}, {r:4});
      const expected = [{d: 1, a: 2, r: 4}];

      expect(result).toEqual(expected)
    });

    test('populates empty object with default', async () => {
      const result = merge({d: 1, a: 2, r: 3}, {}, {r:4});
      const expected = [{d: 1, a: 2, r: 3}, {d: 1, a: 2, r: 4}];

      expect(result).toEqual(expected)
    });
  });

  test('midify', async () => {

    var value = midify(0);
    expect(value).toEqual(0);

    var value = midify(a);
    expect(value).toEqual(36);

    var value = midify(z);
    expect(value).toEqual(127);
  });



});


