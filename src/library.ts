export const MINUTE = 60000;
let _framesPerBeat = 4;
let _framesPerPhrase = 4;

export function clamp(v : number, min: number, max: number) { return v < min ? min : v > max ? max : v }

export function framesPerBeat(set?: number) {
  if (set) {
    _framesPerBeat = set;
  }
  return _framesPerBeat;
}

export function framesPerPhrase(set?: number) {
  if (set) {
    _framesPerPhrase = set;
  }
  return _framesPerPhrase;
}

export function flipPulse(pattern: string | number | number[]): number[] {
  if (!Array.isArray(pattern)) {
    pattern = toPulse(pattern);
  }
  const ary = [];
  for(let n of pattern) {
    const a = n ? 0 : 1;
    ary.push(a);
  }
  return ary;
}

export function midify(value?: number) {
  if (!value) return value;
  return Math.ceil(value * 127/z);
}

export function msPerBeat() {
  return ( MINUTE  / bpm()) / framesPerBeat();
}

export function toPulse(pattern: string | number): number[] {
  pattern = pattern.toString().replaceAll(/[,.\s]/g, '');
  const ary = [];
  for(let char of pattern) {
    const a = (char === BANG || char === '1') ? 1 : 0;
    ary.push(a);
  }
  return ary;
}

export function pulseOnBeat(beat = 1): number[] {
  const len = beat * framesPerBeat();

  const ary = Array(len).fill(0);
  ary[0] = 1;

  return ary;
}

export function unwrap<T>(item: T[]): T | T[] {
  return item.length === 1 ? item[0] : item;
}

export function wrap<T>(item: T | T[]): T[] {
  return Array.isArray(item) ? item : [item];
}

export function merge<T>(opts: T, ...arrays: Partial<T>[]): T[] {

  const ary = []
  let len = arrays.length;

  for (let i = 0; i < len; i++) {
    const o = { ...opts, ...arrays[i] };
    ary.push(o);
  }

  return ary;
}
