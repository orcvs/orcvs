import { WebMidi, Output, Note} from 'webmidi';
import { Computable, compute, midify, msPerBeat } from './library';
import { Playable } from './note';
import { Logger } from "./logger";

const logger = Logger.child({
  source: 'Midi'
});

const MINUTE = 60000;
const FRAMES_PER_BEAT = 4;

export type Buffer = { [channel: number] : Note[] };

export function Midi() {
  let output: Output | null = null;
  let buffer: Buffer = {};

  function on(channelId: number, notes: Note[]) {
    // logger.debug('on');
    logger.debug({octaveOffset: WebMidi.octaveOffset})    
    if (output) {
      // logger.debug({note: typeof note })
      // logger.debug({note})
      // const notes = note instanceof Note ? [note] : note.notes;
      // // if (note instanceof Note) {
      // //   notes = 
      // // }
      // logger.debug({note: typeof note })
      // console.log({notes})
      let channel = output.channels[channelId];      
      channel.playNote(notes);    
      // channel.sendNoteOn(note.midi());
    }
  }
  
  function play(channel: number, playable: Computable<Playable>) {   
    // logger.debug({channel, playable});
    playable = compute(playable);

    const notes: Note[] = [];
    for(let note of playable.notes) {      
      const {value, duration, attack, release} = note;
      const opts = {duration: toMs(duration), rawAttack: toMidiValue(attack), rawRelease: toMidiValue(release)}
      notes.push( new Note(value, opts) ) 
    }
    
    push(channel, notes);
  }

  function push(channel: number, note: Note | Note[]) {
    buffer[channel] = (buffer[channel] || []).concat(note);
  }

  function clear() {    
    buffer = {};
  }

  function selectOutput(selected: number | string) {
    if (typeof selected === 'number') {
      output = WebMidi.outputs[selected];
      if (!output) {
        logger.warn(`Unknown device with index: ${selected}`);
      }
    }

    if (typeof selected === 'string') {
      output = WebMidi.getOutputByName(selected);
      if (!output) {
        logger.warn(`Unknown device with name: ${selected}`);
      }
    }

    if (output) {
      logger.info(`Output Device: ${output.name}`);
    }
  }

  async function setup() {
    logger.info('Midi.setup');
    try {
      await WebMidi.enable();
    } catch(error) {
      logger.error(error);      
    }    
    logger.info({Outputs: WebMidi.outputs.map( o => o.name) });
    logger.info('WebMidi enabled');
  }  

  async function stop() {
    await WebMidi.disable();
  }

  function tick(f?: number) {
    for (let idx in buffer) {
      const notes = buffer[idx];
      const channel = parseInt(idx);    
      on(channel, notes);
    }
    clear();
  }


  return {
    clear,
    play,
    on,
    selectOutput,
    setup,
    stop,
    tick,
    get buffer() {      
      return buffer;    
    }
  };
}

function toMs(duration: Computable<number> | undefined) {
  if (duration) {
    return compute(duration) * msPerBeat();
  }
}

function toMidiValue(value: Computable<number> | undefined) {
  if (value) {
    return midify(compute(value));
  }
}