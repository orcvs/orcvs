import {jest} from '@jest/globals'

const path = require("path");

import { Orcvs } from '../src/orcvs';


require('../src/globals');

const source = path.join(__dirname, 'code.orcvs.js');
// const sourceB = `${__dirname}\\code_b.orcvs.test`;

describe('orcvs', () => {
  const orcvs = Orcvs();

  afterAll( async () => {
    jest.restoreAllMocks();
    await orcvs.stop();

    await new Promise((r) => setTimeout(r, 1000));
  })

  test('load & exec', async () => {
    console.log(lerp(1));
    await orcvs.setup();
    await orcvs.setOutput('LoopMidi');

    await orcvs.load(source);
    await new Promise((r) => setTimeout(r, 500));
    // await orcvs.load(source);
    orcvs.start();

    // orcvs.tick(0);
    // await new Promise((r) => setTimeout(r, 500));

    // orcvs.tick(1);
    // await new Promise((r) => setTimeout(r, 500));


    // orcvs.tick(2);p
    // await new Promise((r) => setTimeout(r, 500));

    // await new Promise((r) => setTimeout(r, 300));

    // await orcvs.load(sourceB);

    await new Promise((r) => setTimeout(r, 500));
    // await new Promise((r) => setTimeout(r, 10000));
    // orcvs.telemetry()
    // await new Promise((r) => setTimeout(r, 15000));

    // await orcvs.load(source);

  });

});
