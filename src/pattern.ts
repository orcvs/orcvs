
export type Bang = (str: string, callback: Bangable) => void;
// export type Bangable = (bang: any, frame: number, cycle: number) => void;
export type Bangable = (options: {bang: Bang, frame: number, cycle: number}) => void;

export interface Pattern {
  bang: (str: string, callback: any) => void;
  shouldBang:  (f: number) => boolean;
  tick: (f: number) => void;
}

export function pattern(pattern: string, callback: Bangable): Pattern {
  var _pattern: string[] = [...str]
  var _patterns: { [name: string]: Pattern } = {}

  const self = {
    bang,
    shouldBang,
    tick,
  }

  function tick(frame: number) {
    if (shouldBang(frame)) {
      const cycle = Math.floor(frame/_pattern.length); 

      callback({bang, frame, cycle});   

      for (const ptn of Object.values(_patterns)) {
        ptn.tick(frame);
      }
    }
  }

  function bang(str: string, callback: Bangable) {
    if (!_patterns[str]) {
      const ptn = pattern(str, callback);
      _patterns[str] = ptn;
    }
  }

  // function at(time: number, callback: () => void) {
  //   // const frames = timeToFrame(time, bpm());
  //   const ptn = pattern(str, callback);
  // }

  function shouldBang(frame: number) {     
    const idx = (frame - 1) % _pattern.length;
    // console.log(idx)
    // console.log(_pattern[idx])
    // const idx = f % _pattern.length;
    return _pattern[idx] === BANG;
  }

  return self;
}
