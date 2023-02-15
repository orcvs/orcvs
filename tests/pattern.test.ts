import {jest} from '@jest/globals'

import { Bang, pattern, timeMatcher, timeToFrame} from '../src/pattern';

import { lerp } from '../src/library';

require('../src/globals');

const bpm = 120;


describe('pattern', () => {

  describe('timeMatcher', () => {

    test('timeMatcher from', async () => {      
      var matcher = timeMatcher('60'); // 60s at 60bpm

      var result = matcher(120, 60)
      expect(result).toEqual(false);

      var result = matcher(240, 60)
      expect(result).toEqual(true);
    
      var result = matcher(480, 60)
      expect(result).toEqual(true);
    });

    test('timeMatcher from to', async () => {      
      var matcher = timeMatcher('60:120'); // 60-120s at 60bpm

      var result = matcher(120, 60)
      expect(result).toEqual(false);

      var result = matcher(240, 60)
      expect(result).toEqual(true);
    
      var result = matcher(480, 60)
      expect(result).toEqual(false);
      
      var result = matcher(666, 60)
      expect(result).toEqual(false);
    });

    test.only('timeMatcher with bad from to', async () => {      
      var matcher = timeMatcher('120:60'); // 60-120s at 60bpm

      var result = matcher(120, 60)
      expect(result).toEqual(false);

      var result = matcher(240, 60)
      expect(result).toEqual(false);
    
      var result = matcher(480, 60)
      expect(result).toEqual(false);
      
      var result = matcher(666, 60)
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

    describe('shouldBang', () => {

      test('cycles from frame 1', async () => {
        const str = '▮▯▯▯';

        const ptn = pattern(str, () => {})

        expect(ptn.match(1, bpm)).toBeTruthy();
        expect(ptn.match(2, bpm)).toBeFalsy();
        expect(ptn.match(3, bpm)).toBeFalsy();
        expect(ptn.match(4, bpm)).toBeFalsy();
        expect(ptn.match(5, bpm)).toBeTruthy();
      });

      test('covers complex patterns', async () => {
        const str = '▮▯▯▯▮';

        const ptn = pattern(str, () => {})

        expect(ptn.match(1, bpm)).toBeTruthy();
        expect(ptn.match(2, bpm)).toBeFalsy();
        expect(ptn.match(3, bpm)).toBeFalsy();
        expect(ptn.match(4, bpm)).toBeFalsy();
        expect(ptn.match(5, bpm)).toBeTruthy();
        expect(ptn.match(6, bpm)).toBeTruthy();
        expect(ptn.match(7, bpm)).toBeFalsy();
      });      

      test('always', async () => {
        var ptn = pattern('▮', ({ bang }) => {})
        expect(ptn.match(1, bpm)).toBeTruthy();
        expect(ptn.match(2, bpm)).toBeTruthy();
        expect(ptn.match(4, bpm)).toBeTruthy();
      });
          
      test('always', async () => {
        var ptn = pattern('▯', ({ bang }) => {})
        expect(ptn.match(1, bpm)).toBeFalsy();
        expect(ptn.match(2, bpm)).toBeFalsy();
        expect(ptn.match(4, bpm)).toBeFalsy();
      });      
    });

    describe('cycle and frame', () => {
  
      test('passes', async () => {
        const str = '▮▯';

        const ptn = pattern(str, ({frame, cycle}) => {
          expect(frame).toEqual(7);
          expect(cycle).toEqual(3)
        })

        ptn.tick(7, bpm);
        
      });

      test('covers complex patterns', async () => {
        const str = '▮▯▯▯▮';

        const ptn = pattern(str, () => {})

        expect(ptn.match(1, bpm)).toBeTruthy();
        expect(ptn.match(2, bpm)).toBeFalsy();
        expect(ptn.match(3, bpm)).toBeFalsy();
        expect(ptn.match(4, bpm)).toBeFalsy();
        expect(ptn.match(5, bpm)).toBeTruthy();
        expect(ptn.match(6, bpm)).toBeTruthy();
        expect(ptn.match(7, bpm)).toBeFalsy();
      });      

      test('always', async () => {
        var ptn = pattern('▮', ({ bang }) => {})
        expect(ptn.match(1, bpm)).toBeTruthy();
        expect(ptn.match(2, bpm)).toBeTruthy();
        expect(ptn.match(42, bpm)).toBeTruthy();
      });
          
      test('always', async () => {
        var ptn = pattern('▯', ({ bang }) => {})
        expect(ptn.match(1, bpm)).toBeFalsy();
        expect(ptn.match(2, bpm)).toBeFalsy();
        expect(ptn.match(42, bpm)).toBeFalsy();
      });      
    });

    test('bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();

      const ptn = pattern('▮', ({ bang }) => {        
        bang('▮', () => {
          mock();
        });
      });

      ptn.tick(1, bpm);

      expect(mock).toHaveBeenCalled();

    });

    test('can have multiple patterns with different shapes', async () => {

      const mock = jest.fn();
      const otherMock = jest.fn();

      const ptn = pattern('▮', ({ bang }) => {        
        bang('▮', () => {
          mock();
        });

        bang('▮▯▯', () => {
          otherMock();
        });
      });

      ptn.tick(1, bpm);

      expect(mock).toHaveBeenCalled();
      expect(otherMock).toHaveBeenCalled();

    });

    test('lerpers lerp in a bang', async () => {
      var x = 0;
      var y = 0;
      const lerpX = lerp(1);
      const lerpY = lerp(3);
  
      const ptn = pattern('▮', () => {
        x = lerpX();
        y = lerpY()
      })
  
      ptn.tick(1, bpm); 
      ptn.tick(2, bpm); 
      ptn.tick(3, bpm); 
      ptn.tick(4, bpm); 
  
      expect(x).toEqual(1);   
      expect(y).toEqual(3);    
    });

    
    test('lerpers created in a bang', async () => {
      var x = 0;
      var y = 0;
      const lerpX = lerp(1);
      const lerpY = lerp(3);
  
      const ptn = pattern('▮', ({ bang }) => {
        const lerpX = lerp(2);
        x = lerpX();
      })
  
      ptn.tick(1, bpm); 
      ptn.tick(2, bpm); 
      ptn.tick(3, bpm); 
      ptn.tick(4, bpm); 
  
      expect(x).toEqual(0); // lerper is reset each cycle
    });
    
    test('bang bang', async () => {

        const mock = jest.fn();
        const innerMock = jest.fn();

        const ptn = pattern('▮', ({ bang }) => {
          mock();
          
          bang('▮▯▯▯', () => {
            innerMock();
          })
        })
        
        // bang every 1st and 4th
        ptn.tick(1, bpm);
        ptn.tick(2, bpm);
        ptn.tick(3, bpm);
        ptn.tick(4, bpm);

        expect(mock).toHaveBeenCalledTimes(4);
        expect(innerMock).toHaveBeenCalledTimes(1);
    });

    test('bang bang bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();
      const innerinnerMock = jest.fn();

      const ptn = pattern('▮', ({ bang }) => {
        // console.log('bang!');
        mock();
        
        bang('▯▯▮', ({ bang }) => {
          // console.log('bang! bang!');
          innerMock();

          bang('▮', () => {
            // console.log('bang! bang! bang!');
            innerinnerMock();
          })
        })
      })
      
      ptn.tick(1, bpm); 
      ptn.tick(2, bpm);
      ptn.tick(3, bpm);
      ptn.tick(4, bpm);
      expect(mock).toHaveBeenCalled();
      expect(mock).toHaveBeenCalledTimes(4);
      expect(innerMock).toHaveBeenCalledTimes(1);
      expect(innerinnerMock).toHaveBeenCalledTimes(1);
  });
});

}); 


