
import {jest} from '@jest/globals'

import { Midi } from '../src/midi';
import { Chord, chord } from '../src/note';
import { WebMidi, Output, Note } from 'webmidi';

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

        expect(notes).toHaveLength(2);
        expect(notes[0].name).toEqual('D');
      });

      test('play chord', async () => {
        let midi = Midi();
            
        var c = chord('C4:M');
        midi.play(channel, c);
  
        expect(midi.buffer).toHaveProperty('1')
        
        const notes = midi.buffer[1];

        expect(notes).toHaveLength(3);
        expect(notes[0].name).toEqual('C');
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



      // test.only('group', async () => {
      //   var buffer = [{channel: 1, note: A},{channel: 2, note:A}, {channel: 1, note: B }]; 
        
      //   var result = buffer.reduce( (acc: any, obj) => {
      //     const key = obj.channel          
      //     const curGroup = acc[key] ?? [];
      
      //     return { ...acc, [key]: [...curGroup, obj] };
      //   }, {}  )

      //   console.log(result);
      // }); 


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
