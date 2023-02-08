import { WebMidi, Output, Note as MidiNote} from 'webmidi';

import { Logger } from "./logger";

// type Note = { channel: number, octave: number, note: string, attack: number, duration: number }

const logger = Logger.child({
  source: 'Midi'
});

class Note extends MidiNote {
    played = false;

    tick() {
      if (this.duration !== 0) {
        this.duration--;
      }     
    }

    play() {
      this.played = true;
    }

    isPlayed() {
      return this.played;
    }
}

export type Buffer = { channel: number, note: Note }[] 

export function Midi(buffer: Buffer = []) {
  var output: Output | null = null;
  
  function off(id: number, note: Note) {
    if (output) {
      logger.debug('off');
      let channel = output.channels[id];
      // channel.sendNoteOff(note);      
    }
  }  
  
  function on(id: number, note: Note) {
    if (output) {
      logger.debug('on');
      let channel = output.channels[id];
      // channel.playNote(note, { duration: note.duration });
      channel.sendNoteOn(note);
    }
  }
    

  function push(channel: number, octave: number, note: string, attack: number, duration: number) {          
    const value = note + octave
    buffer.push({ channel,  note: new Note(value, { attack, duration }) });
  }

  function pull(idx: number) {
    buffer.splice(idx, 1);
  }

  function selectOutput(selected: number | string) {
    if (typeof selected === "number") {
      output = WebMidi.outputs[selected];
      if (!output) {
        logger.warn(`Unknown device with index: ${selected}`);
      }
    }

    if (typeof selected === "string") {
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
      // console.error('ERROR');
      logger.error(error);
    }
    logger.info('WebMidi enabled');
  }  

  async function stop() {
    // if (this.output) {
    //   // const channels = this.buffer.map( ({id, _}) => id);
    //   // this.output.sendAllSoundOff({ channels });
    // }
    logger.info('stop');
    await WebMidi.disable();
    logger.info('WebMidi disabled');
  }

  async function tick() {    
    // console.info('MIDI', 'tick');
    for (let idx = 0; idx < buffer.length; idx++) {
      const { channel, note } = buffer[idx];

      if (!note.isPlayed()) {
        on(channel, note);
        note.play();
      }      
   
      if(note.duration === 0) {
        off(channel, note)
        pull(idx);
      }

      note.tick();
    }
  }

  return {
    off,
    on,
    push,
    pull,
    selectOutput,
    setup,
    stop,
    tick,
  }

}
