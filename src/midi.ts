import { WebMidi, Output, Note} from 'webmidi';
import { Computable, compute } from './library';
import { Chord } from './note';
import { Logger } from "./logger";

// type Note = { channel: number, octave: number, note: string, attack: number, duration: number }

const logger = Logger.child({
  source: 'Midi'
});

// export type Buffer = { channel: number, note: Note[] }
// export type Buffer = { channel: number, note: MidiNote }[] 

export type Buffer = { [channel: number] : Note[] };

export function Midi() {
  let output: Output | null = null;
  var _buffer: Buffer = {};

  const self = {
    clear,
    play,
    on,
    selectOutput,
    setup,
    stop,
    tick,
    get buffer() {      
      return _buffer;    
    }
  };

  // function off(id: number, note: Note) {
  //   // logger.debug('off');
  //   if (output) {
  //     let channel = output.channels[id];
  //     channel.sendNoteOff(note.midi());    
  //   }
  // }  
  
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
  
  function isChordOrNote(playable: Note | Chord) {
    return (playable instanceof Chord) || ( playable instanceof Note);
  }

  function play(channel: number, playable: Computable<Note | Chord>) {   

    logger.debug({channel, playable});
    playable = compute(playable);

    if (!isChordOrNote(playable)) {
      const msg = 'Value is not a Note or Chord';
      logger.error({msg, note: playable});
      throw new TypeError(msg);
    }    

    const notes = playable instanceof Chord ? playable.notes : [playable];
    logger.debug({notes, playable})
    push(channel, notes );
  }

  function push(channel: number, note: Note | Note[]) {
    _buffer[channel] = (_buffer[channel] || []).concat(note);
    // logger.debug({_buffer})
  }

  function clear() {    
    _buffer = {};
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
    for (let idx in _buffer) {
      const notes = _buffer[idx];
      const channel = parseInt(idx);    
      on(channel, notes);
    }
    clear();
  }
  // pull(idx);
  return self
}
