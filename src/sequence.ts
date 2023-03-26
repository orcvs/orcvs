import { clamp } from "./library";

export type Computer<T> = ((...opts: any[]) => T);

export type Computable<T> = (T | (() => T));

export type Indexer = (tofrom: number, to?: number, diff?: number) => Computer<number>;

export type Seq<T> = Computer<T>;

export function compute<T>(x: Computable<T>): T {
  if (typeof x === 'function') {
    return (x as Function)();
  }
  return x;
}

export function seq<T>(...sequence: T[]): Computer<T> {
  return sequencer(cycle, ...sequence);
}

export function seqL<T>(...sequence: T[]): Computer<T> {
  return sequencer(lerp, ...sequence);
}

export function seqW<T>(...sequence: T[]): Computer<T> {
  return sequencer(lerp, ...sequence);
}

export function seqR<T>(...sequence: T[]): Computer<T> {
  return sequencer(random, ...sequence);
}

export function sequencer<T>(indexer: Indexer, ...sequence: T[]): Computer<T> {
  const start = 0;
  const end = sequence.length - 1;

  const idx = indexer(start, end);

  return function(): T {
    const i = idx();
    return sequence[i] as T;
  }
}

export function lerp(to: number): Computer<number>
export function lerp(from: number, to?: number): Computer<number>

export function lerp(tofrom: number, to?: number, diff = 1): Computer<number> {

  const min = to === undefined ? 1 : tofrom;
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

export function random(tofrom: number, to?: number): Computer<number> {

  const min = to === undefined ? 1 : tofrom;
  const max = to === undefined ? tofrom : to;

  return function() {
    return Math.floor(min + Math.random()*(max - min + 1))
  }
}
