
import {jest} from '@jest/globals'

import { Clock} from '../src/clock';

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

    test('touch', async () => {
        const mockCallback = jest.fn();
        const clock = Clock(mockCallback);

        expect(clock.frame).toEqual(0);

        for (let i=1; i < 10; i++) {
            clock.touch();
            expect(clock.frame).toEqual(i);           
            expect(mockCallback).toHaveBeenCalledTimes(i);
        }

    });

    test('start/stop with worker', async () => {
    const mockCallback = jest.fn();
    const clock = Clock(mockCallback);
    // console.log(clock)

    clock.start();

    await new Promise((r) => setTimeout(r, 250));
    expect(mockCallback).toHaveBeenCalledTimes(1);
    
    // console.log('stop')
    clock.stop();
    await new Promise((r) => setTimeout(r, 100));
    expect(mockCallback).toHaveBeenCalledTimes(1);

    clock.start();
    await new Promise((r) => setTimeout(r, 200));
    expect(mockCallback).toHaveBeenCalledTimes(2);
    
    clock.stop();
    });

    test('change bpm', async () => {

        const mockCallback = jest.fn();
        const clock = Clock(mockCallback);
        await clock.start();
        await new Promise((r) => setTimeout(r, 200));
        expect(mockCallback).toHaveBeenCalled();
        expect(mockCallback).toHaveBeenCalledTimes(1);
       
        mockCallback.mockClear();

        await clock.setBPM(150);
        await new Promise((r) => setTimeout(r, 300));
        expect(mockCallback).toHaveBeenCalled();
        expect(mockCallback).toHaveBeenCalledTimes(2);
        
        await clock.stop();
    });
 
}); 
