import {jest} from '@jest/globals'

import { pulsar, matcher, isTimePattern, timeToFrame } from '../src/pulsar';

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

  describe('parameters', () => {
    test('callback is last', async () => {      // @ts-ignore
      expect(() => pulsar(1, 2, 3)).toThrowError();

      expect(() => pulsar(1, 2, ()=>{})).not.toThrowError();
    });
  });

  describe('isTimePattern', () => {

    test('handles time/second patterns', async () => {
      var result = isTimePattern('t:1');
      expect(result).toEqual(true);

      var result = isTimePattern('t:1:1');
      expect(result).toEqual(true);

      var result = isTimePattern('t::1');
      expect(result).toEqual(true);

      var result = isTimePattern('t::a');
      expect(result).toEqual(false);

      var result = isTimePattern('vtha:99');
      expect(result).toEqual(false);
    });

    test('handles frame patterns', async () => {
      var result = isTimePattern('f:1');
      expect(result).toEqual(true);

      var result = isTimePattern('f:1:1');
      expect(result).toEqual(true);

      var result = isTimePattern('f:1');
      expect(result).toEqual(true);

      var result = isTimePattern('f:');
      expect(result).toEqual(true);

      var result = isTimePattern('f');
      expect(result).toEqual(false);

      var result = isTimePattern(':a');
      expect(result).toEqual(false);

      var result = isTimePattern('vtha:99');
      expect(result).toEqual(false);
    });

  });

  describe('matcher', () => {

    describe('time', () => {

      test('matches time from:to', async () => {

        globalThis.bpm(60);

        // var pulse = pulsar('60:120', () => {}); // 60-120s at 60bpm
        let match = matcher('t:60:120');

        var result = match(120)
        expect(result).toEqual(false);

        var result = match(240)
        expect(result).toEqual(true);

        var result = match(480)
        expect(result).toEqual(true);

        var result = match(666)
        expect(result).toEqual(false);
      });
      
      test('wildcard matches forever (MAX_FRAME 999999)', async () => {
        
        globalThis.bpm(60);
        let match = matcher('t:60:*')

        var result = match(100)
        expect(result).toEqual(false);

        var result = match(240)
        expect(result).toEqual(true);
        
        var result = match(575)
        expect(result).toEqual(true);

        var result = match(240 * 10)
        expect(result).toEqual(true);
        
        var result = match(240 * 99)
        expect(result).toEqual(true);
      });

      test('with bad from to', async () => {

        globalThis.bpm(60);

        let match = matcher('t:120:60');

        var result = match(120)
        expect(result).toEqual(false);

        var result = match(240)
        expect(result).toEqual(false);

        var result = match(480)
        expect(result).toEqual(false);

        var result = match(666)
        expect(result).toEqual(false);
      });
    
      test('matches every time seconds', async () => {
        // 60s is 240 at 60bpm
        globalThis.bpm(60);

        let match = matcher('t:60');

        var result = match(120)
        expect(result).toEqual(false);

        var result = match(200)
        expect(result).toEqual(false);

        var result = match(240)
        expect(result).toEqual(true);

        var result = match(480)
        expect(result).toEqual(true);

        var result = match(575)
        expect(result).toEqual(false);
      });

    });

    describe('frame', () => {

      test('every 100', async () => {
        // var pulse = pulsar('f:100', () => {});
        let match = matcher('f:100')

        var result = match(99)
        expect(result).toEqual(false);

        var result = match(100)
        expect(result).toEqual(true);    

        var result = match(199)
        expect(result).toEqual(false);

        var result = match(200)
        expect(result).toEqual(true);
      });

      test('wildcard matches forever (MAX_FRAME 999999)', async () => {
        let match = matcher('f:100:*')

        var result = match(99)
        expect(result).toEqual(false);

        var result = match(100)
        expect(result).toEqual(true);

        var result = match(9999)
        expect(result).toEqual(true);
      });

      test('matches from:to', async () => {
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

      test('with bad from:to', async () => {
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

  });
  
  describe('timeToFrame', () => {

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

  describe('Pattern matcher', () => {

    test('cycles from frame 1', async () => {
      const str = [1,0,0,0];

      const match = matcher(str);

      expect(match(1)).toBeTruthy();
      expect(match(2)).toBeFalsy();
      expect(match(3)).toBeFalsy();
      expect(match(4)).toBeFalsy();
      expect(match(5)).toBeTruthy();
    });

    test('covers complex patterns', async () => {
      const str = [1,0,0,0,1];

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
      const match  = matcher([1]);

      expect(match(1)).toBeTruthy();
      expect(match(2)).toBeTruthy();
      expect(match(4)).toBeTruthy();
    });

    test('always', async () => {
      const match  = matcher([0]);

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

    test('beat as arguments', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();

      const pulse = pulsar(1, 0, 0, 0, (o) => {
        o.ptn(1, () => {
          mock();
        });
      });

      pulse.tick(1);

      expect(mock).toHaveBeenCalled();
    });

    test('beat as integer', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();

      const pulse = pulsar(1000, (o) => {
        o.ptn(1, () => {
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
        mock();

        o.ptn('▯▯▮', (o) => {
          innerMock();

          o.ptn('▮', () => {
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

      const pulse = pulsar('t:1', (o) => {
        mock();

        o.at('t:2:3', () => {
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


