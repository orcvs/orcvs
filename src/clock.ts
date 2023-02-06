
import { Worker } from 'node:worker_threads';

import { Logger } from "./logger";

const MINUTE = 60000;
const FRAMES_PER_BEAT = 4;

const DEFAULT_BPM = 120;

type TickCallback = () => void;

const logger = Logger.child({
  source: 'Clock'
});

export function Clock(callback: TickCallback) {
    var bpm = DEFAULT_BPM;
    var _frame = 0;
    var running = false;
    var timer!: Worker;

    function tick() {
      _frame = _frame + 1;                
      callback();
    }

    function touch() {
      logger.debug('touch');
      stop();
      tick();
    }

    async function setBPM(n: number) {
      if (bpm != n) {  
        bpm = n;
        if (!running) {
          await start();
        }        
      }
    }
    
    function reset() {
      _frame = 0;
    }

    function frame() {
      return _frame;
    }

    async function start() {
      logger.debug('start');
      await stop();        
      running = true;
      setTimer();
    }

    async function stop() {
      logger.debug('stop');
      if (timer !== undefined) {
        await timer.terminate();        
        running = false;
      }
    }

    function ms_per_beat() {
      return ( MINUTE  / bpm) / FRAMES_PER_BEAT;
    }

    async function setTimer() {
      logger.debug('setTimer');

      return new Promise((resolve, reject) => { 
          timer = new Worker(WORKER_SCRIPT, { eval: true });     
          timer.on('message', () => {           
            tick(); 
          });    

          const ms = ms_per_beat();
          timer.postMessage(ms);
          // timer.on("error", () => {
          //     logger.error('timer');
          // });
      
          // timer.on("exit", () => {
          //     // threads.delete(worker);
          //     logger.info('timer/exit');
          //     return resolve;
          // });
      });
    }

    return {
      frame,
      reset,
      setBPM,
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
