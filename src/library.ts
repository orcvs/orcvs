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

export function clamp(v : number, min: number, max: number) { return v < min ? min : v > max ? max : v }

export function wrap<T>(item: T | T[]): T[] {
  return Array.isArray(item) ? item : [item];
}

export function toPulse<T>(pattern: string): number[] {
  pattern = pattern.replaceAll(/[,.\s]/g, '');
  const ary = [];
  for(let char of pattern) {
    const a = (char === BANG || char === '1') ? 1 : 0;
    ary.push(a);
  }
  return ary;
}

export function flipPulse<T>(pattern: string | number[]): number[] {
  if (typeof pattern === 'string') {
    pattern = toPulse(pattern);
  }
  const ary = [];
  for(let n of pattern) {
    const a = n ? 0 : 1;
    ary.push(a);
  }
  return ary;
}
