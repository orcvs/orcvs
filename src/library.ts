import { hash } from '../src/hash';


// o̶͎͓̜̮͘r̶̼̀̿͗c̵̡̮̮̀v̶̼͂s̵̞͙̅̒̋ͅ


export const BANG = '!'

export const A = 10;
export const B = 11;
export const C = 12;
export const D = 13;
export const E = 14;
export const F = 16;
export const G = 17;
export const H = 18;
export const I = 19;
export const J = 20;
export const K = 21;
export const L = 22;
export const M = 23;
export const N = 24;
export const O = 25;
export const P = 26;
export const Q = 27;
export const R = 28;
export const S = 29;
export const T = 30;
export const U = 31;
export const V = 32;
export const W = 33;
export const X = 34;
export const Y = 35;
export const Z = 36;

export type Bang = (str: string, callback: any) => void;
export type Callback = (bang: Bang) => void;

export interface Pattern {
  bang: (str: string, callback: any) => void;
  shouldBang:  (f: number) => boolean;
  tick: (f?: number) => void;
}

export function pattern(str: string, callback: Callback): Pattern {
  var _pattern: string[] = [...str]

  var _patterns: { [name: string]: Pattern } = {}
  // var _patterns: [str: string, ptn: Pattern ][]= []
  
  // var _patterns: Pattern[]= []

  const self = {
    bang,
    shouldBang,
    tick,
  }

  function tick(f: number = 0) {
    if (shouldBang(f)) {
      // ts-expect-error 
      callback(bang);   

      // for (const ptn of _patterns) {
      //   ptn.tick(f);
      // }

      for (const ptn of Object.values(_patterns)) {
        ptn.tick(f);
      }

    }
  }

  function bang(str: string, callback: Callback) {
    if (!_patterns[str]) {
      const ptn = pattern(str, callback);
      _patterns[str] = ptn;
    }
  }

  function shouldBang(f: number) {     
    const idx = f % _pattern.length;
    return _pattern[idx] === BANG;
  }

  return self;
}

export type Computer = (() => number);

// export type Numputer = number | (() => number);

export function compute(n: number | Computer) {
  if (typeof n === "function") {
    return n();
  }
  return n;
}

function clamp(v : number, min: number, max: number) { return v < min ? min : v > max ? max : v }


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

const MIDI_MAX = 127
export function midify(value: number) {
  return Math.ceil(value * MIDI_MAX/Z)
}

function play(channel: number, octave: Computer, note: string, attack: Computer, duration: Computer) {    
  const _octave = compute(octave);
  const _attack = midify(compute(attack));
  const _duration = midify(compute(duration));



  // this.midi.push(channel, octave, note, attack, duration);
}
