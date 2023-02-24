import { Note } from 'webmidi';
import { Logger } from "./logger";

import { Clock } from './clock';
import { Midi } from './midi'
import { codify } from './code'

import { Callback, Pattern, pattern } from './pattern';
import { compute, Computer, Computable, midify} from './library';

import { Chord } from './note';

const logger = Logger.child({
  source: 'Orcvs'
});

export function Orcvs() {
  var clock = Clock(tick)
  var midi = Midi();
  var ptn = pattern(BANG, () => {});  

  var _running
  var _code = () => {};
  var _run: boolean = false;

  function bang(str: string, callback: Callback): void {
    // logger.debug('BANG!');
    ptn.bang(str, callback);
  }

  async function setup() {
    logger.debug('setup');
    registerGlobals();
    await midi.setup();      
  }

  function registerGlobals() {    
    globalThis.bang = bang;    
    globalThis.bpm = clock.bpm;
    globalThis.output = setOutput;
    globalThis.play = midi.play;
  }

  async function load(filename: string) {   
    logger.info(`loading ${filename}`);   
    _code = await codify(filename);    
    _run = false;
    if (!clock.running()) {
      run();
    }
  }

  function reset() {
    clock.reset();
  }

  function run() {  
    logger.info('run');
    
    ptn = pattern(BANG, () => {});  
    _code();
    _run = true;
    
    logger.info('ran');
  }
  
  function shouldRun(frame: number) {
      return !clock.running() || !_run && frame % 8 === 0
  }

  async function setBPM(bpm: number) {
    await clock.setBPM(bpm);    
    globalThis.bpm = clock.bpm;
  }  

  async function setOutput(out: number | string) {
    logger.info({ setOutput: out });
    midi.selectOutput(out); 
  }  

  async function start() {
    logger.info('start');
    await clock.start();   
    logger.info('after start');
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
    // logger.info({ tick: frame });
    ptn.tick(frame);
    midi.tick(frame);    
  }
  
  async function touch() {
    logger.info('touch');
    clock.touch();   
  }

  // function play(channel: number, octave: Computable<number>, note: Computable<string>, attack: Computable<number>, duration: Computable<number>) {   
  // // function play(channel: number, note: Computable<string>, attack: Computable<number>, duration: Computable<number>) {   
  //   // logger.debug('play'); 
  //   octave = compute(octave);
  //   attack = midify(compute(attack));
  //   duration = compute(duration);
  //   note = compute(note)
    
  //   const ms = duration * clock.msPerBeat();
  //   logger.debug({ms});
  //   midi.push(channel, 1, note, attack, ms);
  // }  

  return {    
    load,
    reset,
    setBPM,
    setOutput,
    setup,
    start,
    stop,
    tick,
    touch
  }
}