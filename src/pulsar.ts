import { seq } from "./sequence";

import { toPulse, unwrap, wrap } from "./library";

import { Logger } from "./logger";

export type OnPulse = (pulsar: Pulsar) => void;

export type Match = (frame : number) => boolean;

export interface Pulsar {
  ptn: (...args: PulsarArgs) => void;
  at: (...args: PulsarArgs) => void;
  tick: (frame : number) => void;
  // cycle: number;
  // frame: number;
}

const logger = Logger.child({
  source: 'Pulsar'
});

enum Matcher {
  Pattern,
  Time,
  Frame,
}

function toPattern(args: string | number[]) {
  if (args.length === 1) {
    const first = args[0];
    if (typeof first === 'string') {
      return first;
    } else {
      return wrap(first);
    }
  }
  return args as number[];
}

export type PulsarArgs =  [string, OnPulse] | [number[], OnPulse] | [...number[], OnPulse];

export function pulsar(...args: PulsarArgs): Pulsar {
  const on: OnPulse = args.pop() as OnPulse;

  if (typeof on != 'function') {
    throw Error('Expected Function as last parameter');
  }

  const pattern = toPattern(args as string | number[])

  const patterns: { [name: string]: Pulsar } = {}

  const match = matcher(pattern);

  const self = {
    at,
    ptn,
    tick,
    // get frame() {
    //   return _frame;
    // },
    // get cycle() {
    //   return !_pattern.length ? 0 : Math.floor(_frame/_pattern.length);
    // },
  }

  function tick(frame: number) {
    if (match(frame)) {
      on(self);

      for (const ptn of Object.values(patterns)) {
        ptn.tick(frame);
      }
    }
  }

  function ptn(...args: PulsarArgs) {
    const key = args.slice(0, -1).toString();
    if (!patterns[key]) {
      const ptn = pulsar(...args);
      patterns[key] = ptn;
    }
  }

  function at(...args: PulsarArgs) {
    ptn(...args);
  }

  return self;
}

export function matcher(pattern: string  | number[]): Match {

  if (Array.isArray(pattern)) {
    return patternMatcher(pattern);
  }

  if (isTime(pattern)) {
    return timeMatcher(pattern);
  }

  if (isFrameTime(pattern)) {
    return frameMatcher(pattern);
  }

  return patternMatcher(pattern);
}

function patternMatcher(pattern: string  | number[]): Match {
  if (typeof pattern === 'string') {
    pattern = toPulse(pattern)
  }

  let _pattern = seq(...pattern);

  return function() {
    return _pattern() === 1;
  }
}

function timeMatcher(match: string): Match {

  const [ from = 0, to = 9999 ] = match.slice(2).split(':').map( s => parseInt(s) || 0);

  return function(frame: number) {
    const f = timeToFrame(from, globalThis.bpm());
    const t = timeToFrame(to, globalThis.bpm());

    return f <= frame && frame <= t;
  }
}

function frameMatcher(match: string): Match {
  const [ from = 0, to = 9999 ] = match.slice(2).split(':').map( s => parseInt(s) || 0);

  return function(frame: number) {
    return from <= frame && frame <= to;
  }
}

// Time format string is {start}?:{stop}?
// Both start and stop are optional
export function isTime(str: string) {
  // ^(\d+)?    - start of string, and one or more numbers
  // (:\d+)?$   - ':' followed by one or more numbers
  return /^(t:)(\d+)?(:\d+)?$/.test(str);
}

export function isFrameTime(str: string) {
  return /^(f:)(\d+)?(:\d+)?$/.test(str);
}

export function timeToFrame(time: number, bpm: number): number {
  const beats_per_second = (60 / bpm);
  const frames_per_second = beats_per_second / 4;
  return Math.floor(time/frames_per_second);
}
