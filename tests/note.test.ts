import {jest} from '@jest/globals'

import { Note } from 'webmidi';

import { chord, note } from '../src/note';

require('../src/globals');

describe('note', () => {

    test('note', async () => { 
      var note = C();

      expect(note.name).toEqual('C');
      expect(note.octave).toEqual(4);
    });

    test('note', async () => { 
      var note = C();

      expect(note.name).toEqual('C');
      expect(note.octave).toEqual(4);
    });

    test('note', async () => { 
      var note = C({d: 8, a: 1})

      expect(note.name).toEqual('C');

      expect(note.octave).toEqual(4);      
      expect(note.duration).toEqual(8);
      expect(note.attack).toEqual(1);

    });
    test('chord', async () => { 
      var c = chord('C4:M');
      var notes = c.notes;

      expect(notes).toHaveLength(3);
      expect(c.name).toContainEqual(C().name);
      // expect(notes).toContainEqual(C());
      expect(notes).toContainEqual(E());
      expect(notes).toContainEqual(G());
    });

    test('complex chord', async () => { 
      var c = chord('C4:m13');
      var notes = c.notes;
      
      // console.log({notes})
      expect(notes).toHaveLength(6);
      // expect(notes).toContainEqual(C());      
      expect(c.name).toContainEqual(C().name);
      expect(notes).toContainEqual(D$());
      expect(notes).toContainEqual(G());
      expect(notes).toContainEqual(A$());
      expect(notes).toContainEqual(D5());
      expect(notes).toContainEqual(A5());
    });
});

