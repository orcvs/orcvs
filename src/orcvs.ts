import { Clock } from './clock';
import { Midi } from './midi'
import { codify } from './code'

import { Logger } from "./logger";

export const ORCVS = 'O̴̫͉͌r̸̘͉̫̣̐̈́͊c̶̛̪̖̻͔̈́̃̓v̷̨͎̿͝ŝ̷̩͑̾';

const logger = Logger.child({
  source: 'Orcvs'
});

// declare global {
//   var orcvs: IOrcvs;
//   var bang: any
// }

export function Orcvs() {
  var clock = Clock(tick)
  var midi = Midi();

  async function init() {
    // logger.debug('init');
    await midi.setup();  
    // midi.selectOutput('LoopMidi');
    midi.selectOutput(0);
  }

  async function load(filename: string) {
    
    codify(filename);
  }

  function reset() {
    clock.reset();
  }

  async function setBPM(bpm: number) {
    await clock.setBPM(bpm);
  }  

  async function start() {
    logger.info('start');
    clock.start();   
  }
  
  async function stop() {
    logger.info('stop');
    await clock.stop();
    await midi.stop();
  }

  function tick() {
    midi.tick();
  }
  
  async function touch() {
    logger.info('touch');
    clock.touch();   
  }

  return {
    init,
    reset,
    setBPM,
    start,
    stop,
    tick,
    touch
  }
}
