
import {jest} from '@jest/globals'

// PatternImpl, 
import { Orcvs, Pattern, PatternImpl, lerp, cycle, wave, compute} from '../src/orcvs';

// import { WebMidi } from 'webmidi';


describe.only('orcvs', () => {

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
          
      // var lerper = lerp(7, 3);
      // var value = lerper();
      // expect(value).toEqual(7);  

      // for (let i=0; i<=10; i++) {
      //   value = lerper();
      // }

      // expect(value).toEqual(3);   
    });
  
    test('compute', async () => {
      
      const lerper = lerp(2);
  
      var value = compute(lerper);
      expect(value).toEqual(0);     
      
      var value = compute(1);
      expect(value).toEqual(1); 
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
    
    test.only('bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();

      const ptn = PatternImpl('!', (o) => {        
        o.bang('!', () => {
          mock();
        });
      });

      ptn.tick();

      expect(mock).toHaveBeenCalled();

    });

    test('lerpers lerp in a bang', async () => {
      var x = 0;
      var y = 0;
      const lerpX = lerp(1);
      const lerpY = lerp(3);
  
      const ptn = PatternImpl('!', () => {
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

        const ptn = PatternImpl('!', (o̶͎͓̜̮͘r̶̼̀̿͗c̵̡̮̮̀v̶̼͂s̵̞͙̅̒̋ͅ: Pattern) => {
          console.log('bang!');
          mock();
          
          o̶͎͓̜̮͘r̶̼̀̿͗c̵̡̮̮̀v̶̼͂s̵̞͙̅̒̋ͅ.bang('!...', (o̶͎͓̜̮͘r̶̼̀̿͗c̵̡̮̮̀v̶̼͂s̵̞͙̅̒̋ͅ: Pattern) => {
            console.log('bang! bang!');
            innerMock();
          })
        })
        
        ptn.tick(0);
        ptn.tick(1);
        ptn.tick(2);
        expect(mock).toHaveBeenCalled();
        expect(mock).toHaveBeenCalledTimes(3);
        expect(innerMock).toHaveBeenCalledTimes(1);
    });

    test('bang bang bang', async () => {

      const mock = jest.fn();
      const innerMock = jest.fn();
      const innerinnerMock = jest.fn();

      const ptn = PatternImpl('!', (o: Pattern) => {
        console.log('bang!');
        mock();
        
        o.bang('..!', (o: Pattern) => {
          console.log('bang! bang!');
          innerMock();

          o.bang('!', (o: Pattern) => {
            console.log('bang! bang! bang!');
            innerinnerMock();
          })
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


      // // {sysex: true}
      // // WebMidi
      // //   .enable()
      // //   .then(onEnabled)
      // //   .catch((err: any) => console.log(err));
      
      // await WebMidi.enable({sysex: true});
      // // let output:any = await WebMidi.outputs[0];

      // let output:any =  await WebMidi.getOutputByName("LoopMidi");
      // function onEnabled() {
      //   // ts-expect-error
      //   // let output:any = WebMidi.outputs["LoopMidi"];
      //   // let output:any = WebMidi.outputs[0];
      //   let channel = output.channels[1];
      //   console.log('sendNoteOn');
      //   channel.sendNoteOn("C3", {attack: 1});
      //   console.log('sendNoteOn done');
      //   setTimeout(() => {
      //     console.log('sendNoteOff');
      //     channel.sendNoteOff("C3");
      //     console.log('sendNoteOff done');
      //   }, 1000);
      // }
      // onEnabled();

      // await new Promise((r) => setTimeout(r, 5000));

  // test('lerp increments and limits', async () => {
  //   var value = 0;

  //   const ptn = Pattern('!', (o: any) => {
  //     console.log('bang');  
  //     value = o.lerp(value, 2);
  //     console.log({value});  
  //   })

  //   ptn.tick(); 
  //   ptn.tick(); 
  //   ptn.tick(); 

  //   expect(value).toEqual(2);     
  // });

  // globalThis.lerp = function(value: number) { 
//   // orcvs.lerp(value);
//   console.log(this)
// }


// function lerp(value: number, tofrom: number, to?: number) {

//     const start = to ? tofrom : 0; // default to zero and 
//     const target = to || tofrom;
    
//     value = clamp(value + 1, start, target);
// }




  //   test('bang every', async () => {
  //       const orcvs = new Orcvs();
        
  //      const mock = jest.fn();

  //       orcvs.bang('!', () => {
  //           console.log('HELLO');
  //           mock();
  //       });
        
  //       orcvs.tick();
  //       expect(mock).toHaveBeenCalled();

  //       orcvs.tick();
  //       expect(mock).toHaveBeenCalledTimes(2);
  //  });
    

  //  test('bang every other', async () => {
  //       const orcvs = new Orcvs();
        
  //       const mock = jest.fn();

  //       orcvs.bang('!.', () => {
  //           mock();
  //       });
        
  //       orcvs.tick();
  //       orcvs.clock.frame = 1;

  //       orcvs.tick();
  //       orcvs.clock.frame = 2;
  //       expect(mock).toHaveBeenCalled();

  //       orcvs.tick();
  //       orcvs.clock.frame = 3;
  //       expect(mock).toHaveBeenCalledTimes(2);
  //   });
 


    // test('bang', async () => {
    //     const orcvs = new Orcvs();
    //     // await orcvs.init();

    //     let channel = orcvs.midi.output?.channels[1];
    //     // channel?.playNote('C3', {attack: 0.5, duration: 500} )
    //     // channel?.sendNoteOn('C3', {attack: 0.5} )
    //     // await new Promise((r) => setTimeout(r, 250));
    //     // channel?.sendNoteOff('C3');
        
    //     // orcvs.bang('!.......', () => {
    //     //     console.log('HELLO');
    //     //     orcvs.play(1, 3, 'C', 0.5, 1);
    //     //     // orcvs.play(1, 2, 'E', 0.5, 0.5);
    //     // });
        
    //     // for (let f = 0; f < 2; f++) {
    //     //     orcvs.run();
    //     //     orcvs.clock.tick();
    //     // }

    //     // await new Promise((r) => setTimeout(r, 15000));
    //     await new Promise((r) => setTimeout(r, 5000));

    //     await orcvs.stop();
    //     await new Promise((r) => setTimeout(r, 5000));
    // });

}); 



// function TodoModel(){
//   var todos = [];
//   var lastChange = null;
      
//   function addToPrivateList(){
//       console.log("addToPrivateList"); 
//   }
//   function add() { console.log("add"); }
//   function reload(){}
  
//   return Object.freeze({
//       add,
//       reload
//   });
// }

// class Orcvs {
//   patterns: Pattern[] = [];

//   bang(pattern: string, fn: any) {
//     this.patterns.push( new Pattern(pattern, fn) );
//   }

//   tick(f: number) {
//     for (const pattern of this.patterns) {
//       pattern.bang(f);
//     }
//   }
// }

// type Context = { [name: string]: any };

// const Context: Context = {
//   lerp: (value: number) => {
//     console.log('lerp');
//     console.log(this);
//   }
// }

// Object.getOwnPropertyNames(this.context).forEach((key) => {
//   if (key !== 'constructor') {
//     this[key] = this[key].bind(this);
//   }
// });


// export class Pattern {
//   ptn: string[]
//   fn: any;
//   context: Context = Object.assign({}, Context);

//   constructor(str: string, fn: any) {
//       this.ptn = [...str]
//       // this.fn = fn; //.bind(Context);

//       this.fn = function() {
//         var lerp = this.context.lerp;
//         this.fn.lerp = this.context.lerp;      
//         fn.call(this);
//       }

//       // this.fn.lerp = this.context.lerp;       

//   }

//   shouldBang(f: number) {     
//     const idx = f % this.ptn.length;
//     return this.ptn[idx] === '!'
//   }

//   bang(f: number) {   
//     // console.log('patterns.bang', { f });
//     // console.log('patterns.bang', this.fn);
//     // console.log('patterns.bang', this.shouldBang(f));
//     // if (this.fn) {  
//       console.log('Pattern.bang', this.fn)
//       // if (this.shouldBang(f)) {  
//         // (() => {
//           // var lerp = this.context.lerp
//           // var lerp = this.context.lerp;
//           // this.fn.call(this.context);
//           this.fn();
//         // })()
//         // this.fn.call(Context);
//       // }      
//     // }    
//   }
// }
// declare global {
//   var orcvs: Orcvs;
//   var bang: any
//   // var lerp: any
// }

// globalThis.orcvs = new Orcvs;
// globalThis.bang = (pattern: string, fn: any) => { 
//   orcvs.bang(pattern, fn);
// }