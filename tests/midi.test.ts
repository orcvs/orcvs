
import {jest} from '@jest/globals'

import { Midi } from '../src/midi';
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
    

    test('selectOutput', async () => {
    
        let spyOn = jest.spyOn(Midi.prototype, 'on');
        let spyOff = jest.spyOn(Midi.prototype, 'off');

        let midi = new Midi();
           
        const channel = 1;
        const octave = 2;
        const value = 'C';
        const attack = 0.5;
        let duration = 4;

        midi.push(channel, octave, value, attack, duration);
        const { note } = midi.buffer[0];

        for (let i = 4; i >= 0; i--) {            
            midi.tick()
            expect(spyOn).toHaveBeenCalledTimes(1);    
        }
        
        expect(note.duration).toEqual(0);
        expect(spyOff).toHaveBeenCalledTimes(1);    
    });

}); 
