import { WebMidi, Output, Note as MidiNote} from 'webmidi';

// type Note = { channel: number, octave: number, note: string, attack: number, duration: number }

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

export class Midi {
  output: Output | null = null;
  buffer: { id: number, note: Note }[] = []
  
  constructor() {
  }

  async setup() {
    console.log('WebMidi setup');
    try {
      await WebMidi.enable();
    } catch(error) {
      // console.error('ERROR');
      console.error(error);
    }
    console.info('MIDI', 'WebMidi enabled');
  }
  
  tick() {    
    console.info('MIDI', 'tick');
    for (let i in this.buffer) {
      const { id, note } = this.buffer[i];

      if (!note.isPlayed()) {
        this.on(id, note);
        note.play();
      }      
   
      if(note.duration === 0) {
        this.off(id, note)
        this.pull(id)
      }

      note.tick();
    }
  }
  
  on(id: number, note: Note) {
    if (this.output) {
      console.info('MIDI', 'on', note);
      let channel = this.output.channels[id];
      // channel.playNote(note, { duration: note.duration });
      channel.sendNoteOn(note);
    }
  }
  
  off(id: number, note: Note) {
    if (this.output) {
      console.info('MIDI', 'off', note);
      let channel = this.output.channels[id];
      // channel.sendNoteOff(note);      
      console.info('MIDI', 'off', note);
    }
  }

  push(channel: number, octave: number, note: string, attack: number, duration: number) {          
    const value = note + octave
    this.buffer.push({ id: channel,  note: new Note(value, { attack, duration }) });
  }

  pull(id: number) {
    delete this.buffer[id]
  }

  async stop() {
    // if (this.output) {
    //   // const channels = this.buffer.map( ({id, _}) => id);
    //   // this.output.sendAllSoundOff({ channels });
    // }
    await WebMidi.disable();
    console.info('MIDI', 'WebMidi disabled');
  }

  selectOutput(output: number | string) {
    if (typeof output === "number") {
      this.output = WebMidi.outputs[output];
      if (!this.output) {
        console.warn('MIDI', `Unknown device with index: ${output}`);
      }
    }

    if (typeof output === "string") {
      this.output = WebMidi.getOutputByName(output);
      if (!this.output) {
        console.warn('MIDI', `Unknown device with name: ${output}`);
      }
    }

    if (this.output) {
      console.info('MIDI', `Output Device: ${this.output?.name}`);
    }
  }
}
