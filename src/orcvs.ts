import { Clock } from './clock';
import { Midi } from './midi'

import { Logger } from "./logger";

const BANG = '!'

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
  var midi = new Midi();

  async function init() {
    // logger.debug('init');
    await midi.setup();  
    // midi.selectOutput('LoopMidi');
    midi.selectOutput(0);
  }

  async function load(filename: string) {
    
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

function clamp(v : number, min: number, max: number) { return v < min ? min : v > max ? max : v }

const varToString = (v: any) => Object.keys(v)[0]

export type Callback = (o: Pattern) => void;

export type Pattern = {
  bang: (str: string, callback: any) => void;
  tick: (f?: number) => void;
  lerp(to: number): Computer;
  lerp(from: number, to?: number): Computer;
  cycle(to: number): Computer
  cycle(from: number, to: number): Computer
  wave(to: number): Computer
  wave(from: number, to: number): Computer
}

// o̶͎͓̜̮͘r̶̼̀̿͗c̵̡̮̮̀v̶̼͂s̵̞͙̅̒̋ͅ

export function PatternImpl(str: string, callback: Callback): Pattern {
  var pattern: string[] = [...str]

  var patterns: { [name: string]: Pattern } = {}

  const self: Pattern = {
    bang,
    cycle,
    lerp,
    tick,
    wave,
  }

  function tick(f: number = 0) {
    if (shouldBang(f)) {
      // ts-expect-error 
      callback(self);   

      for (const pattern of Object.values(patterns)) {
        pattern.tick(f);
      }
    }
  }

  function bang(str: string, callback: Callback) {
    if (!patterns[str]) {
      const ptn = PatternImpl(str, callback);
      patterns[str] = ptn;
    }
  }

  function shouldBang(f: number) {     
    const idx = f % pattern.length;
    return pattern[idx] === BANG;
  }

  return self;
}

export function lerp(to: number): Computer
export function lerp(from: number, to?: number): Computer

export function lerp(tofrom: number, to?: number): Computer {

  const start = to === undefined ? 0 : tofrom;
  const target = to === undefined ? tofrom : to;

  return (function(start: number, target: number) {
    var value: number;
    
    return function() {
      if (value === undefined ) {
        value = start
        return value;
      }
      
      if (value > target) {
        value = clamp(value - 1, target, start);   
      }      
      
      if (value < target) {
        value = clamp(value + 1, start, target);   
      }
     
      return value;
    }
  }(start, target));
}

export function cycle(to: number): Computer
export function cycle(from: number, to: number): Computer

export function cycle(tofrom: number, to?: number): Computer {

  const start = to === undefined ? 0 : tofrom;
  const target = to === undefined ? tofrom : to;

  return (function(start: number, target: number) {
    var value: number;
    
    return function() {
      if (value === undefined ) {
        value = start
        return value;
      }
      
      // At limit, start the cycle
      if (value === target) {
        value = start;
        return value;
      }

      if (value > target) {
        value = clamp(value - 1, target, start);   
      }      
      
      if (value < target) {
        value = clamp(value + 1, start, target);   
      }
     
      return value;
    }
  }(start, target));
}

export function wave(to: number): Computer
export function wave(from: number, to: number): Computer

export function wave(tofrom: number, to?: number): Computer {

  const start = to === undefined ? 0 : tofrom;
  const target = to === undefined ? tofrom : to;

  return (function(start: number, target: number) {
    var value: number;
    
    return function() {
      if (value === undefined ) {
        value = start
        return value;
      }
      
      // At limit, flip start/target
      if (value === target) {
        [start, target] = [target, start];
      }

      if (value > target) {
        value = clamp(value - 1, target, start);   
      }      
      
      if (value < target) {
        value = clamp(value + 1, start, target);   
      }
     
      return value;
    }
  }(start, target));
}

export type Computer = (() => number);

export type Numputer = number | (() => number);

export function compute(n: Numputer) {
  if (typeof n === "function") {
    return n();
  }
  return n;
}

function play(channel: number, octave: Numputer, note: string, attack: Numputer, duration: Numputer) {    
  octave = compute(octave);
  attack = compute(attack);
  duration = compute(duration);

  // this.midi.push(channel, octave, note, attack, duration);
}
