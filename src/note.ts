import { Utilities as BaseUtilities } from 'webmidi';
import { Computable, Computer, seq, seqLerp } from './library';


export interface Playable {
  notes: Note[];
}

export interface Note extends Playable{
  value: number; 
  duration?: Computable<number>;
  attack?: Computable<number>;
  release?: Computable<number>;
}

export interface Chord extends Playable {
  name: string;
  root: Note; 
  intervals: number[];  
}

export interface Options {
  d?: Computable<number>;
  a?: Computable<number>;
  r?: Computable<number>;
}

export function chord(chord: string, ...options: Options[]): Chord {  
  const {name, intervals} = toChord(chord);

  const opts = seqLerp(...options);

  const notes: Note[] = [];

  const value = Utilities.toNoteNumber(name);
  const root = createNote(value, opts());
  notes.push(root);

  for (const int of intervals) {   
    const v = Utilities.getOffsetNumber(value, int);
    const note = createNote(v, opts());
    notes.push(note);
  }  

  return {
    name: chord,
    intervals, 
    notes,
    get root() {
      return notes[0];
    }
  };
}

function createNote(value: number, options: Options = {}) {
  const {d: duration, a: attack, r: release} = options;
  return {
    value,
    duration,
    attack, 
    release,
    get notes() {
      return [this];
    }
  }
}

export function note(name: string): Computer<Note> {  
  const value = Utilities.toNoteNumber(name);
  return function(options: Options = {}) {
    return createNote(value, options);
  }
}

class Utilities extends BaseUtilities {
  static getOffsetNumber(number: number, semitoneOffset = 0) {
    return Math.min(Math.max(number + semitoneOffset, 0), 127);
  }  
  // static getOffsetNumber(value: number, octaveOffset = 0, semitoneOffset = 0) {
  //   return Math.min(Math.max(value + octaveOffset * 12 + semitoneOffset, 0), 127);
  // }  
}

export const ChordIntervalsTable: { [key: string]: number[] }  = {
  'M':    [ 4, 7 ],
  'M7':   [ 4, 7, 11 ],
  'M9':   [ 4, 7, 11, 14 ],
  'M13':  [ 4, 7, 11, 14, 21 ],
  'M6':   [ 4, 7, 9 ],
  'M69':  [ 4, 7, 9, 14 ],
  'M7b6': [ 4, 8, 11 ],
  'm':    [ 3, 7 ],
  'm7':   [ 3, 7, 10 ],
  'mM7':  [ 3, 7, 11 ],
  'm6':   [ 3, 7, 9 ],
  'm9':   [ 3, 7, 10, 14 ],
  'm11':  [ 3, 7, 10, 14, 17 ],
  'm13':  [ 3, 7, 10, 14, 21 ],
  'o':    [ 3, 6 ],
  'o7':   [ 3, 6, 9 ],
  'm7b5': [ 3, 6, 10 ],
  '7b9':  [ 4, 7, 10, 13 ],
  '7':    [ 4, 7, 10 ],
  '9':    [ 4, 7, 10, 14 ],
  '13':   [ 4, 7, 10, 14, 21 ],
}

function toChord(value: string): {name:string, intervals:number[]} {
  const [ name, quality] = value.split(':');
  const intervals = ChordIntervalsTable[quality];
  return {name, intervals};
}

export const ChordTable: { [key: string]: string[] } = {
  'M':    ['P1', 'M3', 'm3'],
  'M7':   ['P1', 'M3', 'm3', 'M3'],
  'M9':   ['P1', 'M3', 'm3', 'M3', 'm3'],
  'M13':  ['P1', 'M3', 'm3', 'M3', 'm3', 'P5'],
  'M6':   ['P1', 'M3', 'm3', 'M2'],
  'M69':  ['P1', 'M3', 'm3', 'M2', 'P4'],
  'M7b6': ['P1', 'M3', 'M3', 'm3'],
  'm':    ['P1', 'm3', 'M3'],
  'm7':   ['P1', 'm3', 'M3', 'm3'],
  'mM7':  ['P1', 'm3', 'M3', 'M3'],
  'm6':   ['P1', 'm3', 'M3', 'M2'],
  'm9':   ['P1', 'm3', 'M3', 'm3', 'M3'],
  'm11':  ['P1', 'm3', 'M3', 'm3', 'M3', 'm3'],
  'm13':  ['P1', 'm3', 'M3', 'm3', 'M3', 'P5'],
  'o':    ['P1', 'm3', 'm3'],
  'o7':   ['P1', 'm3', 'm3', 'm3'],
  'm7b5': ['P1', 'm3', 'm3', 'M3'],
  '7':    ['P1', 'M3', 'm3', 'm3'],
  '9':    ['P1', 'M3', 'm3', 'm3', 'M3'],
  '13':   ['P1', 'M3', 'm3', 'm3', 'M3', 'P5'],
  '7b9':  ['P1', 'M3', 'm3', 'm3', 'm3'],
}

export const IntervalTable: { [key: string]: number } = {
  'P1': 0,
  'm2': 1,
  'M2': 2,
  'm3': 3,
  'M3': 4,
  'P4': 5,
  'P5': 7,
}

export function writeIntervals(chord: string){
  const results: { [key: string]: number[] } = {};
  for (let chord in ChordTable) {

    const result = getIntervals(chord)
    // const intervals = ChordTable[chord];
    results[chord] = result;
  }
  return results;
}

export function getIntervals(chord: string){
  const intervals = ChordTable[chord] || ['P1']

  let fromRoot = 0
  const result = []
  for (const i of intervals) {
    const interval = IntervalTable[i]
    if (interval !== 0) {
      fromRoot += interval
      result.push(fromRoot)
    }    
  }
  return result
}