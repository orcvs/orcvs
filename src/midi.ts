import { WebMidi, Output, Note as MidiNote } from 'webmidi';
import { Computable, compute } from './sequence';
import { midify, msPerBeat, wrap } from './library';
import { Cache } from './memoize';
import { Note } from './note';
import { Logger } from "./logger";

const logger = Logger.child({
  source: 'Midi'
});

export type ControlChange = { controller: string | number, value: number};
export type Buffer = { [channel: number] : MidiNote[] };
export type ControlBuffer = { [channel: number] : ControlChange[] };


export function Midi() {
  let output: Output | null = null;
  let buffer: Buffer = {};
  let controlBuffer: ControlBuffer = [];

  let controlCache = Cache();

  function clear() {
    buffer = {};
    controlBuffer = [];
  }

  function control(channel: number, controller: string | number, value: Computable<number>) {
    value = toMidiValue(value) as number;
    pushControl(channel, { controller, value})
  }

  function pause() {
    if (output) {
      controlCache.clear();
      for (const channel of output.channels) {
        if (channel) {
          // channel.sendAllNotesOff();
          channel.sendAllSoundOff();
        }
      }
    }
  }

  function play(channel: number, playable: Computable<Note> | Note[]) {
    playable = wrap(compute(playable));

    const notes: MidiNote[] = [];
    for(let note of playable) {
      const { value, duration, attack, release } = note;
      const opts = {duration: toMs(duration), rawAttack: toMidiValue(attack), rawRelease: toMidiValue(release)}
      notes.push( new MidiNote(value, opts) )
    }

    push(channel, notes);
  }

  function push(channel: number, note: MidiNote | MidiNote[]) {
    buffer[channel] = (buffer[channel] || []).concat(note);
  }

  function pushControl(channel: number, control: ControlChange) {
    const { controller, value } = control;
    const key = `${channel}/${controller}`;
    const current = controlCache.get(key);
    if (current !== value) {
      (controlBuffer[channel] ||= []).push(control);
      controlCache.set(key, value);
    }
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

  function send(channelId: number, notes: MidiNote[]) {
    if (output) {
      let channel = output.channels[channelId];
      channel.playNote(notes);
    }
  }

  function sendControl(channelId: number, controls: ControlChange[]) {
    // logger.debug({source: 'sendControl', controller, value});
    if (output) {
      let channel = output.channels[channelId];
      for (let { controller, value } of controls) {
        channel.sendControlChange(controller, value);
      }
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
    controlCache.clear();
    await WebMidi.disable();
  }

  function tick() {
    for (let idx in controlBuffer) {
      const control = controlBuffer[idx];
      const channel = parseInt(idx);
      sendControl(channel, control);
    }

    for (let idx in buffer) {
      const notes = buffer[idx];
      const channel = parseInt(idx);
      send(channel, notes);
    }

    clear();
  }

  return {
    clear,
    control,
    play,
    pause,
    send,
    sendControl,
    selectOutput,
    setup,
    stop,
    tick,
    get buffer() {
      return buffer;
    },
    get controlBuffer() {
      return controlBuffer;
    }
  };
}

function toMs(duration?: Computable<number>) {
  if (duration) {
    return compute(duration) * msPerBeat();
  }
}

function toMidiValue(value?: Computable<number>) {
  return midify(compute(value));
}