import {jest} from '@jest/globals'


import { hash } from '../src/hash';

import { Bang, pattern, lerp, cycle, wave, compute, midify} from '../src/library';

import { A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z } from '../src/library';

describe('library', () => {

  describe('helpers', () => {

    test('midify', async () => { 
     
      var value = midify(0);
      expect(value).toEqual(0);
      
      var value = midify(A);
      expect(value).toEqual(36);

      var value = midify(Z);
      expect(value).toEqual(127);
    });

    test('compute', async () => {
      
      const lerper = lerp(2);
  
      var value = compute(lerper);
      expect(value).toEqual(0);     
      
      var value = compute(1);
      expect(value).toEqual(1); 
    });        
  });


  describe('lerp', () => {

    test('starts at zero or from', async () => {
      var lerper = lerp(5);
      var value = lerper();    
      expect(value).toEqual(0);   

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

      for (let i=2; i>= 0; i--) {
        value = waver();
        expect(value).toEqual(i);    
      }    
    });

    test('wave within range', async () => {
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
  });

  describe('bang', () => {
    
    test('shouldBang', async () => {
      var ptn = pattern('!', (bang) => {})
      expect(ptn.shouldBang(0)).toBeTruthy();
      expect(ptn.shouldBang(1)).toBeTruthy();
      
      var ptn = pattern('!...', (bang) => {})
      expect(ptn.shouldBang(0)).toBeTruthy();
      
      expect(ptn.shouldBang(1)).toBeFalsy();
      expect(ptn.shouldBang(2)).toBeFalsy();

      expect(ptn.shouldBang(3)).toBeFalsy();
      
    });
  
    test('bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();

      const ptn = pattern('!', (bang) => {        
        bang('!', () => {
          mock();
        });
      });

      ptn.tick();

      expect(mock).toHaveBeenCalled();

    });

    test('can have more than one pattern with some shape', async () => {

      const mock = jest.fn();
      const otherMock = jest.fn();

      const ptn = pattern('!', (bang) => {        
        bang('!', () => {
          mock();
        });

        bang('!', () => {
          otherMock();
        });
      });

      ptn.tick();

      expect(mock).toHaveBeenCalled();
      expect(otherMock).toHaveBeenCalled();

    });

    test('lerpers lerp in a bang', async () => {
      var x = 0;
      var y = 0;
      const lerpX = lerp(1);
      const lerpY = lerp(3);
  
      const ptn = pattern('!', () => {
        x = lerpX();
        y = lerpY()
      })
  
      ptn.tick(); 
      ptn.tick(); 
      ptn.tick(); 
      ptn.tick(); 
  
      expect(x).toEqual(1);   
      expect(y).toEqual(3);    
    });
    
    test('bang bang', async () => {

        const mock = jest.fn();
        const innerMock = jest.fn();

        const ptn = pattern('!', (bang) => {
          mock();
          
          bang('!...', () => {
            innerMock();
          })
        })
        
        // bang every 1st and 4th
        ptn.tick(0);
        ptn.tick(1);
        ptn.tick(2);
        ptn.tick(3);

        expect(mock).toHaveBeenCalledTimes(4);
        expect(innerMock).toHaveBeenCalledTimes(1);
    });

    test('bang hash', async () => {

      var ptnA = '!';
      var fnA = (() => {});
      console.log(fnA.length);

      var sA = ptnA + fnA.toString()
      
      var hA = hash(sA);

      var ptnB = '!';
      var fnB = (() => {});
  
      var sB= ptnB + fnB.toString()
      
      var hB= hash(sB);

      expect(hB).toEqual(hB)

      var ptnC = '!.';
      var fnC = (() => {});
  
      var sC = ptnC + fnC.toString()
      
      var hC = hash(sC);

      expect(hC).not.toEqual(hA);

      var ptnD = '!';
      var fnD = ((x: string) => {});
  
      var sD = ptnD + fnD.toString()
      
      var hD = hash(sD);

      expect(hD).not.toEqual(hA);
      expect(hD).not.toEqual(hC);

    });

    test.only('bang bang bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();
      const innerinnerMock = jest.fn();

      const ptn = pattern('!', (bang: Bang) => {
        console.log('bang!');
        mock();
        
        bang('..!', (bang: Bang) => {
          console.log('bang! bang!');
          innerMock();

          // bang('!', (bang: Bang) => {
          //   console.log('bang! bang! bang!');
          //   innerinnerMock();
          // })
        })
      })
      
      ptn.tick(0); 
      ptn.tick(1);
      ptn.tick(2);
      ptn.tick(3);
      expect(mock).toHaveBeenCalled();
      expect(mock).toHaveBeenCalledTimes(4);
      expect(innerMock).toHaveBeenCalledTimes(1);
      expect(innerinnerMock).toHaveBeenCalledTimes(1);
  });
});

}); 


