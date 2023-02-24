import { Logger } from "./logger";

export type Bang = (str: string, callback: Callback) => void;

export type Callback = (pattern: Pattern) => void;

export type Match = (frame : number) => boolean;

const _GLOBAL = globalThis; 

const logger = Logger.child({
  source: 'Pattern'
});

export interface Pattern {
  bang: (str: string, callback: Callback) => void;
  at: (str: string, callback: Callback) => void;
  match: (frame : number) => boolean;
  tick: (frame : number) => void;
  when: (condition: boolean, callback: Callback) => void; 
  cycle: number;
  frame: number;
}

export function pattern(str: string, callback: Callback): Pattern {
  var _pattern: string[] = [];
  var _frame = 0;
  var _patterns: { [name: string]: Pattern } = {}

  const match = matcher(str);

  const self = {
    at,
    bang,
    match,
    tick,
    when,
    get frame() {
      return _frame;
    },
    get cycle() {
      return !_pattern.length ? 0 : Math.floor(_frame/_pattern.length);
    },
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
      const f = timeToFrame(from, _GLOBAL.bpm);
      const t = timeToFrame(to, _GLOBAL.bpm);
      // if (f >= t) {
      //   logger.info({msg: 'Invalid time', match});
      //   return false
      // }
      return f <= frame && frame <= t;
    }
  }

  function tick(frame: number) {       
    if (match(frame)) {
     
      callback(self);   

      for (const ptn of Object.values(_patterns)) {
        ptn.tick(frame);
      }
    }
  }

  function bang(str: string, callback: Callback) {
    if (!_patterns[str]) {
      const ptn = pattern(str, callback);
      _patterns[str] = ptn;
    }
  }

  function at(str: string, callback: Callback) {
    bang(str, callback);
  }
  
  function when(condition: boolean, callback: Callback) {
    if (condition) {
      callback(self);   
    }
  }

  function on(frame: number, callback: Callback) {
    // if (condition) {
    //   callback(self);   
    // }
  }

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

  // function at(time: number, callback: () => void) {
  //   // const frames = timeToFrame(time, bpm());
  //   const ptn = pattern(str, callback);
  // }
// type at = {t: number, f?: number};

// export function pulsar(ptn: string, callback: Bangable): void

// export function pulsar(at: at, callback: Bangable): void

// export function pulsar(param: string | at, callback: Bangable) {
//   var _patterns: { [name: string]: Pattern } = {}
//   var _pattern: string[];;  
//   var _to: number | undefined;
//   var _from: number | undefined;

//   if (typeof param === "string") {
//     _pattern =  [...param]
//   } else {
//     ({ t: _to, f: _from } = param);
//     console.log({ _to, _from });
//   }

// }
// export function pulsar(from: number, to: number, callback: Bangable) {  

//   const self = {
//     bang,
//     shouldBang,
//     tick,
//   }

//   function tick(frame: number) {
//     // if (shouldBang(frame)) {
//     //   const cycle = Math.floor(frame/_pattern.length); 

//     //   callback({bang, frame, cycle});   

//     //   for (const ptn of Object.values(_patterns)) {
//     //     ptn.tick(frame);
//     //   }
//     // }
//   }

//   // function at(time: number, callback: () => void) {
//   //   // const frames = timeToFrame(time, bpm());  
//   //   const ptn = pattern(str, callback);
//   // }

//   function shouldBang(frame: number) {   
//     const timeframe = timeToFrame(time, bpm());  
//     return timeframe === frame;
//   }

//   return self;
// }



// stof: (p, origin) => { // seconds to frames
//   const beats_per_min = (60 / client.clock.speed.value)
//   const frames_per_second = beats_per_min/ 4
//   const frames = Math.floor(parseInt(p.str) / frames_per_second)
//   const nearest_beat = Math.round(frames / 4) * 4;
//   const zz = nearest_beat.toString(36)
//   client.orca.writeBlock(origin ? origin.x : client.cursor.x, origin ? origin.y : client.cursor.y, `${zz}.${frames}.${nearest_beat}`)

// },