import {jest} from '@jest/globals'

import { midify, toPulse, flipPulse } from '../src/library';

require('../src/globals');

describe('library', () => {

  describe('toBeatArray', () => {

    test('converts ▮▯', async () => {
      const result = toPulse('▮▯');
      expect(result).toEqual([1, 0]);
    });

    test('converts numeric string', async () => {
      const result = toPulse('10101010');
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


    test('flips array', async () => {
      const result = flipPulse([0, 1, 0, 1, 0, 1, 0, 1]);
      expect(result).toEqual([1, 0, 1, 0, 1, 0, 1, 0]);
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


