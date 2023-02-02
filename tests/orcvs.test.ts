
import {jest} from '@jest/globals'

import { Orcvs } from '../src/orcvs';


describe.skip('codes', () => {

    test('bang every', async () => {
        const orcvs = new Orcvs();
        
       const mock = jest.fn();

        orcvs.bang('!', () => {
            console.log('HELLO');
            mock();
        });
        
        orcvs.tick();
        expect(mock).toHaveBeenCalled();

        orcvs.tick();
        expect(mock).toHaveBeenCalledTimes(2);
   });
    

   test('bang every other', async () => {
        const orcvs = new Orcvs();
        
        const mock = jest.fn();

        orcvs.bang('!.', () => {
            mock();
        });
        
        orcvs.tick();
        orcvs.clock.frame = 1;

        orcvs.tick();
        orcvs.clock.frame = 2;
        expect(mock).toHaveBeenCalled();

        orcvs.tick();
        orcvs.clock.frame = 3;
        expect(mock).toHaveBeenCalledTimes(2);
    });
 

    // test('bang', async () => {
    //     const orcvs = new Orcvs();
    //     // await orcvs.init();

    //     let channel = orcvs.midi.output?.channels[1];
    //     // channel?.playNote('C3', {attack: 0.5, duration: 500} )
    //     // channel?.sendNoteOn('C3', {attack: 0.5} )
    //     // await new Promise((r) => setTimeout(r, 250));
    //     // channel?.sendNoteOff('C3');
        
    //     // orcvs.bang('!.......', () => {
    //     //     console.log('HELLO');
    //     //     orcvs.play(1, 3, 'C', 0.5, 1);
    //     //     // orcvs.play(1, 2, 'E', 0.5, 0.5);
    //     // });
        
    //     // for (let f = 0; f < 2; f++) {
    //     //     orcvs.run();
    //     //     orcvs.clock.tick();
    //     // }

    //     // await new Promise((r) => setTimeout(r, 15000));
    //     await new Promise((r) => setTimeout(r, 5000));

    //     await orcvs.stop();
    //     await new Promise((r) => setTimeout(r, 5000));
    // });

}); 
