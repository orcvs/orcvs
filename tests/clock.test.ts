
import {jest} from '@jest/globals'

import { Clock, handler} from '../src/clock';

// Handy reference
// 60 bpm  - 250ms
// 100 bpm - 150ms
// 120 bpm - 125ms
// 150 bpm - 100ms
// 200 bpm - 75ms

describe('clock', () => {

    afterEach( () => {
        jest.restoreAllMocks();
    });

    // test('touch', async () => {
         
    //     const timer = new Timer();
    //     const clock = Clock(tick);

    //     expect(clock.frame()).toEqual(0);

    //     for (let i=1; i < 10; i++) {
    //         clock.touch();
    //         expect(clock.frame()).toEqual(i);
    //     }

    // });

    // test('start/stop with worker', async () => {
    //     const mock = jest.fn();
    //     const clock = Clock(mock);
    //     // console.log(clock)

    //     clock.start();

    //     await new Promise((r) => setTimeout(r, 200));
    //     expect(mock).toHaveBeenCalledTimes(1);
       
    //     // console.log('stop')
    //     clock.stop();
    //     await new Promise((r) => setTimeout(r, 100));
    //     expect(mock).toHaveBeenCalledTimes(1);

    //     // mock.mockClear();
    //     // console.log('start')
    //     clock.start();
    //     await new Promise((r) => setTimeout(r, 200));
    //     expect(mock).toHaveBeenCalledTimes(2);
        
    //     clock.stop();
    // });

    test.only('change bpm', async () => {

        const mock = jest.fn();
        const clock = Clock(mock);
        await clock.start();

        // console.log("HEEELLLOOO")
        // await new Promise((r) => setTimeout(r, 1000));
        // await clock.stop();        
        // console.log("END")
        
        await new Promise((r) => setTimeout(r, 200));
        expect(mock).toHaveBeenCalled();
        expect(mock).toHaveBeenCalledTimes(1);
       
        mock.mockClear();
        await clock.setBpm(150);
        await new Promise((r) => setTimeout(r, 250));
        expect(mock).toHaveBeenCalled();
        expect(mock).toHaveBeenCalledTimes(2);
        
        await clock.stop();
    });
 
}); 
