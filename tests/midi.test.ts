
import {jest} from '@jest/globals'

import { Midi, Buffer } from '../src/midi';
import { WebMidi, Output } from 'webmidi';

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

      test.only('push/pull', async () => {
        var buffer: Buffer = []; 
        let midi = Midi(buffer);
        
        midi.push(channel, octave, value, attack, duration);
        midi.push(channel, octave, 'D', attack, duration);
        expect(buffer).toHaveLength(2);

        midi.pull(0);
        expect(buffer).toHaveLength(1);

        const { note } = buffer[0];
        expect(note.name).toEqual('D');
      });

      test('on/off', async () => {
              
          var buffer: Buffer = []; 
          let midi = Midi(buffer);
                  
          let spyOn = jest.spyOn(midi, 'on');
          let spyOff = jest.spyOn(midi, 'off');
        
          midi.push(channel, octave, value, attack, duration);
          const { note } = buffer[0];

          for (let i = 4; i >= 0; i--) {            
              midi.tick()
              expect(spyOn).toHaveBeenCalledTimes(1);    
          }
          
          expect(note.duration).toEqual(0);
          expect(spyOff).toHaveBeenCalledTimes(1);    
      });
  });
}); 
