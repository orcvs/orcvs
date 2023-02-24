import { Note } from 'webmidi';
import { Computer, Computable} from './library';


export type NoteOpts = {
  d?: number;
  a?: number;
  r?: number;
}

export class Chord extends Note {
  notes: Note[] = [this];

  constructor(value: string | number, options?: {
    duration?: number,
    attack?: number,
    release?: number,
    rawAttack?: number,
    rawRelease?: number,
  }) {
    super(value, options);
  }

}

export function chord(value: string, opts: NoteOpts = {}): Chord {
  let _root: Chord;

  const {d: duration, a: attack, r: release} = opts;

  const {root, intervals} = toChord(value);
  _root = new Chord(root);

  for (const i of intervals) {      
    const value = _root.getOffsetNumber(0, i);
    const note = new Note(value, {duration, attack, release});
    _root.notes.push(note);
  }

  return _root;
}

export function note(value: string | number): Computer<Note> {  

  return function(opts: NoteOpts = {}) {
    const {d: duration, a: attack, r: release} = opts;
    return new Note(value, {duration, attack, release} )
  }
}

export const ChordIntervalsTable: { [key: string]: number[] }  = {
  '7': [ 4, 7, 10 ],
  '9': [ 4, 7, 10, 14 ],
  '13': [ 4, 7, 10, 14, 21 ],
  'M': [ 4, 7 ],
  'M7': [ 4, 7, 11 ],
  'M9': [ 4, 7, 11, 14 ],
  'M13': [ 4, 7, 11, 14, 21 ],
  'M6': [ 4, 7, 9 ],
  'M69': [ 4, 7, 9, 14 ],
  'M7b6': [ 4, 8, 11 ],
  'm': [ 3, 7 ],
  'm7': [ 3, 7, 10 ],
  'mM7': [ 3, 7, 11 ],
  'm6': [ 3, 7, 9 ],
  'm9': [ 3, 7, 10, 14 ],
  'm11': [ 3, 7, 10, 14, 17 ],
  'm13': [ 3, 7, 10, 14, 21 ],
  'o': [ 3, 6 ],
  'o7': [ 3, 6, 9 ],
  'm7b5': [ 3, 6, 10 ],
  '7b9': [ 4, 7, 10, 13 ]
}

function toChord(str: string): {root:string, intervals:number[]} {
  const [ root, quality] = str.split(':');
  const intervals = ChordIntervalsTable[quality];
  return {root, intervals};
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