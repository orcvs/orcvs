
import {jest} from '@jest/globals'


const clamp = (num: number, min: number, max: number) => Math.min(Math.max(num, min), max)

const varToString = (v: any) => Object.keys(v)[0]


function Pattern(str: string, callback: any) {
  var pattern: string[] = [...str]

  var patterns: any[] = []
  
  const lerps: { [name: string]: number } = {}

  function tick(f: number = 0) {
    console.log({pattern});

    // (function(lerp) {
    //   lerp();
    //   callback();
    // })(_lerp);    
    // var lerp = _lerp;
    // callback.lerp = _lerp;
    // @ts-expect-error 
    callback(this);
  }


  function bang(str: string, callback: any) {
    const ptn = Pattern(str, callback);
    patterns.push(ptn)
  }
  // let lerps = []
  // function lerp() {
    
  //   console.log('lerp');
  //   return _lerp;
  // }


  function lerp(variable: number, tofrom: number, to?: number) {

      console.log('lerp');

      const start = to ? tofrom : 0; // default to zero and 
      const target = to || tofrom;
      
      const name = varToString(variable);

      let value = lerps[name] || start;

      value = clamp(value + 1, start, target);

      lerps[name] = value;
      return value;
  }


  // function lerp(value: number, tofrom: number, to?: number) {

  //     const start = to ? tofrom : 0; // default to zero and 
  //     const target = to || tofrom;
      
  //     value = clamp(value + 1, start, target);
  // }

  return {
    tick,
    lerp
  }
}



describe.only('orcvs', () => {
  test('lerp increments and limits', async () => {
    var value = 0;

    const ptn = Pattern('!', (o: any) => {
      console.log('bang');  
      value = o.lerp(value, 2);
      console.log({value});  
    })

    ptn.tick(); 
    ptn.tick(); 
    ptn.tick(); 

    expect(value).toEqual(2);     
  });

  test('lerp increments and limits', async () => {
    var value = 0;

    const ptn = Pattern('!', (o: any) => {
      console.log('bang');  
      value = o.lerp(value, 2);
      console.log({value});  
    })

    ptn.tick(); 
    ptn.tick(); 
    ptn.tick(); 

    expect(value).toEqual(2);     
  });

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