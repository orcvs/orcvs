import {jest} from '@jest/globals'

import { lerp, cycle, wave, seq, compute, midify} from '../src/library';

require('../src/globals');

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

  describe('seq', () => {

    test.only('starts at zero or from', async () => {
      const _p = ['!', '.', '.', '.'];

      // var f = 0;
      // var idx = (_p.length & f);

      for (var i = 1; i<= 9; i++) {
        var f = i - 1; 
        var idx = f % _p.length;
        // console.log({i, f, idx});
        // console.log(_p[idx]);
      }

      // var ary = [A, B, C, D, E ];
      // var s = seq(...ary);
      // console.log(s());
      // console.log(s());
      // console.log(s());
      // console.log(s());
      // console.log(s());
      // console.log(s());
      // console.log(s());
      // console.log(s());
      // console.log(s());
 
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

 
}); 


