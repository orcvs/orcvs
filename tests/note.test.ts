import {jest} from '@jest/globals'

import { arp, chord, notes, Note, transpose} from '../src/note';

import { lerp } from '../src/sequence';
import { unwrap } from '../src/library';

require('../src/globals');

describe('notes & chords', () => {

  describe('arpeggiator', () => {

    test('arp', async () => {
      const crd = chord('C4:M');
      // crd.arp;
      // arp(crd);
    });
  });

  describe('note', () => {

    test('is a global function', async () => {
      const result = typeof C === 'function';
      expect(result).toBeTruthy();
    });

    test('factory returns a note', async () => {
      const note = C();

      expect(note.value).toEqual(60);
      expect(note).toEqual(C4());
    });

    test('can be created with options', async () => {
      const note = C2({d: 8, a: 1});

      expect(note.value).toEqual(36);
      expect(note.duration).toEqual(8);
      expect(note.attack).toEqual(1);
    });

    test('options are computable', async () => {
      const lerper = lerp(1, 10);

      let note = C({d: 8, a: lerper});
      expect(note.attack).toEqual(lerper);

      note = C({d: 8, a: lerper()});
      expect(note.attack).toEqual(1);
    });

  });

  describe('notes', () => {

    test('creates note with options', async () => {
      const result = notes(A$2, {d:8});
      const note = result[0];
      expect(note.duration).toBe(8);
    });

    test('creates several notes with options', async () => {
      const result = notes(A$2, C, D3, {d:8});
      expect(result.length).toEqual(3);
      for (const note of result) {
        expect(note.duration).toBe(8);
      }
    });

    test('creates notes with correct options', async () => {
      const result = notes(A$2, C, D3, {d:8}, {a: 1, d: 4});
      expect(result.length).toEqual(3);

      const note = result[0];
      expect(note.duration).toBe(8);

      for (const note of result.slice(1)) {
        expect(note.duration).toBe(4);
        expect(note.attack).toBe(1);
      }
    });

  });

  describe('chord', () => {

    test('generates correct notes', async () => {
      const notes = chord('C4:M');

      expect(notes).toHaveLength(3);

      expect(notes).toContainEqual(C()); // root note is chord
      expect(notes).toContainEqual(E());
      expect(notes).toContainEqual(G());
    });

    test('complex chord', async () => {
      const notes = chord('C4:m13');

      expect(notes).toHaveLength(6);
      expect(notes).toContainEqual(C());
      expect(notes).toContainEqual(D$());
      expect(notes).toContainEqual(G());
      expect(notes).toContainEqual(A$());
      expect(notes).toContainEqual(D5());
      expect(notes).toContainEqual(A5());
    });


    test('can be created with options', async () => {
      const notes = chord('C4:M', {d: 1, a: 2, r: 3});
      const root = notes[0];

      expect(notes).toHaveLength(3);

      expect(root.duration).toEqual(1);
      expect(root.attack).toEqual(2);
      expect(root.release).toEqual(3);
    });

    test('options apply to all notes', async () => {
      const notes = chord('C4:M', {d: 1, a: 2, r: 3});

      expect(notes).toHaveLength(3);
      for(let note of notes) {
        expect(note.duration).toEqual(1);
        expect(note.attack).toEqual(2);
        expect(note.release).toEqual(3);
      }
    });

    test('each note can have options', async () => {
      const notes = chord('C4:M', {d: 1, a: 1, r: 1}, {d: 2, a: 2, r: 2}, {d: 3, a: 3, r: 3});

      expect(notes).toHaveLength(3);
      for(let idx = 0; idx < notes.length; idx++) {
        let expected = idx + 1;
        let note = notes[idx];
        expect(note.duration).toEqual(expected);
        expect(note.attack).toEqual(expected);
        expect(note.release).toEqual(expected);
      }
    });

    test('last option applies for all remaining', async () => {
      const notes = chord('C4:M', {d: 1, a: 2, r: 3}, {d: 2, a: 2, r: 2});

      const root = notes[0];

      expect(notes).toHaveLength(3);

      expect(root.duration).toEqual(1);
      expect(root.attack).toEqual(2);
      expect(root.release).toEqual(3);

      for(let idx = 1; idx < notes.length; idx++) {
        let expected = 2;
        let note = notes[idx];
        expect(note.duration).toEqual(expected);
        expect(note.attack).toEqual(expected);
        expect(note.release).toEqual(expected);
      }
    });


  });

  describe('arp', () => {

    test('generates correct notes', async () => {
      const a = arp('C4:M');
      const notes = chord('C4:M');

      for(let note of notes) {
        const next = a();
        expect(next).toEqual(note);
      }
    });

    test('generates correct notes', async () => {
      const a = arp('A$2:m');
      const notes = chord('A$2:m');
      for(let note of notes) {
        const next = a();
        expect(next).toEqual(note);
      }
    });


    test('applies options', async () => {
      const a = arp('C4:M', {d: 1, a: 2, r: 3});

      for(let i = 0; i < 3; i++) {
        const next = a() as Note;
        expect(next.duration).toEqual(1);
        expect(next.attack).toEqual(2);
        expect(next.release).toEqual(3);
      }
    });

  });


  describe('transpose', () => {

    test('defaults to octave', async () => {
      const note = C4();
      const expected = C5();

      const transposed = unwrap(transpose(note));

      expect(transposed).toEqual(expected);
    });

    test('transposes a chord', async () => {
      const notes = chord('C4:M');
      const expected = chord('C5:M');

      const transposed = transpose(notes);

      expect(transposed).toEqual(expected);
    });

    test('transposes semitones', async () => {
      const note = C4();
      const expected = E4();

      const transposed = unwrap(transpose(note, 4));

      expect(transposed).toEqual(expected);
    });


  });

});

