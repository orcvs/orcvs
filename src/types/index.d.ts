import { Bangable } from '../pattern';
import { Computer } from '../library';

declare global {
  var bang: (str: string, callback: Bangable) => void;
  var lerp: (tofrom: number, to?: number, diff?: number) => Computer;
  var cycle: (tofrom: number, to?: number, diff?: number) => Computer;
  var wave: (tofrom: number, to?: number, diff?: number) => Computer;
  var seq:  (...sequence: readonly (string | number)[]) => (() => (string | number));

  var play: (channel: number, octave: Computer, note: string, attack: Computer, duration: Computer) => void;
  var output: (output: number | string) => void;

  var ORCVS: string;
  var BANG: string;
  var A: number;
  var B: number;
  var C: number;
  var D: number;
  var E: number;
  var F: number;
  var G: number;
  var H: number;
  var I: number;
  var J: number;
  var K: number;
  var L: number;
  var M: number;
  var N: number;
  var O: number;
  var P: number;
  var Q: number;
  var R: number;
  var S: number;
  var T: number;
  var U: number;
  var V: number;
  var W: number;
  var X: number;
  var Y: number;
  var Z: number;
}

export {};