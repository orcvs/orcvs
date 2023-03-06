
import { Clock } from './clock';
import { codify } from './code'
import { dememoize } from './library'
import { Logger } from "./logger";
import { Midi } from './midi'

import { OnPulse, pulsar } from './pulsar';

const logger = Logger.child({
  source: 'Orcvs'
});

export function Orcvs() {
  let clock = Clock(tick)
  let midi = Midi();
  let pulse = pulsar(BANG, (on) => {});
  let code: OnPulse;
  let hasRun: boolean = false;

  function ptn(str: string, onPulse: OnPulse): void {
    // logger.debug('BANG!');
    pulse.ptn(str, onPulse);
  }

  async function setup() {
    console.log(`Welcome to ${ORCVS}`);
    registerGlobals();
    await midi.setup();
  }

  function registerGlobals() {
    // globalThis.pattern = ptn;
    // globalThis.ptn = ptn;
    globalThis.bpm = bpm;
    globalThis.output = setOutput;
    globalThis.out = setOutput;
    globalThis.play = midi.play;
    globalThis.ply = midi.play;
  }

  async function play(filename: string) {
    await load(filename);

    if (!clock.running) {
      start();
    }
  }

  async function load(filename: string) {
    logger.info(`loading ${filename}`);
    code = await codify(filename);
    hasRun = false;
    if (!clock.running) {
      run();
    }
  }

  function reset() {
    clock.reset();
  }

  function run() {
    logger.info('run');
    dememoize();
    // Clear the current pulsar
    pulse = pulsar(BANG, (on) => {});
    code(pulse);
    hasRun = true;
  }

  function shouldRun(frame: number) {
    return !clock.running || !hasRun && frame % 8 === 0
  }

  function bpm(bpm?: number) {
    if (bpm) {
      logger.info(`Set BPM: ${bpm}`);
      clock.setBPM(bpm);
    }
    return clock.bpm;
  }

  async function setOutput(out: number | string) {
    logger.info({ setOutput: out });
    midi.selectOutput(out);
  }

  async function start() {
    logger.info('start');
    await clock.start();
  }

  async function stop() {
    logger.info('stop');
    await clock.stop();
    await midi.stop();
  }

  function tick(frame: number) {
    if (shouldRun(frame)) {
      run();
    }
    pulse.tick(frame);
    midi.tick(frame);
  }

  async function touch() {
    logger.info('touch');
    clock.touch();
  }

  return {
    bpm,
    load,
    play,
    reset,
    setOutput,
    setup,
    start,
    stop,
    tick,
    touch
  }
}