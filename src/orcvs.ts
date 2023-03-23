
import { codify } from './code';
import { Clock } from './clock';
import { Queue } from './queue';
import { Logger } from "./logger";
import { OnPulse, pulsar, PulsarArgs} from './pulsar';
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
  let hasRun: boolean = false;
  let queue = Queue();

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
    return !clock.running || !hasRun && frame % framesPerPhrase() === 0
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
    if (shouldRun(frame)) {
      run();
    }
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
