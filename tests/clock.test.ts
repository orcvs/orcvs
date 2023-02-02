
import {jest} from '@jest/globals'

import { Clock, handler} from '../src/clock';

 // @ts-expect-error
global.parentPort = { postMessage: ()=>{} }; 

class Timer {

    postMessage(ms: number) {
        handler(ms);
    }

    on(_name: string , callback: any) {
        // @ts-expect-error
        parentPort.postMessage = callback;
    }

    terminate() {}
}

// Handy reference
// 60 bpm  - 250ms
// 100 bpm - 150ms
// 120 bpm - 125ms
// 150 bpm - 100ms
// 200 bpm - 75ms

describe.skip('clock', () => {
    const tick = () => {};

    afterEach( () => {
        jest.restoreAllMocks();
    });

    let bpm = 60;

    test('touch', async () => {
         
        const timer = new Timer();
        const clock = new Clock(tick);

        expect(clock.frame).toEqual(0);

        for (let i=1; i < 10; i++) {
            clock.touch();
            expect(clock.frame).toEqual(i);
        }

    });

    test('start/stop with worker', async () => {

        let spy = jest.spyOn(Clock.prototype, 'tick');

        const clock = new Clock(tick);
        // console.log('start')
        clock.start();

        await new Promise((r) => setTimeout(r, 200));
        expect(spy).toHaveBeenCalledTimes(1);
       
        // console.log('stop')
        clock.stop();
        await new Promise((r) => setTimeout(r, 100));
        expect(spy).toHaveBeenCalledTimes(1);

        // spy.mockClear();
        // console.log('start')
        clock.start();
        await new Promise((r) => setTimeout(r, 200));
        expect(spy).toHaveBeenCalledTimes(2);
        
        clock.stop();
    });

    test('change bpm', async () => {

        let spy = jest.spyOn(Clock.prototype, 'tick');

        const clock = new Clock(tick);
        clock.start();

        await new Promise((r) => setTimeout(r, 200));
        expect(spy).toHaveBeenCalled();
        expect(spy).toHaveBeenCalledTimes(1);
       
        spy.mockClear();
        clock.setBpm(150);
        await new Promise((r) => setTimeout(r, 250));
        expect(spy).toHaveBeenCalled();
        expect(spy).toHaveBeenCalledTimes(2);
        
        clock.stop();
    });


    test.skip('with worker', async () => {
        let spy = jest.spyOn(Clock.prototype, 'tick');

        const clock = new Clock(tick);

        clock.start();
        await new Promise((r) => setTimeout(r, 1000));
        expect(spy).toHaveBeenCalled();
    
        clock.stop();
    });


    test.skip('timer', async () => {
        
        jest.useFakeTimers();

        let spy = jest.spyOn(Clock.prototype, 'tick');

        let bpm = 60;
        const timer = new Timer();
        const clock = new Clock(tick);
        
        clock.setBpm(bpm)
        clock.setTimer = () => timer

        clock.start();
        jest.advanceTimersByTime(1000);

        expect(spy).toHaveBeenCalled();
        expect(spy).toHaveBeenCalledTimes(4);
    
        clock.stop();
    });
}); 
