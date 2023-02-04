
import { Worker } from 'node:worker_threads';

const MINUTE = 60000;
const FRAMES_PER_BEAT = 4;

const DEFAULT_BPM = 120;

type Tick = () => void;

export function Clock(callback: Tick) {
    var bpm = DEFAULT_BPM;
    var _frame = 0;
    var running = false;
    var timer!: Worker;

    function tick() {
      _frame++;
    }

    function reset() {
      _frame = 0;
    }

    function touch() {
      console.info('Clock', 'touch');
      stop();
      tick();
    }

    async function setBpm(n: number) {
      if (bpm != n) {
        bpm = n;
        if (running) {
          await start();
        }        
      }
    }

    function frame() {
      return _frame;
    }

    async function start() {
      console.info('Clock', 'start');
      await stop();        
      running = true;
      setTimer();
    }

    async function stop() {
      console.info('Clock', 'stop');
      if (timer !== undefined) {
        await timer.terminate();        
        running = false;
      }
    }

    function ms_per_beat() {
      return ( MINUTE  / bpm) / FRAMES_PER_BEAT;
    }

    async function setTimer() {
      console.info('Clock', 'setTimer');

      return new Promise((resolve, reject) => { 
          timer = new Worker(WORKER_SCRIPT, { eval: true });     
          timer.on('message', () => {           
            tick();           
            callback();
          });    

          const ms = ms_per_beat();
          timer.postMessage(ms);
          // timer.on("error", () => {
          //     console.error('Clock', 'timer');
          // });
      
          // timer.on("exit", () => {
          //     // threads.delete(worker);
          //     console.info('Clock', 'timer/exit');
          //     return resolve;
          // });
      });
    }

    return {
      frame,
      setBpm,
      start,
      stop,
      touch,
      tick
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
