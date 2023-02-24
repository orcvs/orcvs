import {jest} from '@jest/globals'

import { pattern, isTime, timeToFrame} from '../src/pattern';

import { lerp } from '../src/library';

require('../src/globals');

globalThis.bpm = 120;

describe('pattern', () => {

  describe('timeMatcher', () => {

    test('timeMatcher from', async () => {   
      
      globalThis.bpm = 60;

      var ptn = pattern('60', () => {}); // 60s at 60bpm 
      var matcher = ptn.match; 

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
      
      globalThis.bpm = 60;

      var ptn = pattern('60:120', () => {}); // 60-120s at 60bpm
      var matcher = ptn.match; 

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
      
      globalThis.bpm = 60;
      
      var ptn = pattern('120:60', () => {}); // 60-120s at 60bpm
      var matcher = ptn.match;       

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

        const ptn = pattern(str, () => {})

        expect(ptn.match(1)).toBeTruthy();
        expect(ptn.match(2)).toBeFalsy();
        expect(ptn.match(3)).toBeFalsy();
        expect(ptn.match(4)).toBeFalsy();
        expect(ptn.match(5)).toBeTruthy();
      });

      test('covers complex patterns', async () => {
        const str = '▮▯▯▯▮';

        const ptn = pattern(str, () => {})

        expect(ptn.match(1)).toBeTruthy();
        expect(ptn.match(2)).toBeFalsy();
        expect(ptn.match(3)).toBeFalsy();
        expect(ptn.match(4)).toBeFalsy();
        expect(ptn.match(5)).toBeTruthy();
        expect(ptn.match(6)).toBeTruthy();
        expect(ptn.match(7)).toBeFalsy();
      });      

      test('always', async () => {
        var ptn = pattern('▮', (bang) => {})
        expect(ptn.match(1)).toBeTruthy();
        expect(ptn.match(2)).toBeTruthy();
        expect(ptn.match(4)).toBeTruthy();
      });
          
      test('always', async () => {
        var ptn = pattern('▯', (bang) => {})
        expect(ptn.match(1)).toBeFalsy();
        expect(ptn.match(2)).toBeFalsy();
        expect(ptn.match(4)).toBeFalsy();
      });      
    });

    describe('cycle and frame', () => {
  
      test('passes', async () => {
        const str = '▮▯';

        const ptn = pattern(str, (o) => {
          // expect(frame).toEqual(7);
          // expect(cycle).toEqual(3)
        })

        ptn.tick(7);
        
      });

      test('covers complex patterns', async () => {
        const str = '▮▯▯▯▮';

        const ptn = pattern(str, () => {})

        expect(ptn.match(1)).toBeTruthy();
        expect(ptn.match(2)).toBeFalsy();
        expect(ptn.match(3)).toBeFalsy();
        expect(ptn.match(4)).toBeFalsy();
        expect(ptn.match(5)).toBeTruthy();
        expect(ptn.match(6)).toBeTruthy();
        expect(ptn.match(7)).toBeFalsy();
      });      

      test('always', async () => {
        var ptn = pattern('▮', (bang) => {})
        expect(ptn.match(1)).toBeTruthy();
        expect(ptn.match(2)).toBeTruthy();
        expect(ptn.match(42)).toBeTruthy();
      });
          
      test('always', async () => {
        var ptn = pattern('▯', (bang) => {})
        expect(ptn.match(1)).toBeFalsy();
        expect(ptn.match(2)).toBeFalsy();
        expect(ptn.match(42)).toBeFalsy();
      });      
    });

    test('bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();

      const ptn = pattern('▮', (o) => {        
        o.bang('▮', () => {
          mock();
        });
      });

      ptn.tick(1);

      expect(mock).toHaveBeenCalled();

    });

    test('can have multiple patterns with different shapes', async () => {

      const mock = jest.fn();
      const otherMock = jest.fn();

      const ptn = pattern('▮', (o) => {        
        o.bang('▮', () => {
          mock();
        });

        o.bang('▮▯▯', () => {
          otherMock();
        });
      });

      ptn.tick(1);

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
  
      ptn.tick(1); 
      ptn.tick(2); 
      ptn.tick(3); 
      ptn.tick(4); 
  
      expect(x).toEqual(1);   
      expect(y).toEqual(3);    
    });

    
    test('lerpers created in a bang', async () => {
      var x = 0;
      var y = 0;
      const lerpX = lerp(1);
      const lerpY = lerp(3);
  
      const ptn = pattern('▮', () => {
        const lerpX = lerp(2);
        x = lerpX();
      })
  
      ptn.tick(1); 
      ptn.tick(2); 
      ptn.tick(3); 
      ptn.tick(4); 
  
      expect(x).toEqual(0); // lerper is reset each cycle
    });
    
    test('bang bang', async () => {

        const mock = jest.fn();
        const innerMock = jest.fn();

        const ptn = pattern('▮', (o) => {
          mock();
          
          o.bang('▮▯▯▯', () => {
            innerMock();
          })
        })
        
        // bang every 1st and 4th
        ptn.tick(1);
        ptn.tick(2);
        ptn.tick(3);
        ptn.tick(4);

        expect(mock).toHaveBeenCalledTimes(4);
        expect(innerMock).toHaveBeenCalledTimes(1);
    });

    test('bang bang bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();
      const innerinnerMock = jest.fn();

      const ptn = pattern('▮', (o) => {
        // console.log('bang!');
        mock();
    
        o.bang('▯▯▮', (o) => {
          // console.log('bang! bang!');
          innerMock();

          o.bang('▮', () => {
            // console.log('bang! bang! bang!');
            innerinnerMock();
          })
        })
      })

      ptn.tick(1); 
      ptn.tick(2);
      ptn.tick(3);
      ptn.tick(4);
      expect(mock).toHaveBeenCalled();
      expect(mock).toHaveBeenCalledTimes(4);
      expect(innerMock).toHaveBeenCalledTimes(1);
      expect(innerinnerMock).toHaveBeenCalledTimes(1);
  });
  });

    describe('at', () => {

      test('at bang at', async () => {
               
        globalThis.bpm = 120;

        const mock = jest.fn();
        const innerMock = jest.fn();
        
        const ptn = pattern('1', (o) => {
          mock();
          
          o.at('2:3', () => {
            innerMock();           
          })
        })
        
        ptn.tick(8); // 8 frames a second
        ptn.tick(16);
        ptn.tick(24);
        ptn.tick(32);
        ptn.tick(40);
        expect(mock).toHaveBeenCalled();
        expect(mock).toHaveBeenCalledTimes(5);
        expect(innerMock).toHaveBeenCalledTimes(2);
    });
  }); 
}); 


