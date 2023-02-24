import { Note } from 'webmidi';
import { Computer, Computable} from './library';


export type NoteOpts = {
  d?: number;
  a?: number;
  r?: number;
}

// export type Chord = {
//   notes: Note[];
// }
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

const ChordTable: { [key: string]: string[] } = {
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

const IntervalTable: { [key: string]: number } = {
  'P1': 0,
  'm2': 1,
  'M2': 2,
  'm3': 3,
  'M3': 4,
  'P4': 5,
  'P5': 7,
}

function getIntervals(chord: string){
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

function toChord(str: string): {root:string, intervals:number[]} {
  // const octave = s[0]
  // const tonic = s[0]
  // const chord = s.slice(1)
  // const intervals = mapChordIntervals(chord)

  // const r = /^[a-g]#?/i
  // ^([CDEFGAB])(#{0,2}|b{0,2})(-?\d+)$
  
  const [ root, quality] = str.split(':');

  // const matches = str.match(/^([CDEFGAB])(#{0,2}|b{0,2})(-?\d+)$/i);
  // if (!matches) throw new TypeError("Invalid note identifier");

  // const matches = str.match(r);
  // const root = matches ? matches[0] : '';
  // const quality = str.replace(root, '');
  const intervals = getIntervals(quality);
  
  return {root, intervals};
}
