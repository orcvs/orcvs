import { Logger } from "./logger";

export type OnPulse = (pulsar: Pulsar) => void;

export type Match = (frame : number) => boolean;

export interface Pulsar {
  ptn: (str: string, on: OnPulse) => void;
  at: (str: string, on: OnPulse) => void;
  match: (frame : number) => boolean;
  tick: (frame : number) => void;
  when: (condition: boolean, on: OnPulse) => void;
  // cycle: number;
  // frame: number;
}

const logger = Logger.child({
  source: 'Pulsar'
});

export function pulsar(str: string, on: OnPulse): Pulsar {
  let _pattern: string[] = [];
  // let _frame = 0;
  let patterns: { [name: string]: Pulsar } = {}

  const match = matcher(str);

  const self = {
    at,
    ptn,
    match,
    tick,
    when,
    // get frame() {
    //   return _frame;
    // },
    // get cycle() {
    //   return !_pattern.length ? 0 : Math.floor(_frame/_pattern.length);
    // },
  }

  function matcher(match: string): Match {
    return isTime(match) ? timeMatcher(match) : patternMatcher(match)
  }

  function patternMatcher(match: string): Match {

    _pattern = [...match];

    return function(frame: number) {
      const idx = (frame - 1) % _pattern.length
      return _pattern[idx] === BANG;
    }
  }

  function timeMatcher(match: string): Match {

    const [ from = 0, to = 999 ] = match.split(':').map( s => parseInt(s) || 0);

    return function(frame: number) {
      const f = timeToFrame(from, globalThis.bpm());
      const t = timeToFrame(to, globalThis.bpm());
      // if (f >= t) {
      //   logger.info({msg: 'Invalid time', match});
      //   return false
      // }
      return f <= frame && frame <= t;
    }
  }

  function tick(frame: number) {
    if (match(frame)) {

      on(self);

      for (const ptn of Object.values(patterns)) {
        ptn.tick(frame);
      }
    }
  }

  function ptn(str: string, on: OnPulse) {
    if (!patterns[str]) {
      const ptn = pulsar(str, on);
      patterns[str] = ptn;
    }
  }

  function at(str: string, on: OnPulse) {
    ptn(str, on);
  }

  function when(condition: boolean, on: OnPulse) {
    if (condition) {
      on(self);
    }
  }

  // function on(frame: number, callback: OnPulse) {
  //   // if (condition) {
  //   //   callback(self);
  //   // }
  // }

  return self;
}

export function isTime(str: string) {
  return /^(\d+)?(:\d+)?$/.test(str);
}

export function timeToFrame(time: number, bpm: number): number {
  const beats_per_second = (60 / bpm);
  const frames_per_second = beats_per_second / 4;
  return Math.floor(time/frames_per_second);
}
