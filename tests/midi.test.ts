
import {jest} from '@jest/globals'

import { Midi } from '../src/midi';
import { chord } from '../src/note';

require('../src/globals');

let _bpm = 120;

globalThis.bpm = (set?: number) => {
  if (set) {
    _bpm = set;
  }
  return _bpm;
}


describe('test', () => {

    // beforeAll( async () => {
    //     console.log('beforeAll start')
    //     await midi.setup();
    //     console.log('beforeAll end')
    // });

    // // afterAll( async () => {
    // //     await midi.stop();
    // // });

    // test('selectOutput', async () => {
    //     midi.selectOutput(0);
    //     midi.selectOutput('LoopMidi');
    //     midi.selectOutput(99);
    // });

    describe('play', () => {

      const channel = 1;

      test('play', async () => {
        let midi = Midi();

        midi.play(channel, D);
        midi.play(channel, C);

        midi.play(10, E);

        expect(midi.buffer).toHaveProperty('1')
        expect(midi.buffer).toHaveProperty('10')

        const notes = midi.buffer[1];
        const note = notes[0];
        expect(notes).toHaveLength(2);
        expect(note.name).toEqual('D');
      });

      test('play chord', async () => {
        let midi = Midi();

        var c = chord('C4:M');
        midi.play(channel, c);

        expect(midi.buffer).toHaveProperty('1')

        const notes = midi.buffer[1];
        const note = notes[0];
        expect(notes).toHaveLength(3);
        expect(note.name).toEqual('C');
      });

      test('clear', async () => {
        let midi = Midi();

        midi.play(channel, D);
        midi.play(channel, C);

        const notes = midi.buffer[1];

        expect(midi.buffer).toHaveProperty('1')
        expect(notes).toHaveLength(2);

        midi.clear();
        expect(midi.buffer).not.toHaveProperty('1')
      });

      describe('options', () => {
        test('converts duration to ms', async () => {

          let midi = Midi();

          midi.play(channel, C({d: 1}));
          const notes = midi.buffer[1];
          const note = notes[0];
          expect(note.duration).toEqual(125);
        });

        test('converts attack and release to midified values', async () => {

          let midi = Midi();

          midi.play(channel, C({a: 1, r: z}));
          const notes = midi.buffer[1];
          const note = notes[0];
          expect(note.rawRelease).toEqual(127);
          expect(note.rawAttack).toEqual(4);
        });
      });

  });

  describe('control change', () => {
    test('adds control to buffer', async () => {

      let midi = Midi();

      midi.control(1, 10, z);

      expect(midi.controlBuffer).toHaveProperty('1');

      const controls = midi.controlBuffer[1];
      const control = controls[0];
      expect(control.value).toEqual(127);
    });

    test('adds many controls to buffer', async () => {

      let midi = Midi();

      midi.control(1, 10, z);
      midi.control(2, 10, z);

      expect(midi.controlBuffer).toHaveProperty('1');
      expect(midi.controlBuffer).toHaveProperty('2');
    });

    test('clears controls from buffer', async () => {

      let midi = Midi();

      midi.control(1, 10, z);
      midi.control(2, 10, z);

      midi.clear();
      expect(midi.controlBuffer).not.toHaveProperty('1');
      expect(midi.controlBuffer).not.toHaveProperty('2');
    });

    test.only('only adds control to buffer once', async () => {

      let midi = Midi();

      midi.control(1, 10, z);
      midi.control(1, 10, z);

      expect(midi.controlBuffer).toHaveProperty('1');

      const controls = midi.controlBuffer[1];
      expect(controls.length).toBe(1);
    });


  });
});
