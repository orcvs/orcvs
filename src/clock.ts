
import { Worker } from 'node:worker_threads';

import { Logger } from "./logger";

export const MINUTE = 60000;
export const FRAMES_PER_BEAT = 4;

const DEFAULT_BPM = 110;

type TickCallback = (frame: number) => void;

const logger = Logger.child({
  source: 'Clock'
});

export function Clock(callback: TickCallback) {
    var bpm = DEFAULT_BPM;
    var frame = 0;
    var running = false;
    var timer!: Worker;

    function tick() {
      frame = frame + 1;    
      // logger.debug({ tick: frame });            
      callback(frame);
    }

    function touch() {
      // logger.debug('touch');
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
      frame = 0;
    }

    async function start() {
      logger.debug('start');
      await stop();        
      running = true;
      await setTimer();
      
      // logger.debug('postmessage');
      const ms = msPerBeat();
      timer.postMessage(ms);

      // logger.debug('after start');
    }

    async function stop() {
      logger.debug('stop');
      if (timer !== undefined) {
        await timer.terminate();        
        running = false;
      }
    }

    function msPerBeat() {
      return ( MINUTE  / bpm) / FRAMES_PER_BEAT;
    }

    // function bpm() {
    //   return bpm;
    // }

    async function setTimer() {
      logger.debug('setTimer');

      // return new Promise((resolve, reject) => { 
          timer = new Worker(WORKER_SCRIPT, { eval: true });     
          timer.on('message', () => {           
            tick(); 
          });    
          // timer.on("error", () => {
          //     logger.error('timer');
          // });
      
          // timer.on("exit", () => {
          //     // threads.delete(worker);
          //     logger.info('timer/exit');
          //     return resolve;
          // });
      // });
    }


    return {
      reset,
      setBPM,
      start,
      stop,
      touch,
      tick,
      get bpm() {
        return bpm;
      },
      get frame() {
        return frame;
      },
      get running() {
        return running;
      }
    }
}


function msPerBeat() {
  return ( MINUTE  / bpm) / FRAMES_PER_BEAT;
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
