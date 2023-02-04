import { Clock } from './clock';
import { Midi } from './midi'

const BANG = '!'

export interface IOrcvs {
  // bang: (pattern: string, fn: Callback) => void;
}

export class Orcvs implements IOrcvs {
  clock: Clock;
  midi: Midi;
  // pattern: Pattern;
  // patterns: Pattern[] = [];

  constructor() { 
    this.clock = new Clock( () => this.tick() )
    console.log( this.clock )
    this.midi = new Midi();
  }

  async init() {
    await this.midi.setup();
    this.midi.selectOutput('LoopMidi');
  }

  async stop() {
    await this.midi.stop();
  }

  // bang(pattern: string, fn: Callback) {
  //   const ptn = PatternImpl(pattern, fn);
  //   this.patterns.push(ptn);
  // }

  play(channel: number, octave: number, note: string, attack: number, duration: number) {    
    this.midi.push(channel, octave, note, attack, duration);
  }

  tick() {
    // console.info('Orcvs', 'tick');
    // console.info('Orcvs', 'tick', this);
    // this.runPatterns();
    this.midi.tick();
  }

  async run() {
    await this.clock.start()
  }

  // runPatterns() {
  //   const frame = this.clock.frame;
  //   for (const pattern of this.patterns) {
  //     // console.log('run_patterns', { frame });
  //     pattern.bang(frame);
  //   }
  // }
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
    tick,
    bang,
    lerp,
    cycle,
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
    return pattern[idx] === '!'
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

