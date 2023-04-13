import { Utilities } from 'webmidi';
import { wrap } from './library';
import { Computable, Computer, seq, seqL } from './sequence';

export interface Note {
  value: number;
  duration?: Computable<number>;
  attack?: Computable<number>;
  release?: Computable<number>;
}

export interface Option {
  d?: Computable<number>;
  a?: Computable<number>;
  r?: Computable<number>;
}

function isOption(obj: any): obj is Option {
  return 'd' in obj || 'a' in obj || 'r' in obj;
}

export function arp(value: string, ...options: Option[]): Computer<Note> {

  const crd = chord(value, ...options);
  const notes = seq(...crd);

  return function(): Note {
    return notes();
  }
}

export function chord(chord: string, ...options: Option[]): Note[] {
  const {name, intervals} = toChord(chord);

  const opts = seqL(...options);

  const notes: Note[] = [];

  const value = Utilities.toNoteNumber(name);
  const root = createNote(value, opts());
  notes.push(root);

  for (const int of intervals) {
    const v = Utilities.offsetNumber(value, 0, int);
    const note = createNote(v, opts());
    notes.push(note);
  }

  return notes;
}

function createNote(value: number, options: Option = {}) {
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
  return function(options: Option = {}) {
    return createNote(value, options);
  }
}

export function notes(...args: any[]): Note[] {

  const notes = [] as Computer<Note>[];
  const options = [] as Option[];

  for (const arg of args) {
    if (isOption(arg)) {
      options.push(arg);
    } else {
      notes.push(arg);
    }
  }

  const opts = seqL(...options);

  const ary = [] as Note[];
  for (const n of notes) {
    const note = n.call(undefined, opts());
    ary.push(note);
  }
  return ary;
}

export function transpose(notes: Note | Note[], semitoneOffset = 12): Note[] {
  const ary: Note[] = [];
  notes = wrap(notes);
  for (const n of notes) {
    const value = Utilities.offsetNumber(n.value, 0, semitoneOffset);
    const note = createNote(value, toOption(n));
    ary.push(note);
  }
  return ary;
}

function toOption(note: Note): Option {
  return {d: note?.duration, a: note?.attack, r: note?.release};
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
  const [ name, quality] = value.replace('$', '#').split(':');
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