import { euclid, rotate } from './algo';
import { cycle, lerp, wave, seq, random  } from './sequence';
import { framesPerBeat, framesPerPhrase, flipPulse, pulseOnBeat, merge  } from './library';

import { memoize  } from './memoize';
import { arp, note, notes, chord, transpose } from './note';

// Globals defined at runtime by Orcvs
// globalThis.pattern;
// globalThis.ptn;
// globalThis.bpm;;
// globalThis.play;
// globalThis.ply;
// globalThis.control;
// globalThis.ctl;
// globalThis.output
// globalThis.out;


globalThis.ORCVS = 'O̴̫͉͌r̸̘͉̫̣̐̈́͊c̶̛̪̖̻͔̈́̃̓v̷̨͎̿͝ŝ̷̩͑̾';

globalThis.BANG = '▮';
globalThis.UNBANG = '▯';

globalThis.OFF = 0;

globalThis.CC_MOD = 1;
globalThis.CC_VOL = 7;
globalThis.CC_BAL = 8;

globalThis.lerp = lerp;
globalThis.lrp = lerp;

globalThis.cycle = cycle;
globalThis.cyc = cycle;

globalThis.wave = wave;
globalThis.wav = wave;

globalThis.seq = seq;

globalThis.random = random
globalThis.rnd = random

globalThis.framesPerBeat = framesPerBeat;
globalThis.beat = framesPerBeat;
globalThis.fpb = framesPerBeat;

globalThis.framesPerPhrase = framesPerPhrase;
globalThis.phrase = framesPerPhrase;
globalThis.fpp = framesPerPhrase;

globalThis.pulseOnBeat = pulseOnBeat;
globalThis.pulse = pulseOnBeat;
globalThis.pls = pulseOnBeat;

globalThis.flipPulse = flipPulse;
globalThis.flip = flipPulse;
globalThis.flp = flipPulse;

globalThis.merge = merge;
globalThis.mrg = merge;

globalThis.chord = chord;
globalThis.crd = chord;

globalThis.note = note;
globalThis.nte = note;

globalThis.notes = notes;
globalThis.nts = notes;

globalThis.arp = arp;

globalThis.euclid = euclid;
globalThis.euc = euclid;

globalThis.rotate = rotate;
globalThis.rot = rotate;

globalThis.transpose = transpose;
globalThis.trn = transpose;


globalThis.memoize = memoize;

globalThis.a = 10;
globalThis.b = 11;
globalThis.c = 12;
globalThis.d = 13;
globalThis.e = 14;
globalThis.f = 16;
globalThis.g = 17;
globalThis.h = 18;
globalThis.i = 19;
globalThis.j = 20;
globalThis.k = 21;
globalThis.l = 22;
globalThis.m = 23;
globalThis.n = 24;
globalThis.o = 25;
globalThis.p = 26;
globalThis.q = 27;
globalThis.r = 28;
globalThis.s = 29;
globalThis.t = 30;
globalThis.u = 31;
globalThis.v = 32;
globalThis.w = 33;
globalThis.x = 34;
globalThis.y = 35;
globalThis.z = 36;

globalThis.C2 = note('C2');
globalThis.C$2 = note('C#2');
globalThis.D2  = note('D2');
globalThis.D$2 = note('D#2');
globalThis.Db2 = note('Db2');
globalThis.E2  = note('E2');
globalThis.E$2 = note('E#2');
globalThis.Eb2 = note('Eb2');
globalThis.F2  = note('F2');
globalThis.F$2 = note('F#2');
globalThis.G2  = note('G2');
globalThis.G$2 = note('G#2');
globalThis.Gb2 = note('Gb2');
globalThis.A2  = note('A2');
globalThis.A$2 = note('A#2');
globalThis.Ab2 = note('Ab2');
globalThis.B2  = note('B2');
globalThis.Bb2 = note('Bb2');
globalThis.C3  = note('C3');
globalThis.C$3 = note('C#3');
globalThis.Cb3 = note('Cb3');
globalThis.D3  = note('D3');
globalThis.D$3 = note('D#3');
globalThis.Db3 = note('Db3');
globalThis.E3  = note('E3');
globalThis.E$3 = note('E#3');
globalThis.Eb3 = note('Eb3');
globalThis.F3  = note('F3');
globalThis.F$3 = note('F#3');
globalThis.G3  = note('G3');
globalThis.G$3 = note('G#3');
globalThis.Gb3 = note('Gb3');
globalThis.A3  = note('A3');
globalThis.A$3 = note('A#3');
globalThis.Ab3 = note('Ab3');
globalThis.B3  = note('B3');
globalThis.Bb3 = note('Bb3');
globalThis.C4  = note('C4');
globalThis.C$4 = note('C#4');
globalThis.Cb4 = note('Cb4');
globalThis.D4  = note('D4');
globalThis.D$4 = note('D#4');
globalThis.Db4 = note('Db4');
globalThis.E4  = note('E4');
globalThis.E$4 = note('E#4');
globalThis.Eb4 = note('Eb4');
globalThis.F4  = note('F4');
globalThis.F$4 = note('F#4');
globalThis.G4  = note('G4');
globalThis.G$4 = note('G#4');
globalThis.Gb4 = note('Gb4');
globalThis.A4  = note('A4');
globalThis.A$4 = note('A#4');
globalThis.Ab4 = note('Ab4');
globalThis.B4  = note('B4');
globalThis.Bb4 = note('Bb4');
globalThis.C5  = note('C5');
globalThis.C$5 = note('C#5');
globalThis.Cb5 = note('Cb5');
globalThis.D5  = note('D5');
globalThis.D$5 = note('D#5');
globalThis.Db5 = note('Db5');
globalThis.E5  = note('E5');
globalThis.E$5 = note('E#5');
globalThis.Eb5 = note('Eb5');
globalThis.F5  = note('F5');
globalThis.F$5 = note('F#5');
globalThis.G5  = note('G5');
globalThis.G$5 = note('G#5');
globalThis.Gb5 = note('Gb5');
globalThis.A5  = note('A5');
globalThis.A$5 = note('A#5');
globalThis.Ab5 = note('Ab5');
globalThis.B5  = note('B5');
globalThis.Bb5 = note('Bb5');
globalThis.C6  = note('C6');
globalThis.C$6 = note('C#6');
globalThis.Cb6 = note('Cb6');
globalThis.D6  = note('D6');
globalThis.D$6 = note('D#6');
globalThis.Db6 = note('Db6');
globalThis.E6  = note('E6');
globalThis.E$6 = note('E#6');
globalThis.Eb6 = note('Eb6');
globalThis.F6  = note('F6');
globalThis.F$6 = note('F#6');
globalThis.G6  = note('G6');
globalThis.G$6 = note('G#6');
globalThis.Gb6 = note('Gb6');
globalThis.A6  = note('A6');
globalThis.A$6 = note('A#6');
globalThis.Ab6 = note('Ab6');
globalThis.B6  = note('B6');
globalThis.Bb6 = note('Bb6');

globalThis.C   = note('C4');
globalThis.C$  = note('C#4');
globalThis.Db  = note('Db4');
globalThis.D   = note('D4');
globalThis.D$  = note('D#4');
globalThis.Eb  = note('Eb4');
globalThis.E   = note('E4');
globalThis.E$  = note('E#4');
globalThis.F   = note('F4');
globalThis.F$  = note('F#4');
globalThis.G   = note('G4');
globalThis.G$  = note('G#4');
globalThis.Ab  = note('Ab4');
globalThis.A   = note('A4');
globalThis.A$  = note('A#4');
globalThis.Bb  = note('Bb4');
globalThis.B   = note('B4');

// globalThis.a = 10;
// globalThis.b = 11;
// globalThis.c = 12;
// globalThis.d = 13;
// globalThis.e = 14;
// globalThis.f = 16;
// globalThis.g = 17;
// globalThis.h = 18;
// globalThis.i = 19;
// globalThis.j = 20;
// globalThis.k = 21;
// globalThis.l = 22;
// globalThis.m = 23;
// globalThis.n = 24;
// globalThis.o = 25;
// globalThis.p = 26;
// globalThis.q = 27;
// globalThis.r = 28;
// globalThis.s = 29;
// globalThis.t = 30;
// globalThis.u = 31;
// globalThis.v = 32;
// globalThis.w = 33;
// globalThis.x = 34;
// globalThis.y = 35;
// globalThis.z = 36;
