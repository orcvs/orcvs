import { WebMidi, Output, Note as MidiNote} from 'webmidi';
import { Logger } from "./logger";

// type Note = { channel: number, octave: number, note: string, attack: number, duration: number }

const logger = Logger.child({
  source: 'Midi'
});

interface Note {
  midi: () => MidiNote;
  play: () => void;
  played: () => boolean;
  playing: () => boolean;
  shouldOff: () => boolean;
  shouldPlay: () => boolean;
  stop: () => void;
  stopped: () => boolean;
  tick: () => void;
  value: string;
}

export function note(value: string, attack: number, duration: number):Note {
    var _played = false;
    var _stopped = false;
    // let midi = midi();

    const self = {
      midi,
      shouldOff,
      play,
      played,
      playing,
      shouldPlay,
      stop,
      stopped,
      tick,
      value,
    };



    function play() {
      _played = true;
    }
    
    function shouldPlay() {
      return !_played;
    }
    
    function played() {
      return _played;
    }
    
    function playing() {
      return _played && !_stopped;
    }

    function midi() {
      return new MidiNote(value, { rawAttack: attack });
    }

    function shouldOff() {
      return duration === 0
    }

    function stop() {
      _stopped = true;
    }
    
    function stopped() {
      return _stopped;
    }

    function tick() {
      if (duration !== 0) {
        duration--;
      }     
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
    clear,
    selectOutput,
    setup,
    stop,
    tick,
  };

  function off(id: number, note: Note) {
    logger.debug('off');
    if (output) {
      let channel = output.channels[id];
      channel.sendNoteOff(note.midi());    
      note.stop();  
    }
  }  
  
  function on(channelId: number, note: Note) {
    logger.debug('on');
    if (output) {
      let channel = output.channels[channelId];
      // channel.playNote(note, { duration: note.duration });
      channel.sendNoteOn(note.midi());
      note.play();
    }
  }
    
  function push(channel: number, octave: number, value: string, attack: number, duration: number) {    
    logger.debug('push');      
    value = value + octave
    buffer.push({ channel,  note: note(value, attack, duration) });
  }

  function clear() {    
    buffer = buffer.filter( ({ note }) => note.playing() )
    // let deleted = buffer.splice(idx, 1);
    // logger.debug({ deleted });   
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
      // console.error('ERROR');
      logger.error(error);      
    }    

    // WebMidi.outputs.forEach(output => logger.info(output.manufacturer, output.name));

    logger.info({Outputs: WebMidi.outputs.map( o => o.name) });
    // for (let output of WebMidi.outputs) {
    //   logger.info(output.name);
    // }
    
    logger.info('WebMidi enabled');
  }  

  async function stop() {
    // if (this.output) {
    //   // const channels = this.buffer.map( ({id, _}) => id);
    //   // this.output.sendAllSoundOff({ channels });
    // }
    await WebMidi.disable();
  }

  function tick(f?: number) {       
    logger.debug({ tick: f });
    logger.debug({ buffer });
    for (let idx = 0; idx < buffer.length; idx++) {
      const { channel, note } = buffer[idx];
      logger.debug({ idx, note, played: note.played() });

      if (note.shouldPlay()) {
        on(channel, note);
        logger.debug('played');        
      }      

      if(note.shouldOff()) {        
        off(channel, note)      
      }
      note.tick();
    }
    clear();
  }
  // pull(idx);
  return self
}
