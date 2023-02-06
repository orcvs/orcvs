import {jest} from '@jest/globals'

import { readFileSync } from 'fs';
import { codify} from '../src/code';


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

      const source = `${__dirname}\\code.orcvs.test`;
  
      const code = await codify(source)
      
      const result = code();
      expect(result).toEqual('vtha!');
    });
      
   });

    // test('has access to global context', async () => {
    //   const mock = jest.fn();

    //   const orcvs = {
    //     bang: mock
    //   }; 

    //   registerGlobal(orcvs);

    //   const source = `
    //     bang();
    //   `
    //   const code = await codify(source)

    //   const result = code();
    //   expect(result).toBeFalsy();
      
    //   expect(mock).toBeCalled();
    // });


}); 
