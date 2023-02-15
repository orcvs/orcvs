import { Clock } from './clock';
import { Midi } from './midi'
import { codify } from './code'

import { Logger } from "./logger";
import { Bangable, Pattern, pattern } from './pattern';
import { compute, Computer, Numputer, midify} from './library';


const logger = Logger.child({
  source: 'Orcvs'
});


export function Orcvs() {
  var clock = Clock(tick)
  var midi = Midi();
  var ptn = pattern(BANG, () => {});  
  // var _patterns: { [name: string]: Pattern } = {}


  function bang(str: string, callback: Bangable) {
    // logger.debug('BANG!');
    ptn.bang(str, callback);
  }

  async function setup() {
    logger.debug('setup');
    registerGlobals();
    await midi.setup();      
    // midi.selectOutput(0); // Default Channel
  }

  function registerGlobals() {    
    globalThis.bang = bang;    
    // globalThis.bpm = clock.bpm;
    globalThis.output = setOutput;
    globalThis.play = play;
  }

  async function load(filename: string) {   
    logger.info(`loading ${filename}`);   
    const code = await codify(filename);
    code();
  }

  function reset() {
    clock.reset();
  }

  async function setBPM(bpm: number) {
    await clock.setBPM(bpm);
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
    logger.info({ tick: frame });
    ptn.tick(frame, clock.bpm());
    midi.tick(frame);    
  }
  
  async function touch() {
    logger.info('touch');
    clock.touch();   
  }

  function play(channel: number, octave: Numputer, note: string, attack: Numputer, duration: Numputer) {   
    logger.info('play'); 
    octave = compute(octave);
    attack = midify(compute(attack));
    duration = compute(duration);

    midi.push(channel, octave, note, attack, duration);
  }

  return {    
    load,
    play,
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