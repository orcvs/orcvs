
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

// export function euclid(steps = 8, beats = 4, rotateSteps = 0): string[] {
//   return generateEuclid(steps, beats, rotateSteps));
// }

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

  // sequence = toSequence(sequence);

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

// export function random(to: number): Computer<number>
export function random(tofrom: number, to?: number): Computer<number> {

  const min = to === undefined ? 1 : tofrom;
  const max = to === undefined ? tofrom : to;

  return function() {
    return Math.floor(min + Math.random()*(max - min + 1))
  }
}

export function midify(value?: number) {
  if (!value) return value;
  return Math.ceil(value * 127/z);
}

export const MINUTE = 60000;
let _framesPerBeat = 4;

export function msPerBeat() {
  return ( MINUTE  / bpm()) / framesPerBeat();
}

export function framesPerBeat(set?: number) {
  if (set) {
    _framesPerBeat = set;
  }
  return _framesPerBeat;
}

// function toSequence<T>(sequence: T[]): T[] {
//   if (sequence.length === 1) {
//     const item = sequence[0];

//     if (Array.isArray(item)){
//       return item;
//     }

//     if (typeof item === 'string') {
//       return item.split('') as T[]
//     }
//   }

//   return sequence as T[];
// }

export function Queue(length = 100) {
  let _queue: number[] = [];

  function push(t: number) {
    _queue.push(t);
    if (_queue.length > length) {
      _queue.shift()
    }
  }

  function get(t: number) {
    return _queue;
  }

  function percentile() {
    const sorted = _queue.sort((a, b) => a - b);
    const index = Math.ceil(0.90 * sorted.length);
    return sorted[index].toFixed(4);
  }

  function average() {
    return _queue.reduce( (a,e,i) => (a*i+e)/(i+1)).toFixed(4);
  }

  return {
    push,
    get,
    percentile,
    average
  }
}

export function clamp(v : number, min: number, max: number) { return v < min ? min : v > max ? max : v }

export function wrap<T>(item: T | T[]): T[] {
  return Array.isArray(item) ? item : [item];
}

export function toBeatArray<T>(pattern: string): number[] {
  pattern = pattern.replaceAll(/[,.\s]/g, '');
  const ary = [];
  for(let char of pattern) {
    const a = (char === BANG || char === '1') ? 1 : 0;
    ary.push(a);
  }
  return ary;
}