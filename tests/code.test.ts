import {jest} from '@jest/globals'

import { readFileSync } from 'fs';
import { P } from 'pino';
import { Project, ts } from 'ts-morph';
import { codify, sourceFromFile, transform } from '../src/code';

require('../src/globals');

describe('code', () => {

  describe('codify', () => {

    test('wraps source in function', async () => {

      const source = `
        return 'vtha!';
      `
      const code = await codify(source)

      const result = code();

      expect(result).toEqual('vtha!');
    });

    test('loads source from file', async () => {

      const source = `${__dirname}\\code.orcvs.js`;

      const code = await sourceFromFile(source)

      expect(code).toContain('ptn');
    });

   });

   describe('transform', () => {

    test('does not break code', async () => {
      let code = `
          const lerper = lerp(1, 10);
          if (true) {
            play(10, D2({d: 4}));
          }
          for (let i = 0; i<= 10; i++) {
            ptn('▮▯', () => {});
          }
      `;
      const result = await transform(code);
      // console.log(result);

      expect(result).toContain('on.ptn');
      expect(result).toContain('memoize');
      expect(result).toContain('if (true)');
      expect(result).toContain('(let i = 0; i<= 10; i++)');
    });
  });

   describe('memoize', () => {

    test('wraps library calls', async () => {
      let code = `
        const lerper = lerp(1, 10);
        ptn('▮▯▯▯▯▯▯', () => {
          const cycler = cycle(10);
          ptn('▮▯', () => {});
          play(10, D2({d: 4}));
        })
      `;

      const result = await transform(code);
      // console.log(result);
      expect(result).toContain('memoize');
    });
  });

   describe('pulsar injection', () => {

    test('ignores existing property access', async () => {
      let code = `ctx.ptn('▮▯', () => {});`;

      const result = await transform(code);

      expect(result).toEqual(`ctx.ptn('▮▯', on => { });`);
    });

    test('ignores existing property access in nested call', async () => {
      let code = `
        ptn('▮▯▯▯▯▯▯', (ctx) => {
          ctx.ptn('▮▯', () => {});
        })
      `;

      const result = await transform(code);

      expect(result).toContain('on.ptn');
      expect(result).toContain('(ctx)');
      expect(result).toContain(`ctx.ptn('▮▯', on => { });`);
    });


    test('attaches property calls and arrow vars', async () => {

      let code = `
        at('▮▯', () => {
          ptn('▮▯', () => {
            ptn('▮▯', () => {});
            at('▮▯', () => {});
          });
        });
      `;

      const result = await transform(code);

      let expected = `
        on.at('▮▯', on => {
          on.ptn('▮▯', on => {
            on.ptn('▮▯', on => { });
            on.at('▮▯', on => { });
          });
        });
      `;

      expect(clean(result)).toContain(clean(expected));
    });

    test('correctly handles complex nesting', async () => {

      let code = `
        blah.at('▮▯', () => {
          ptn('▮▯', (vtha) => {
            vtha.ptn('▮▯', (on) => {
              if (true) {
                play(10, D2({d: 4}));
              }
            });
            at('▮▯', () => {});
          });
        });
      `;

      const result = await transform(code);

      let expected = `
        blah.at('▮▯', on => {
          on.ptn('▮▯', (vtha) => {
            vtha.ptn('▮▯', (on) => {
              if (true) {
                play(10, D2({d: 4}));
              }
            });
            on.at('▮▯', on => { });
          });
        });
      `;

      expect(clean(result)).toContain(clean(expected));
    });
  });

});


function clean(s: string) {
  return s.replace(/\s/g, '')
}