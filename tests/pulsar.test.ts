import {jest} from '@jest/globals'

import { pulsar, matcher, isTime, timeToFrame, isFrameTime } from '../src/pulsar';

import { lerp } from '../src/sequence';

require('../src/globals');

let _bpm = 120;

globalThis.bpm = (set?: number) => {
  if (set) {
    _bpm = set;
  }
  return _bpm;
}

describe('pulsar', () => {

  describe('Time Matcher', () => {

    test('timeMatcher from', async () => {

      globalThis.bpm(60);

      // var pulse = pulsar('60', () => {}); // 60s at 60bpm
      let match = matcher('60');

      var result = match(120)
      expect(result).toEqual(false);

      var result = match(240)
      expect(result).toEqual(true);

      var result = match(480)
      expect(result).toEqual(true);
    });

    test('isTime', async () => {
      var result = isTime('1');
      expect(result).toEqual(true);

      var result = isTime('1:1');
      expect(result).toEqual(true);

      var result = isTime(':1');
      expect(result).toEqual(true);

      var result = isTime(':a');
      expect(result).toEqual(false);

      var result = isTime('vtha:99');
      expect(result).toEqual(false);
    });

    test('timeMatcher from to', async () => {

      globalThis.bpm(60);

      // var pulse = pulsar('60:120', () => {}); // 60-120s at 60bpm
      let match = matcher('60:120');

      var result = match(120)
      expect(result).toEqual(false);

      var result = match(240)
      expect(result).toEqual(true);

      var result = match(480)
      expect(result).toEqual(true);

      var result = match(666)
      expect(result).toEqual(false);
    });

    test('timeMatcher with bad from to', async () => {

      globalThis.bpm(60);

      let match = matcher('120:60');

      var result = match(120)
      expect(result).toEqual(false);

      var result = match(240)
      expect(result).toEqual(false);

      var result = match(480)
      expect(result).toEqual(false);

      var result = match(666)
      expect(result).toEqual(false);
    });

    test('timeToFrame', async () => {
      var result = timeToFrame(60, 60); // 60s at 60bpm
      expect(result).toEqual(240);

      var result = timeToFrame(60, 120); // 60s at 120bpm
      expect(result).toEqual(480);

      var result = timeToFrame(90, 120); // 90s at 120bpm
      expect(result).toEqual(720);

      var result = timeToFrame(1, 60); // 90s at 120bpm
      expect(result).toEqual(4);
    });

  });

  describe('Frame Matcher', () => {

    test('frameMatcher from', async () => {
      // var pulse = pulsar('f:100', () => {});
      let match = matcher('f:100')

      var result = match(99)
      expect(result).toEqual(false);

      var result = match(100)
      expect(result).toEqual(true);
    });

    test('isFrameTime', async () => {
      var result = isFrameTime('f:1');
      expect(result).toEqual(true);

      var result = isFrameTime('f:1:1');
      expect(result).toEqual(true);

      var result = isFrameTime('f:1');
      expect(result).toEqual(true);

      var result = isFrameTime('f:');
      expect(result).toEqual(true);

      var result = isFrameTime('f');
      expect(result).toEqual(false);

      var result = isFrameTime(':a');
      expect(result).toEqual(false);

      var result = isFrameTime('vtha:99');
      expect(result).toEqual(false);
    });

    test('frameMatcher from to', async () => {
      // var pulse = pulsar('f:100:200', () => {});
      let match = matcher('f:100:200');

      var result = match(99)
      expect(result).toEqual(false);

      var result = match(101)
      expect(result).toEqual(true);

      var result = match(199)
      expect(result).toEqual(true);

      var result = match(201)
      expect(result).toEqual(false);
    });

    test('frameMatcher with bad from to', async () => {
      // var pulse = pulsar('f:100:60', () => {});
      let match = matcher('f:100:200');

      var result = match(99)
      expect(result).toEqual(false);

      var result = match(240)
      expect(result).toEqual(false);

      var result = match(480)
      expect(result).toEqual(false);

      var result = match(666)
      expect(result).toEqual(false);
    });

  });

  describe('Pattern matcher', () => {

    test('cycles from frame 1', async () => {
      const str = '▮▯▯▯';

      const match = matcher(str);

      expect(match(1)).toBeTruthy();
      expect(match(2)).toBeFalsy();
      expect(match(3)).toBeFalsy();
      expect(match(4)).toBeFalsy();
      expect(match(5)).toBeTruthy();
    });

    test('covers complex patterns', async () => {
      const str = '▮▯▯▯▮';

      const match = matcher(str);

      expect(match(1)).toBeTruthy();
      expect(match(2)).toBeFalsy();
      expect(match(3)).toBeFalsy();
      expect(match(4)).toBeFalsy();
      expect(match(5)).toBeTruthy();
      expect(match(6)).toBeTruthy();
      expect(match(7)).toBeFalsy();
    });

    test('always', async () => {
      const match  = matcher('▮');

      expect(match(1)).toBeTruthy();
      expect(match(2)).toBeTruthy();
      expect(match(4)).toBeTruthy();
    });

    test('always', async () => {
      const match  = matcher('▯');

      expect(match(1)).toBeFalsy();
      expect(match(2)).toBeFalsy();
      expect(match(4)).toBeFalsy();
    });
  });

  describe('bang', () => {

    test('bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();

      const pulse = pulsar('▮', (o) => {
        o.ptn('▮', () => {
          mock();
        });
      });

      pulse.tick(1);

      expect(mock).toHaveBeenCalled();

    });

    test('can have multiple patterns with different shapes', async () => {

      const mock = jest.fn();
      const otherMock = jest.fn();

      const pulse = pulsar('▮', (o) => {
        o.ptn('▮', () => {
          mock();
        });

        o.ptn('▮▯▯', () => {
          otherMock();
        });
      });

      pulse.tick(1);

      expect(mock).toHaveBeenCalled();
      expect(otherMock).toHaveBeenCalled();

    });

    test('generate pattern in ', async () => {

      const mock = jest.fn();
      const otherMock = jest.fn();

      const pulse = pulsar('▮', (o) => {
        const e = euclid(); //[1, 0, 1, 0, 1, 0, 1, 0])
        o.ptn(e, () => {
          mock();
        });

        o.ptn('▮▯▯', () => {
          otherMock();
        });
      });

      pulse.tick(1);
      pulse.tick(2);
      pulse.tick(3);
      pulse.tick(4);

      expect(mock).toBeCalledTimes(2);
      expect(otherMock).toBeCalledTimes(2);
    });

    test('lerpers lerp in a bang', async () => {
      var x = 0;
      var y = 0;
      const lerpX = lerp(1);
      const lerpY = lerp(3);

      const pulse = pulsar('▮', () => {
        x = lerpX();
        y = lerpY()
      })

      pulse.tick(1);
      pulse.tick(2);
      pulse.tick(3);
      pulse.tick(4);

      expect(x).toEqual(1);
      expect(y).toEqual(3);
    });


    test('lerpers created in a bang', async () => {
      var x = 0;
      var y = 0;
      const lerpX = lerp(2);
      const lerpY = lerp(3);

      const pulse = pulsar('▮', () => {
        const lerpX = lerp(2);
        x = lerpX();
      })

      pulse.tick(1);
      pulse.tick(2);
      pulse.tick(3);
      pulse.tick(4);

      expect(x).toEqual(1); // lerper is reset each cycle
    });

    test('bang bang', async () => {
        const mock = jest.fn();
        const innerMock = jest.fn();

        const pulse = pulsar('▮▯▮▯', (o) => {
          mock();

          o.ptn('▯▮▯', () => {
            innerMock();
          })
        })

        pulse.tick(1);
        pulse.tick(2);
        pulse.tick(3);
        pulse.tick(4);
        pulse.tick(5);

        expect(mock).toHaveBeenCalledTimes(3);
        expect(innerMock).toHaveBeenCalledTimes(1);
    });

    test('bang bang bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();
      const innerinnerMock = jest.fn();

      const pulse = pulsar('▮', (o) => {
        // console.log('bang!');
        mock();

        o.ptn('▯▯▮', (o) => {
          // console.log('bang! bang!');
          innerMock();

          o.ptn('▮', () => {
            // console.log('bang! bang! bang!');
            innerinnerMock();
          })
        })
      })

      pulse.tick(1);
      pulse.tick(2);
      pulse.tick(3);
      pulse.tick(4);
      expect(mock).toHaveBeenCalled();
      expect(mock).toHaveBeenCalledTimes(4);
      expect(innerMock).toHaveBeenCalledTimes(1);
      expect(innerinnerMock).toHaveBeenCalledTimes(1);
  });
  });

  describe('at', () => {

    test('at bang at', async () => {

      globalThis.bpm(120);

      const mock = jest.fn();
      const innerMock = jest.fn();

      const pulse = pulsar('1', (o) => {
        mock();

        o.at('2:3', () => {
          innerMock();
        })
      })

      pulse.tick(8); // 8 frames a second
      pulse.tick(16);
      pulse.tick(24);
      pulse.tick(32);
      pulse.tick(40);
      expect(mock).toHaveBeenCalled();
      expect(mock).toHaveBeenCalledTimes(5);
      expect(innerMock).toHaveBeenCalledTimes(2);
  });
  });

  describe('sequence patterns', () => {

    test('euclid', async () => {

      // const e = euclid();

      // ptn(e, () => {

      // });


    });
  });

});


