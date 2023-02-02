
// import {jest} from '@jest/globals'
// import {
//   importFromString,
// } from 'module-from-string'

// import { IOrcvs, Pattern } from '../src/orcvs';



// // class Orcvs {
// //   patterns: Pattern[] = [];

// //   bang(pattern: string, fn: any) {
// //     this.patterns.push( new Pattern(pattern, fn) );
// //   }

// //   tick(f: number) {
// //     for (const pattern of this.patterns) {
// //       pattern.bang(f);
// //     }
// //   }
// // }

// declare global {
//   var orcvs: IOrcvs;
//   var bang: any
// }


// describe.skip('pattern', () => {

//     test('bang', async () => {

//       // bang('!', () => {
//       //   console.log('ptn')
//       // })

//       // globalThis.Orcvs.tick(0);

   
//    });
    

//    test('eval', async () => {

//     const orcvs = new Orcvs();
//     orcvs.clock.setBpm(60);
//     // const inject = `bang('!', () => {
//     //   console.log('bang!')
//     // })`

//     // const code = `
//     // {
//     //   globalThis.bang = (pattern, fn) => { 
//     //     Orcvs.bang(pattern, fn);
//     //   }
//     //   ${inject}
//     // }
//     // `
//     globalThis.orcvs = orcvs;
//     globalThis.bang = (pattern: string, fn: any) => { 
//       orcvs.bang(pattern, fn);
//     }

//     const source = `
//       console.log("vtha!");

//       console.log(orcvs);
//       bang('!...', () => {
//         console.log('bang!')
//       })
//     `
//     const module = `
//         export const run = () => {
//           ${source}
//         }
//     `
//     // console.log(module)
//     const { run } = await importFromString(module, { useCurrentGlobal: true });
    
//     run();
//     console.log(orcvs);
//     await orcvs.run();
//     console.log(orcvs);
//     console.log('--------------------------------------------');
//     // await new Promise((r) => setTimeout(r, 5000));
//     // orcvs.tick(0);
//     // run();

//     // var orcvs = new Orcvs();

//     // const context = {
//     //   Orcvs: orcvs
//     // }

//     // vm.createContext(context); 

//     // // // clock.tick( () => {  
//     // vm.runInContext(code, context);
//     // // });

//     // orcvs.tick(0);


//   });


// }); 
