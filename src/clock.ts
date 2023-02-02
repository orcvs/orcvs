
import { Worker } from 'node:worker_threads';

const MINUTE = 60000;
const FRAMES_PER_BEAT = 4;

const DEFAULT_BPM = 120;

type Tick = () => void;

export class Clock {
    bpm = DEFAULT_BPM;
    frame = 0;
    running = false;
    timer!: Worker;
    callback: Tick;

    constructor(callback: Tick) {
        this.bpm = DEFAULT_BPM;
        this.callback = callback;
    }

    tick() {
        this.frame++ 
        this.callback();
    }

    reset() {
        this.frame = 0;
    }

    touch() {
        this.stop();
        this.tick();
    }

    setBpm(bpm: number) {
      if (this.bpm != bpm) {
        this.bpm = bpm;
        if (this.running) {
            this.start();
        }        
      }
    }

    setTimer() {
        this.timer = new Worker(WORKER_SCRIPT, { eval: true });
        this.timer.on('message', this.tick);
    }

    start() {
        this.stop();
        this.setTimer();
        this.running = true;
        const ms = this.ms_per_beat();
        this.timer.postMessage(ms);
    }

    ms_per_beat() {
       return ( MINUTE  / this.bpm) / FRAMES_PER_BEAT;
    }

    stop() {
        if (this.timer) {
            // console.log('TERMINATE');
            this.timer.terminate();
            // delete this.timer;
        }
    }
}

// Dedicated Worker Thread for more accurate timing
// Workerr runs an intervalTimer that callsback to parent
// Timer is FRAME INTERVAL as ms
const WORKER_SCRIPT = `
const { parentPort } = require('worker_threads');

${handler}

parentPort.on('message', handler);
`
export function handler(ms: number) {
    // @ts-expect-error
    setInterval(() => { parentPort.postMessage(true) }, ms);
}
