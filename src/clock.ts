
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
        console.info('Clock', 'new');
        console.info('Clock', 'new', callback);
        this.callback = callback;
    }

    tick() {
        // console.info('Clock', 'tick');
        // console.log('Clock', 'tick', this)
        // console.log('Clock', 'tick', this.callback)
        this.frame++ 
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

    async setTimer() {
        console.info('Clock', 'setTimer');
        // return (async () => {
        //     this.timer = new Worker(WORKER_SCRIPT, { eval: true });     
        //     this.timer.on('message', () => {
        //         this.tick();           
        //         this.callback();
        //     });    
        //     const ms = this.ms_per_beat();
        //     this.timer.postMessage(ms);
            
        // })().catch(console.error);

        return new Promise((resolve, reject) => { 
            this.timer = new Worker(WORKER_SCRIPT, { eval: true });     
            this.timer.on('message', () => {
                this.tick();           
                this.callback();
            });    

            this.timer.on("error", () => {
                console.error('Clock', 'timer');
            });
        
            this.timer.on("exit", () => {
                // threads.delete(worker);
                console.info('Clock', 'timer/exit');
                return resolve;
            });

            const ms = this.ms_per_beat();
            this.timer.postMessage(ms);
        });
    }

    async start() {
        console.info('Clock', 'start');
        this.stop();        
        this.running = true;
        await this.setTimer();
        // console.info('Clock', 'running', true);
    }

    ms_per_beat() {
       return ( MINUTE  / this.bpm) / FRAMES_PER_BEAT;
    }

    stop() {
      if (this.timer) {
        // console.log('TERMINATE');
        this.timer.terminate();        
        this.running = false;
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
