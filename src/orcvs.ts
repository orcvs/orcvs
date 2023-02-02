import { Clock } from './clock';
import { Midi } from './midi'

const BANG = '!'

type Callback = () => void

export class Orcvs {
  clock: Clock;
  midi: Midi;
  patterns: Pattern[] = [];

  constructor() { 
    this.clock = new Clock(this.tick);
    this.midi = new Midi();
  }

  async init() {
    await this.midi.setup();
    this.midi.selectOutput('LoopMidi');
  }

  async stop() {
    await this.midi.stop();
  }

  bang(pattern: Pattern | string, fn: Callback) {
    if (typeof pattern === 'string'){
      pattern = ptn(pattern, fn);
    }
    this.patterns.push(pattern);
  }

  play(channel: number, octave: number, note: string, attack: number, duration: number) {    
    this.midi.push(channel, octave, note, attack, duration);
  }

  tick() {
    this.run_patterns();
    this.midi.tick();
  }

  run_patterns() {
    const frame = this.clock.frame;
    for (const pattern of this.patterns) {
      // console.log('run_patterns', { frame });
      pattern.bang(frame);
  
    }
  }
}

export class Pattern {
  ptn: string[]
  fn: Callback;
  
  constructor(str: string, fn: Callback) {
      this.ptn = [...str]
      this.fn = fn;
  }

  shouldBang(f: number) {     
    const idx = f % this.ptn.length;
    return this.ptn[idx] === BANG
  }

  bang(f: number) {   
    // console.log('patterns.bang', { f });
    // console.log('patterns.bang', this.fn);
    // console.log('patterns.bang', this.shouldBang(f));
    // if (this.fn) {  
      if (this.shouldBang(f)) {          
        this.fn();
      }      
    // }    
  }

}

export function ptn(str: string, fn: Callback): Pattern {         
    return new Pattern(str, fn);
}
