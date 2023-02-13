import {jest} from '@jest/globals'

import { Bang, pattern, timeToFrame} from '../src/pattern';

import { lerp } from '../src/library';

require('../src/globals');


describe('pattern', () => {

  
  describe.only('at', () => {

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

    test('at', async () => {      
      const mock = jest.fn();    
      orcvs.at(60, mock);
      orcvs.tick(120);
      expect(mock).not.toBeCalled();
      orcvs.tick(240);
      expect(mock).toBeCalled();
    });
    

  });



  describe('bang', () => {

    describe('shouldBang', () => {

      test('cycles from frame 1', async () => {
        const str = '▮▯▯▯';

        const ptn = pattern(str, () => {})

        expect(ptn.shouldBang(1)).toBeTruthy();
        expect(ptn.shouldBang(2)).toBeFalsy();
        expect(ptn.shouldBang(3)).toBeFalsy();
        expect(ptn.shouldBang(4)).toBeFalsy();
        expect(ptn.shouldBang(5)).toBeTruthy();
      });

      test('covers complex patterns', async () => {
        const str = '▮▯▯▯▮';

        const ptn = pattern(str, () => {})

        expect(ptn.shouldBang(1)).toBeTruthy();
        expect(ptn.shouldBang(2)).toBeFalsy();
        expect(ptn.shouldBang(3)).toBeFalsy();
        expect(ptn.shouldBang(4)).toBeFalsy();
        expect(ptn.shouldBang(5)).toBeTruthy();
        expect(ptn.shouldBang(6)).toBeTruthy();
        expect(ptn.shouldBang(7)).toBeFalsy();
      });      

      test('always', async () => {
        var ptn = pattern('▮', ({ bang }) => {})
        expect(ptn.shouldBang(1)).toBeTruthy();
        expect(ptn.shouldBang(2)).toBeTruthy();
        expect(ptn.shouldBang(42)).toBeTruthy();
      });
          
      test('always', async () => {
        var ptn = pattern('▯', ({ bang }) => {})
        expect(ptn.shouldBang(1)).toBeFalsy();
        expect(ptn.shouldBang(2)).toBeFalsy();
        expect(ptn.shouldBang(42)).toBeFalsy();
      });      
    });

    describe('cycle and frame', () => {

      test('passes', async () => {
        const str = '▮▯';

        const ptn = pattern(str, ({frame, cycle}) => {
          expect(frame).toEqual(7);
          expect(cycle).toEqual(3)
        })

        ptn.tick(7);
        
      });

      test('covers complex patterns', async () => {
        const str = '▮▯▯▯▮';

        const ptn = pattern(str, () => {})

        expect(ptn.shouldBang(1)).toBeTruthy();
        expect(ptn.shouldBang(2)).toBeFalsy();
        expect(ptn.shouldBang(3)).toBeFalsy();
        expect(ptn.shouldBang(4)).toBeFalsy();
        expect(ptn.shouldBang(5)).toBeTruthy();
        expect(ptn.shouldBang(6)).toBeTruthy();
        expect(ptn.shouldBang(7)).toBeFalsy();
      });      

      test('always', async () => {
        var ptn = pattern('▮', ({ bang }) => {})
        expect(ptn.shouldBang(1)).toBeTruthy();
        expect(ptn.shouldBang(2)).toBeTruthy();
        expect(ptn.shouldBang(42)).toBeTruthy();
      });
          
      test('always', async () => {
        var ptn = pattern('▯', ({ bang }) => {})
        expect(ptn.shouldBang(1)).toBeFalsy();
        expect(ptn.shouldBang(2)).toBeFalsy();
        expect(ptn.shouldBang(42)).toBeFalsy();
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

      ptn.tick(1);

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
  
      const ptn = pattern('▮', ({ bang }) => {
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

        const ptn = pattern('▮', ({ bang }) => {
          mock();
          
          bang('▮▯▯▯', () => {
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

}); 


