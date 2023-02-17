
export type Computer = (() => number);

export type Numputer = (number | Computer);

export function compute(n: Numputer) {
  if (typeof n === "function") {
    return n();
  }
  return n;
}



export function seq<T>(...sequence: readonly T[]): (() => T) {
  const start = 0;
  const end = sequence.length - 1;

  return (function(start: number, end: number) {
    var idx: number;

    return function(): T {

      if (idx === undefined)  { idx = start } else
      if (idx < end)          { idx = idx + 1 } else       
      if (idx === end)        { idx = start };
      
      return sequence[idx];
    }
  }(start, end));
}

// export function seq(...sequence: readonly (string | number)[]): (() => (string | number)) {
//   const start = 0;
//   const end = sequence.length - 1;

//   return (function(start: number, end: number) {
//     var idx: number;

//     return function()  {

//       if (idx === undefined)  { idx = start } else
//       if (idx < end)          { idx = idx + 1 } else       
//       if (idx === end)        { idx = start };
    
//       return sequence[idx];
//     }
//   }(start, end));
// }

export function lerp(to: number): Computer
export function lerp(from: number, to?: number): Computer

export function lerp(tofrom: number, to?: number, diff = 1): Computer {

  const min = to === undefined ? 0 : tofrom;
  const max = to === undefined ? tofrom : to;

  return (function(min: number, max: number) {
    var value: number;
    
    return function() {

      if (value === undefined)  { value = min } else
      if (value < max)          { value = clamp(value + diff, min, max) } else 
      if (value > max)          { value = clamp(value - diff, max, min) };
     
      return value;
    }
  }(min, max));
}

export function cycle(to: number): Computer
export function cycle(from: number, to: number): Computer

export function cycle(tofrom: number, to?: number, diff = 1): Computer {

  const start = to === undefined ? 0 : tofrom;
  const target = to === undefined ? tofrom : to;

  return (function(start: number, target: number) {
    var value: number;
    
    return function() {
      
      if (value === undefined)  { value = start } else    
      if (value === target)     { value = start } else
      if (value < target)       { value = value + diff};
       
      return value;
    }
  }(start, target));
}

export function wave(to: number): Computer
export function wave(from: number, to: number): Computer

export function wave(tofrom: number, to?: number, diff = 1): Computer {

  const min = to === undefined ? 0 : tofrom;
  const max = to === undefined ? tofrom : to;

  return (function(min: number, max: number) {
    var value: number;
    
    return function() {
      // At limit, flip min/max      
      if (value === max)        { [min, max] = [max, min] };      

      if (value === undefined)  { value = min } else
      if (value < max)          { value = clamp(value + diff, min, max) } else 
      if (value > max)          { value = clamp(value - diff, max, min) };

      return value;
    }
  }(min, max));
}
 
const MIDI_MAX = 127
export function midify(value: number) {
  return Math.ceil(value * MIDI_MAX/Z)
}

function clamp(v : number, min: number, max: number) { return v < min ? min : v > max ? max : v }
