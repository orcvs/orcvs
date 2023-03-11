import {jest} from '@jest/globals'

import { lerp, cycle, wave, seq, compute, midify, random, toBeatArray} from '../src/library';

require('../src/globals');

describe('library', () => {

  describe('helpers', () => {

    test('midify', async () => {

      var value = midify(0);
      expect(value).toEqual(0);

      var value = midify(a);
      expect(value).toEqual(36);

      var value = midify(z);
      expect(value).toEqual(127);
    });

    test('compute', async () => {

      const lerper = lerp(2);

      var value = compute(lerper);
      expect(value).toEqual(1);

      var value = compute(1);
      expect(value).toEqual(1);
    });
  });

  describe('toBeatArray', () => {

    test('converts ▮▯', async () => {
      const result = toBeatArray('▮▯');
      expect(result).toEqual([1, 0]);
    });

    test('converts numeric string', async () => {
      const result = toBeatArray('10101010');
      expect(result).toEqual([1, 0, 1, 0, 1, 0, 1, 0]);
    });

  });

  describe('seq', () => {

    test('accepts rest params ', async () => {
      const ary = ['!', '.', '.', '!', '.'];
      let s = seq(...ary);

      for (let i = 0; i < 5; i++) {
        let result = s();
        expect(result).toEqual(ary[i]);
      }
    });

    // test('accepts array ', async () => {
    //   const ary = ['!', '.', '.', '!', '.'];
    //   let s = seq(ary);

    //   for (let i = 0; i < 5; i++) {
    //     let result = s();
    //     expect(result).toEqual(ary[i]);
    //   }
    // });

    test('accepts params', async () => {
      const ary = [A, B, C, D, E];
      let s = seq(A, B, C, D, E);

      for (let i = 0; i < 5; i++) {
        let result = s();
        expect(result).toEqual(ary[i]);
      }
    });

    test('numbers', async () => {

      const ary = [A, B, C, D, E];
      const s = seq(...ary);

      for (let i = 0; i < 5; i++) {
        const result = s();
        expect(result).toEqual(ary[i]);
      }
      for (let i = 0; i < 5; i++) {
        const result = s();
        expect(result).toEqual(ary[i]);
      }

    });

    test('str', async () => {
      const ary = ['!', '.', '.', '!', '.'];
      const s = seq(...ary);

      for (let i = 0; i < 5; i++) {
        let result = s();
        expect(result).toEqual(ary[i]);
      }
      for (let i = 0; i < 5; i++) {
        let result = s();
        expect(result).toEqual(ary[i]);
      }
    });

    test('multiple sequences', async () => {
      const num = [A, B, C, D, E ];
      const str = ['!', '.', '.', '!', '.'];

      const numSeq = seq(A, B, C, D, E );
      const strSeq = seq('!', '.', '.', '!', '.');

      for (let i = 0; i < 5; i++) {
        let n = numSeq();
        let s = strSeq();
        expect(n).toEqual(num[i]);
        expect(s).toEqual(str[i]);
      }

      for (let i = 0; i < 5; i++) {
        let n = numSeq();
        expect(n).toEqual(num[i]);

        if (i === 3) {
          let s = strSeq();
          expect(s).toEqual('!');
        }
      }
    });

  });

  describe('lerp', () => {

    test('starts at zero or from', async () => {

      var lerper = lerp(5);
      var value = lerper();
      expect(value).toEqual(1);

      var lerper = lerp(5, 7);
      var value = lerper();
      expect(value).toEqual(5);
    });

    test('increments and limits', async () => {
      var lerper = lerp(5);
      var value = lerper();

      for (let i=0; i<=10; i++) {
        value = lerper();
      }
      expect(value).toEqual(5);

      var value = 0;
      var lerper = lerp(3, 7);

      for (let i=0; i<=10; i++) {
        value = lerper();
      }

      expect(value).toEqual(7);
    });

    test('lerp decrements and limits', async () => {
      var value = 5;
      var lerper = lerp(5, 0);

      for (let i=0; i<=10; i++) {
        value = lerper();
      }

      expect(value).toEqual(0);

      var lerper = lerp(7, 3);
      var value = lerper();
      expect(value).toEqual(7);

      for (let i=0; i<=10; i++) {
        value = lerper();
      }

      expect(value).toEqual(3);
    });

    test('multiple lerps', async () => {
      let lerpA = lerp(5);
      let lerpB = lerp(10);

      for (let i=1; i<=5; i++) {
        const vA = lerpA();
        expect(vA).toEqual(i);
        const vB = lerpB();
        expect(vB).toEqual(i)
      }

      lerpA = lerp(5);
      lerpB = lerp(10);

      for (let i=1; i<=10; i++) {
        const vA = lerpA();

        if (i <= 5) {
          expect(vA).toEqual(i);
        } else {
          expect(vA).toEqual(5);
        }

        const vB = lerpB();
        expect(vB).toEqual(i)
      }


    });

  });

  describe('cycle', () => {
    test('cycle', async () => {
      const cycler = cycle(1);

      var value = cycler();
      expect(value).toEqual(0);

      var value = cycler();
      expect(value).toEqual(1);

      var value = cycler();
      expect(value).toEqual(0);
    });

    test('cycle', async () => {
      var value = 0;
      const cycler = cycle(3);

      for (let i=0; i <= 3; i++) {
        value = cycler();
        expect(value).toEqual(i);
      }

      // Cycle repeats
      for (let i=0; i <= 3; i++) {
        value = cycler();
        expect(value).toEqual(i);
      }
    });

    test('cycle within range', async () => {
      var value = 0;
      const cycler = cycle(3,7);

      for (let i=3; i <= 7; i++) {
        value = cycler();
        expect(value).toEqual(i);
      }

      // Cycle repeats
      for (let i=3; i <= 7; i++) {
        value = cycler();
        expect(value).toEqual(i);
      }

    });
  });

  describe('wave', () => {

    test('waves', async () => {
      var value = 0;
      const waver = wave(3);

      for (let i=0; i <= 3; i++) {
        value = waver();
        expect(value).toEqual(i);
      }

      for (let i=2; i >= 0; i--) {
        value = waver();
        expect(value).toEqual(i);
      }

      //Wave continues
      value = waver();
      expect(value).toEqual(1);
    });

    test('wave within positive range', async () => {
      var value = 0;
      const waver = wave(5,9);

      for (let i=5; i <= 9; i++) {
        value = waver();
        expect(value).toEqual(i);
      }

      for (let i=8; i>= 5; i--) {
        value = waver();
        expect(value).toEqual(i);
      }
    });

    test('wave within negative range', async () => {
      var value = 0;
      const waver = wave(7, 3);

      for (let i=7; i >= 3; i--) {
        value = waver();
        expect(value).toEqual(i);
      }

      for (let i=4; i <= 7; i++) {
        value = waver();
        expect(value).toEqual(i);
      }
    });

  });

  describe('random', () => {

    test('starts at 1 by default', async () => {

      var r = random(10);

      var last = 0
      for (let i = 1; i <= 1000; i++) {
        var value = r();
        expect(value).toBeGreaterThanOrEqual(1);
        expect(value).toBeLessThanOrEqual(10);
      }
    });

    test('between min and max', async () => {

      var r = random(10, 15);

      for (let i = 1; i <= 1000; i++) {
        var value = r();
        expect(value).toBeGreaterThanOrEqual(10);
        expect(value).toBeLessThanOrEqual(15);
      }
    });

    test('well distributed', async () => {

      var r = random(1000);

      var s = new Set();

      for (let i = 1; i <= 100; i++) {
        var value = r();
        s.add(value);
      }
      expect(s.size).toBeGreaterThanOrEqual(90);
    });



  });

});


