import {jest} from '@jest/globals'

import { euclid, rotate } from '../src/algo';

require('../src/globals');

describe('algo', () => {

  describe('euclid', () => {

    test('generates default', async () => {

      const result = euclid();
      expect(result).toEqual([1, 0, 1, 0, 1, 0, 1, 0]);
    });

    test('generates & rotates', async () => {

      const result = euclid(16, 9, 1);
      expect(result).toEqual([ 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 1 ]);
    });

  });

  describe('rotate', () => {

    test('1', async () => {
      const ary = [1, 2, 3, 4, 5];
      const result = rotate(ary, 1);

      expect(result).toEqual([2, 3, 4, 5, 1]);
    });

    test('-1', async () => {
      const ary = [1, 2, 3, 4, 5];
      const result = rotate(ary, -1);

      expect(result).toEqual([5, 1, 2, 3, 4]);
    });

    test('5', async () => {
      const ary = [1, 2, 3, 4, 5];
      const result = rotate(ary, 5);

      expect(result).toEqual([1, 2, 3, 4, 5]);
    });
  });
});


