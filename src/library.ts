import { Chord } from './note';

export type Computer<T> = ((...opts: any[]) => T);

export type Computable<T> = (T | (() => T));

export function compute<T>(x: Computable<T>): T {
  if (typeof x === 'function') {
    return (x as Function)();
  }
  return x;
}



export function arp(chord: Chord) {
  const s = seq(chord.notes);

  // return (function() {


  // }
}

export function seq<T>(...sequence: readonly T[]): Computer<T> {
  if (Array.isArray(sequence[0])){
    sequence = sequence[0];
  }
  
  const start = 0;
  const end = sequence.length - 1;

  const indexer = cycle(start, end);

  return function(): T {
    const idx = indexer();
    return sequence[idx];
  }
}

export function lerp(to: number): Computer<number>
export function lerp(from: number, to?: number): Computer<number>

export function lerp(tofrom: number, to?: number, diff = 1): Computer<number> {

  const min = to === undefined ? 0 : tofrom;
  const max = to === undefined ? tofrom : to;

  let value: number;
  
  return function() {

    if (value === undefined)  { value = min } else
    if (value < max)          { value = clamp(value + diff, min, max) } else 
    if (value > max)          { value = clamp(value - diff, max, min) };
    
    return value;
  }
}

export function cycle(to: number): Computer<number>
export function cycle(from: number, to: number): Computer<number>

export function cycle(tofrom: number, to?: number, diff = 1): Computer<number> {

  const start = to === undefined ? 0 : tofrom;
  const target = to === undefined ? tofrom : to;

  let value: number;
    
  return function() {
    
    if (value === undefined)  { value = start } else    
    if (value === target)     { value = start } else
    if (value < target)       { value = value + diff};
      
    return value;
  }
}

export function wave(to: number): Computer<number>
export function wave(from: number, to: number): Computer<number>

export function wave(tofrom: number, to?: number, diff = 1): Computer<number> {

  let min = to === undefined ? 0 : tofrom;
  let max = to === undefined ? tofrom : to;

  let value: number;
    
  return function() {
    // At limit, flip min/max      
    if (value === max)        { [min, max] = [max, min] };      

    if (value === undefined)  { value = min } else
    if (value < max)          { value = clamp(value + diff, min, max) } else 
    if (value > max)          { value = clamp(value - diff, max, min) };

    return value;
  }
}
 
const MIDI_MAX = 127
export function midify(value: number) {
  return Math.ceil(value * MIDI_MAX/z);
}

function clamp(v : number, min: number, max: number) { return v < min ? min : v > max ? max : v }
