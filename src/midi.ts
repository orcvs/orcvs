import { WebMidi, Output, Note as MidiNote} from 'webmidi';
import { P } from './library';

import { Logger } from "./logger";

// type Note = { channel: number, octave: number, note: string, attack: number, duration: number }

const logger = Logger.child({
  source: 'Midi'
});

interface Note {
  midi: () => MidiNote;
  doOff: () => boolean;
  play: () => void;
  played: () => boolean;
  tick: () => void;
}

export function note(value: string, attack: number, duration: number):Note {
    var _played = false;
    // let midi = midi();

    const self = {
      midi,
      doOff,
      play,
      played,
      tick
    };

    function tick() {
      if (duration !== 0) {
        duration--;
      }     
    }

    function play() {
      _played = true;
    }
    
    function played() {
      return _played;
    }

    function midi() {
      return new MidiNote(value, { attack, duration });
    }

    function doOff() {
      return duration === 0
    }

    return self;
}

export type Buffer = { channel: number, note: Note }[] 

export function Midi(buffer: Buffer = []) {
  var output: Output | null = null;
  
  const self = {
    off,
    on,
    push,
    pull,
    selectOutput,
    setup,
    stop,
    tick,
  };

  function off(id: number, note: Note) {
    if (output) {
      let channel = output.channels[id];
      channel.sendNoteOff(note.midi());      
    }
  }  
  
  function on(channelId: number, note: Note) {
    if (output) {
      let channel = output.channels[channelId];
      // channel.playNote(note, { duration: note.duration });
      channel.sendNoteOn(note.midi());
    }
  }
    
  function push(channel: number, octave: number, value: string, attack: number, duration: number) {          
    value = value + octave
    buffer.push({ channel,  note: note(value, attack, duration) });
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
    await WebMidi.disable();
  }

  function tick() {    
    // logger.debug('MIDI', 'tick');
    for (let idx = 0; idx < buffer.length; idx++) {
      const { channel, note } = buffer[idx];

      if (!note.played()) {
        on(channel, note);
        note.play();
      }      

      if(note.doOff()) {
        off(channel, note)
        pull(idx);
      }

      note.tick();
    }
  }

  return self
}
