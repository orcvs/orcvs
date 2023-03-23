
import { Code, Runnable } from './code';
import { Clock } from './clock';
import { Queue } from './queue';
import { Logger } from "./logger";
import { OnPulse, pulsar } from './pulsar';
import { Midi } from './midi';
import { dememoize } from './memoize';
import { Eventer } from './eventer';

const logger = Logger.child({
  source: 'Orcvs'
});

export function Orcvs() {
  let clock = Clock(tick)
  let midi = Midi();
  let pulse = pulsar(BANG, (on) => {});
  let queue = Queue();
  let event = Eventer();

  let code: Runnable;

  let hasRun: boolean = false;


  async function setup() {
    logger.info(`Welcome to ${ORCVS}`);
    registerGlobals();
    await midi.setup();
  }

  function registerGlobals() {
    globalThis.bpm = bpm;

    globalThis.output = setOutput;
    globalThis.out = setOutput;

    globalThis.play = midi.play;
    globalThis.ply = midi.play;

    globalThis.send = event.send;
    globalThis.snd = event.send;

    globalThis.listen = event.listen;
    globalThis.lsn = event.listen;
  }

  async function play(filename: string) {
    await load(filename);

    if (!clock.running) {
      start();
    }
  }

  async function load(filename: string) {
    logger.info(`loading ${filename}`);
    code = await Code(filename);
    run();
  }

  function reset() {
    clock.reset();
  }

  function run(frame? :number) {
    if (code?.pending && shouldRun(frame)) {
      logger.info('runnning');
      dememoize();
      // Clear the current pulsar
      pulse = pulsar(BANG, (on) => {});
      code.run(pulse);
    }
  }

  function shouldRun(frame?: number) {
    return !clock.running || frame && frame % framesPerPhrase() === 0
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

  function frame() {
    return clock.frame;
  }

  function tick(frame: number) {
    let startTime = performance.now();``

    run(frame);

    pulse.tick(frame);

    event.tick();
    midi.tick();

    let endTime = performance.now();
    let elapsed = endTime - startTime
    queue.push(elapsed);
  }

  async function telemetry() {
    const average = queue.average();
    const percentile = queue.percentile();
    logger.debug({ tick: `${average}ms Average` });
    logger.debug({ tick: `${percentile}ms 90th percentile` } );
    return {
      average,
      percentile
    }
  }

  async function touch() {
    logger.info('touch');
    clock.touch();
  }

  return {
    bpm,
    frame,
    load,
    play,
    reset,
    setOutput,
    setup,
    start,
    stop,
    telemetry,
    tick,
    touch,
  }
}
