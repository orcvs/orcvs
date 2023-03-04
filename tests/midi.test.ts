
import {jest} from '@jest/globals'

import { Midi } from '../src/midi';
import { chord } from '../src/note';

require('../src/globals');

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
      const octave = 2;
      const value = 'C';
      const attack = 0.5;
      let duration = 4;

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

      describe('control change', () => {
        test('adds control to buffer', async () => {

          let midi = Midi();

          midi.control(1, 10, z);

          expect(midi.controlBuffer).toHaveProperty('1');

          const controls = midi.controlBuffer[1];
          const control = controls[0];
          console.log(control);
          expect(control.value).toEqual(127);


          midi.clear();
          expect(midi.controlBuffer).not.toHaveProperty('1');
        });
      });

      // test('on/off', async () => {
      //     var buffer: Buffer = [];
      //     let midi = Midi(buffer);
      //     await midi.setup();
      //     await midi.selectOutput(0);

      //     midi.push(channel, octave, value, attack, duration);
      //     var { note } = buffer[0];

      //     expect(note.played()).toBeTruthy();
      //     expect(note.playing()).toBeTruthy();
      //     expect(note.shouldOff()).toBeFalsy();

      //     midi.tick();
      //     await new Promise((r) => setTimeout(r, 500));

      //     expect(note.played()).toBeTruthy();
      //     expect(note.playing()).toBeTruthy();
      //     expect(note.shouldOff()).toBeFalsy();

      //     midi.tick();
      //     await new Promise((r) => setTimeout(r, 500));

      //     expect(note.played()).toBeTruthy();
      //     expect(note.playing()).toBeTruthy();
      //     expect(note.shouldOff()).toBeFalsy();

      //     midi.tick();
      //     await new Promise((r) => setTimeout(r, 500));

      //     expect(note.played()).toBeTruthy();
      //     expect(note.playing()).toBeTruthy();
      //     expect(note.shouldOff()).toBeFalsy();

      //     midi.tick();
      //     await new Promise((r) => setTimeout(r, 500));
      //     expect(note.shouldOff()).toBeTruthy();
      //     expect(note.playing()).toBeFalsy();

      //     await WebMidi.disable();
      // });
  });
});
