import { Note } from 'webmidi';
import { Logger } from "./logger";

import { Clock } from './clock';
import { Midi } from './midi'
import { codify } from './code'

import { Callback, Pattern, pattern } from './pattern';

const logger = Logger.child({
  source: 'Orcvs'
});

export function Orcvs() {
  let clock = Clock(tick)
  let midi = Midi();
  let ptn = pattern(BANG, () => {});  
  let code = () => {};
  let hasRun: boolean = false;

  
  function bang(str: string, callback: Callback): void {
    // logger.debug('BANG!');
    ptn.bang(str, callback);
  }

  async function setup() {
    console.log(`Welcome to ${ORCVS}`);
    logger.debug('setup');
    registerGlobals();
    await midi.setup();      
  }

  function registerGlobals() {    
    globalThis.bang = bang;    
    globalThis.bpm = clock.bpm;
    globalThis.output = setOutput;
    globalThis.play = midi.play;
    globalThis.out = setOutput;
    globalThis.ply = midi.play;
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
    
    ptn = pattern(BANG, () => {});  
    code();
    hasRun = true;
    
    logger.info('ran');
  }
  
  function shouldRun(frame: number) {
    return !clock.running || !hasRun && frame % 8 === 0
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
    ptn.tick(frame);
    midi.tick(frame);    
  }
  
  async function touch() {
    logger.info('touch');
    clock.touch();   
  }

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