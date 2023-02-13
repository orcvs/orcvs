import {jest} from '@jest/globals'

import { Orcvs } from '../src/orcvs';

require('../src/globals');

const source = `${__dirname}\\code.orcvs.test`;

describe('orcvs', () => {
  const orcvs = Orcvs();

  afterAll(() => {
    orcvs.stop();
  })
  test.skip('load & exec', async () => {

    await orcvs.setup();
    await orcvs.setOutput('LoopMidi');  

    await orcvs.load(source);
    // await new Promise((r) => setTimeout(r, 500));

    orcvs.start();
    // orcvs.tick(0);
    // await new Promise((r) => setTimeout(r, 500));

    // orcvs.tick(1);
    // await new Promise((r) => setTimeout(r, 500));

    // orcvs.tick(2);
    // await new Promise((r) => setTimeout(r, 500));

  
    await new Promise((r) => setTimeout(r, 5000));

  });

}); 
