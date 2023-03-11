export { Orcvs } from './orcvs'

import { Computable, Computer } from './library';
// import { Context } from './orcvs'
import { OnPulse, Pulsar } from './pulsar';
import { Note, Options } from './note';
import { euclid } from './algo';


declare var Pulsar: Pulsar;
declare var OnPulse: OnPulse;


declare global {

  var pattern: (str: string, on: OnPulse) => void;
  var lerp: (tofrom: number, to?: number, diff?: number) => Computer<number>;
  var cycle: (tofrom: number, to?: number, diff?: number) => Computer<number>;
  var wave: (tofrom: number, to?: number, diff?: number) => Computer<number>;
  var random: (tofrom: number, to?: number, diff?: number) => Computer<number>;
  var seq:  <T>(...sequence: readonly T[]) => Computer<T>

  var framesPerBeat: (set?: number) => number;
  var bpm: (set?: number) => number;
  var fpb: (set?: number) => number;

  var memoize: (key: string, ...args: any[]) => any;

  var chord: (chord: string, ...options: Options[]) => Note[];
  var note: (name: string) => Computer<Note>
  var arp: (value: string, ...options: Options[]) => Computer<Note>;

  var euclid: (steps?: number, beats?: number, rotateSteps?: number) => number[];

  // Defined in Orcvs and attached at runtime
  var play: (channel: number, note: Computable<Note>) => void;
  var output: (output: number | string) => void;

  // Aliases
  var ptn: (str: string, on: OnPulse) => void;
  var at: (str: string, on: OnPulse) => void;
  var at: (str: string, on: OnPulse) => void;
  var lrp: (tofrom: number, to?: number, diff?: number) => Computer<number>;
  var cyc: (tofrom: number, to?: number, diff?: number) => Computer<number>;
  var wav: (tofrom: number, to?: number, diff?: number) => Computer<number>;
  var rnd: (tofrom: number, to?: number, diff?: number) => Computer<number>;

  var crd: (chord: string, ...options: Options[]) => Note[];
  var nte: (name: string) => Computer<Note>

  var euc: (steps?: number, beats?: number, rotateSteps?: number) => number[];

  var ply: (channel: number, note: Computable<Note>) => void;
  var out: (output: number | string) => void;

  var ORCVS: string;
  var BANG: string;
  var UNBANG: string;

  var a: number;
  var b: number;
  var c: number;
  var d: number;
  var e: number;
  var f: number;
  var g: number;
  var h: number;
  var i: number;
  var j: number;
  var k: number;
  var l: number;
  var m: number;
  var n: number;
  var o: number;
  var p: number;
  var q: number;
  var r: number;
  var s: number;
  var t: number;
  var u: number;
  var v: number;
  var w: number;
  var x: number;
  var y: number;
  var z: number;

  var C2:   Computer<Note>;
  var C$2:  Computer<Note>;
  var D2:   Computer<Note>;
  var D$2:  Computer<Note>;
  var Db2:  Computer<Note>;
  var E2:   Computer<Note>;
  var E$2:  Computer<Note>;
  var Eb2:  Computer<Note>;
  var F2:   Computer<Note>;
  var F$2:  Computer<Note>;
  var G2:   Computer<Note>;
  var G$2:  Computer<Note>;
  var Gb2:  Computer<Note>;
  var A2:   Computer<Note>;
  var A$2:  Computer<Note>;
  var Ab2:  Computer<Note>;
  var B2:   Computer<Note>;
  var Bb2:  Computer<Note>;
  var C3:   Computer<Note>;
  var C$3:  Computer<Note>;
  var Cb3:  Computer<Note>;
  var D3:   Computer<Note>;
  var D$3:  Computer<Note>;
  var Db3:  Computer<Note>;
  var E3:   Computer<Note>;
  var E$3:  Computer<Note>;
  var Eb3:  Computer<Note>;
  var F3:   Computer<Note>;
  var F$3:  Computer<Note>;
  var G3:   Computer<Note>;
  var G$3:  Computer<Note>;
  var Gb3:  Computer<Note>;
  var A3:   Computer<Note>;
  var A$3:  Computer<Note>;
  var Ab3:  Computer<Note>;
  var B3:   Computer<Note>;
  var Bb3:  Computer<Note>;
  var C4:   Computer<Note>;
  var C$4:  Computer<Note>;
  var Cb4:  Computer<Note>;
  var D4:   Computer<Note>;
  var D$4:  Computer<Note>;
  var Db4:  Computer<Note>;
  var E4:   Computer<Note>;
  var E$4:  Computer<Note>;
  var Eb4:  Computer<Note>;
  var F4:   Computer<Note>;
  var F$4:  Computer<Note>;
  var G4:   Computer<Note>;
  var G$4:  Computer<Note>;
  var Gb4:  Computer<Note>;
  var A4:   Computer<Note>;
  var A$4:  Computer<Note>;
  var Ab4:  Computer<Note>;
  var B4:   Computer<Note>;
  var Bb4:  Computer<Note>;
  var C5:   Computer<Note>;
  var C$5:  Computer<Note>;
  var Cb5:  Computer<Note>;
  var D5:   Computer<Note>;
  var D$5:  Computer<Note>;
  var Db5:  Computer<Note>;
  var E5:   Computer<Note>;
  var E$5:  Computer<Note>;
  var Eb5:  Computer<Note>;
  var F5:   Computer<Note>;
  var F$5:  Computer<Note>;
  var G5:   Computer<Note>;
  var G$5:  Computer<Note>;
  var Gb5:  Computer<Note>;
  var A5:   Computer<Note>;
  var A$5:  Computer<Note>;
  var Ab5:  Computer<Note>;
  var B5:   Computer<Note>;
  var Bb5:  Computer<Note>;
  var C6:   Computer<Note>;
  var C$6:  Computer<Note>;
  var Cb6:  Computer<Note>;
  var D6:   Computer<Note>;
  var D$6:  Computer<Note>;
  var Db6:  Computer<Note>;
  var E6:   Computer<Note>;
  var E$6:  Computer<Note>;
  var Eb6:  Computer<Note>;
  var F6:   Computer<Note>;
  var F$6:  Computer<Note>;
  var G6:   Computer<Note>;
  var G$6:  Computer<Note>;
  var Gb6:  Computer<Note>;
  var A6:   Computer<Note>;
  var A$6:  Computer<Note>;
  var Ab6:  Computer<Note>;
  var B6:   Computer<Note>;
  var Bb6:  Computer<Note>;

  var C:    Computer<Note>;
  var C$:   Computer<Note>;
  var Db:    Computer<Note>;
  var D:    Computer<Note>;
  var D$:   Computer<Note>;
  var Eb:    Computer<Note>;
  var E:    Computer<Note>;
  var E$:   Computer<Note>;
  var F:    Computer<Note>;
  var F$:   Computer<Note>;
  var Gb:    Computer<Note>;
  var G:    Computer<Note>;
  var G$:   Computer<Note>;
  var Ab:    Computer<Note>;
  var A:    Computer<Note>;
  var A$:   Computer<Note>;
  var Bb:    Computer<Note>;
  var B:    Computer<Note>;
}

require('./globals');

