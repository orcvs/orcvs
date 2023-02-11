
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

      test('push/pull', async () => {
        var buffer: Buffer = []; 
        let midi = Midi(buffer);
        
        midi.push(channel, octave, value, attack, duration);
        midi.push(channel, octave, 'D', attack, duration);
        expect(buffer).toHaveLength(2);

        midi.pull(0);
        expect(buffer).toHaveLength(1);

        const { note } = buffer[0];
        expect(note.midi().name).toEqual('D');
      });


      test.only('on/off', async () => {              
          var buffer: Buffer = []; 
          let midi = Midi(buffer);
          await midi.setup();
          await midi.selectOutput(0);
                            
          midi.push(channel, octave, value, attack, duration);
          var { note } = buffer[0];

          expect(note.played()).toBeFalsy();
  
          midi.tick();
          await new Promise((r) => setTimeout(r, 500));
 
          expect(note.played()).toBeTruthy();
          expect(note.doOff()).toBeFalsy(); 

          midi.tick();
          await new Promise((r) => setTimeout(r, 500));
          expect(note.doOff()).toBeFalsy(); 

          midi.tick();
          await new Promise((r) => setTimeout(r, 500));
          expect(note.doOff()).toBeFalsy(); 

          midi.tick();
          await new Promise((r) => setTimeout(r, 500));
          expect(note.doOff()).toBeTruthy(); 
          
          await WebMidi.disable();
      });
  });
}); 
