import {jest} from '@jest/globals'

import { pulsar, isTime, timeToFrame} from '../src/pulsar';

import { lerp } from '../src/library';

require('../src/globals');

globalThis.bpm(120);

describe('pattern', () => {

  describe('timeMatcher', () => {

    test('timeMatcher from', async () => {

      globalThis.bpm(60);

      var pulse = pulsar('60', () => {}); // 60s at 60bpm
      var matcher = pulse.match;

      var result = matcher(120)
      expect(result).toEqual(false);

      var result = matcher(240)
      expect(result).toEqual(true);

      var result = matcher(480)
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

      var pulse = pulsar('60:120', () => {}); // 60-120s at 60bpm
      var matcher = pulse.match;

      var result = matcher(120)
      expect(result).toEqual(false);

      var result = matcher(240)
      expect(result).toEqual(true);

      var result = matcher(480)
      expect(result).toEqual(true);

      var result = matcher(666)
      expect(result).toEqual(false);
    });

    test('timeMatcher with bad from to', async () => {

      globalThis.bpm(60);

      var pulse = pulsar('120:60', () => {}); // 60-120s at 60bpm
      var matcher = pulse.match;

      var result = matcher(120)
      expect(result).toEqual(false);

      var result = matcher(240)
      expect(result).toEqual(false);

      var result = matcher(480)
      expect(result).toEqual(false);

      var result = matcher(666)
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

  describe('bang', () => {

    describe('matcher', () => {

      test('cycles from frame 1', async () => {
        const str = '▮▯▯▯';

        const pulse = pulsar(str, () => {})

        expect(pulse.match(1)).toBeTruthy();
        expect(pulse.match(2)).toBeFalsy();
        expect(pulse.match(3)).toBeFalsy();
        expect(pulse.match(4)).toBeFalsy();
        expect(pulse.match(5)).toBeTruthy();
      });

      test('covers complex patterns', async () => {
        const str = '▮▯▯▯▮';

        const pulse = pulsar(str, () => {})

        expect(pulse.match(1)).toBeTruthy();
        expect(pulse.match(2)).toBeFalsy();
        expect(pulse.match(3)).toBeFalsy();
        expect(pulse.match(4)).toBeFalsy();
        expect(pulse.match(5)).toBeTruthy();
        expect(pulse.match(6)).toBeTruthy();
        expect(pulse.match(7)).toBeFalsy();
      });

      test('always', async () => {
        var pulse = pulsar('▮', (bang) => {})
        expect(pulse.match(1)).toBeTruthy();
        expect(pulse.match(2)).toBeTruthy();
        expect(pulse.match(4)).toBeTruthy();
      });

      test('always', async () => {
        var pulse = pulsar('▯', (bang) => {})
        expect(pulse.match(1)).toBeFalsy();
        expect(pulse.match(2)).toBeFalsy();
        expect(pulse.match(4)).toBeFalsy();
      });
    });

    describe('cycle and frame', () => {

      test('passes', async () => {
        const str = '▮▯';

        const pulse = pulsar(str, (o) => {
          // expect(frame).toEqual(7);
          // expect(cycle).toEqual(3)
        })

        pulse.tick(7);

      });

      test('covers complex patterns', async () => {
        const str = '▮▯▯▯▮';

        const pulse = pulsar(str, () => {})

        expect(pulse.match(1)).toBeTruthy();
        expect(pulse.match(2)).toBeFalsy();
        expect(pulse.match(3)).toBeFalsy();
        expect(pulse.match(4)).toBeFalsy();
        expect(pulse.match(5)).toBeTruthy();
        expect(pulse.match(6)).toBeTruthy();
        expect(pulse.match(7)).toBeFalsy();
      });

      test('always', async () => {
        var pulse = pulsar('▮', (bang) => {})
        expect(pulse.match(1)).toBeTruthy();
        expect(pulse.match(2)).toBeTruthy();
        expect(pulse.match(42)).toBeTruthy();
      });

      test('always', async () => {
        var pulse = pulsar('▯', (bang) => {})
        expect(pulse.match(1)).toBeFalsy();
        expect(pulse.match(2)).toBeFalsy();
        expect(pulse.match(42)).toBeFalsy();
      });
    });

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

        const pulse = pulsar('▮', (o) => {
          mock();

          o.ptn('▮▯▯▯', () => {
            innerMock();
          })
        })

        // bang every 1st and 4th
        pulse.tick(1);
        pulse.tick(2);
        pulse.tick(3);
        pulse.tick(4);

        expect(mock).toHaveBeenCalledTimes(4);
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
});


