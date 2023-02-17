
import { Worker } from 'node:worker_threads';

import { Logger } from "./logger";

const MINUTE = 60000;
const FRAMES_PER_BEAT = 4;

const DEFAULT_BPM = 110;

type TickCallback = (frame: number) => void;

const logger = Logger.child({
  source: 'Clock'
});

export function Clock(callback: TickCallback) {
    var _bpm = DEFAULT_BPM;
    var _frame = 0;
    var _running = false;
    var _timer!: Worker;

    function tick() {
      _frame = _frame + 1;    
      // logger.debug({ tick: _frame });            
      callback(_frame);
    }

    function touch() {
      // logger.debug('touch');
      stop();
      tick();
    }

    async function setBPM(n: number) {
      if (_bpm != n) {  
        _bpm = n;
        if (!_running) {
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

    function running() {
      return _running;
    }
    async function start() {
      logger.debug('start');
      await stop();        
      _running = true;
      await setTimer();
      
      const ms = ms_per_beat();
      // logger.debug('postmessage');
      _timer.postMessage(ms);

      // logger.debug('after start');
    }

    async function stop() {
      logger.debug('stop');
      if (_timer !== undefined) {
        await _timer.terminate();        
        _running = false;
      }
    }

    function ms_per_beat() {
      return ( MINUTE  / _bpm) / FRAMES_PER_BEAT;
    }

    function bpm() {
      return _bpm;
    }

    async function setTimer() {
      logger.debug('setTimer');

      // return new Promise((resolve, reject) => { 
          _timer = new Worker(WORKER_SCRIPT, { eval: true });     
          _timer.on('message', () => {           
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
      bpm,
      frame,
      reset,
      running,
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
