
import {jest} from '@jest/globals'

import { Midi, Buffer } from '../src/midi';
import { WebMidi, Output } from 'webmidi';

describe.skip('test', () => {
    
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

      test('push', async () => {
        var buffer: Buffer = []; 
        let midi = Midi(buffer);
        
        midi.push(channel, octave, value, attack, duration);
        midi.push(channel, octave, 'D', attack, duration);
        expect(buffer).toHaveLength(2);

        const { note } = buffer[1];
        expect(note.midi().name).toEqual('D');
      });

      test('clear', async () => {
        var buffer: Buffer = []; 
        let midi = Midi(buffer);
        
        midi.push(channel, octave, value, attack, duration);
        midi.push(channel, octave, 'D', attack, duration);
        expect(buffer).toHaveLength(2);
        
        var { note } = buffer[0];
        note.play();
        note.stop();

        midi.clear();
        expect(buffer).toHaveLength(1);

        var { note } = buffer[0];
        note.play();
        note.stop();

        midi.clear();
        expect(buffer).toHaveLength(0);
      });

      test('on/off', async () => {              
          var buffer: Buffer = []; 
          let midi = Midi(buffer);
          await midi.setup();
          await midi.selectOutput(0);
                            
          midi.push(channel, octave, value, attack, duration);
          var { note } = buffer[0];

          expect(note.played()).toBeTruthy();
          expect(note.playing()).toBeTruthy();
          expect(note.shouldOff()).toBeFalsy(); 
  
          midi.tick();
          await new Promise((r) => setTimeout(r, 500));
 
          expect(note.played()).toBeTruthy();
          expect(note.playing()).toBeTruthy();
          expect(note.shouldOff()).toBeFalsy(); 

          midi.tick();
          await new Promise((r) => setTimeout(r, 500));
          
          expect(note.played()).toBeTruthy();
          expect(note.playing()).toBeTruthy();
          expect(note.shouldOff()).toBeFalsy(); 

          midi.tick();
          await new Promise((r) => setTimeout(r, 500));
      
          expect(note.played()).toBeTruthy();
          expect(note.playing()).toBeTruthy();
          expect(note.shouldOff()).toBeFalsy(); 

          midi.tick();
          await new Promise((r) => setTimeout(r, 500));
          expect(note.shouldOff()).toBeTruthy(); 
          expect(note.playing()).toBeFalsy();
          
          await WebMidi.disable();
      });
  });
}); 
