// The Sound Module (ADR 0088): sixteen channels and the Built-in Synth's eight effects, drawn as
// Cells. Each channel has its own effects; the effects show and send those of the chosen channel,
// which follows the channel of each Control Change heard until the reader chooses one. It shows
// what the page's `orcvs.midi` messages carry, whoever sent them, and sends the reader's changes
// as Control Change messages on the same page, so the synth and the Sound Module hear one stream.

/** How long a channel shows as sounding after a Note On. */
const SOUNDING_MS = 150;

/**
 * What an effect's column shows: its two-Cell name, the word its Cells show on hover, and its load
 * value.
 */
type Label = { name: string; word: string; load: number };

/**
 * Each effect's controller and label, in Control Change order. The word tells `CH` (chorus) from a
 * channel. On the acid voice's channel CC 74 and 75 are its cutoff and decay, so those columns take
 * the acid voice's label there.
 */
const EFFECTS: { controller: number; label: Label; acid?: Label }[] = [
  { controller: 1, label: { name: "MD", word: "Modulation", load: 0x00 } },
  { controller: 74, label: { name: "EQ", word: "EQ", load: 0x40 }, acid: { name: "CU", word: "Cutoff", load: 0x30 } },
  { controller: 75, label: { name: "LN", word: "Length", load: 0x7f }, acid: { name: "DC", word: "Decay", load: 0x30 } },
  { controller: 85, label: { name: "OF", word: "Offset", load: 0x00 } },
  { controller: 91, label: { name: "R1", word: "Reverb 1", load: 0x00 } },
  { controller: 92, label: { name: "R2", word: "Reverb 2", load: 0x00 } },
  { controller: 93, label: { name: "CH", word: "Chorus", load: 0x00 } },
  { controller: 94, label: { name: "DL", word: "Delay", load: 0x00 } },
];
/** The acid voice's channel. */
const ACID = 0x8;
/** Reset All Controllers, which returns the acid voice's controls to their load values. */
const RESET_ALL_CONTROLLERS = 121;
/** Effect `index`'s label on `channel`. */
const labelOf = (index: number, channel: number) =>
  (channel === ACID && EFFECTS[index].acid) || EFFECTS[index].label;
/** Each channel's effects at load, by channel, then in `EFFECTS` order. */
const loads = () => Array.from({ length: 16 }, (_, channel) => EFFECTS.map((_, index) => labelOf(index, channel).load));
/**
 * The Grid's width: sixteen channels of one Cell and a spacer, or eight effects of two Cells and
 * two spacers, each effect's magnitude, value and name stacked in its own column.
 */
const COLUMNS = 32;

const hex = (value: number) => value.toString(16).toUpperCase().padStart(2, "0");

function cell(role = "", text = ""): HTMLSpanElement {
  const span = document.createElement("span");
  if (role) span.className = role;
  span.textContent = text;
  return span;
}

/** Draws the Sound Module into `module`. */
export function soundModule(module: HTMLElement) {
  const grid = document.createElement("div");
  grid.className = "orcvs-grid orcvs-sound-module";
  grid.style.setProperty("--columns", String(COLUMNS));

  const channels = Array.from({ length: 16 }, (_, channel) => {
    const digit = cell("channel", channel.toString(16).toUpperCase());
    digit.title = `Channel ${hex(channel)}, MIDI channel ${channel + 1}`;
    // A button the keyboard reaches with Tab: Enter or Space chooses it, and the arrow keys step to
    // the next channel. Its keys stop here, so a digit typed on it writes no value.
    digit.tabIndex = 0;
    digit.setAttribute("role", "button");
    digit.addEventListener("keydown", (event) => {
      event.stopPropagation();
      if (event.metaKey || event.ctrlKey || event.altKey) return;
      if (event.key === "Enter" || event.key === " ") pick(channel);
      else if (event.key === "ArrowLeft") channels[Math.max(0, channel - 1)].digit.focus();
      else if (event.key === "ArrowRight") channels[Math.min(15, channel + 1)].digit.focus();
      else return;
      event.preventDefault();
    });
    grid.append(digit, cell());
    return { digit, timer: 0 };
  });
  // Every channel's effect values, and the channel the effects show and send. Until the reader
  // chooses one, the chosen channel follows each Control Change heard, so a Source's effects show.
  const values = loads();
  let chosen = 0;
  let picked = false;
  // A blank row between the channels and the effects.
  for (let blank = 0; blank < COLUMNS; blank++) grid.append(cell());

  const effects = EFFECTS.map(() => ({
    value: 0,
    magnitude: [cell("magnitude"), cell("magnitude")],
    digits: [cell("number"), cell("number")],
    name: [cell("function"), cell("function")],
  }));
  // Each effect's column: its magnitude, then its value, then its name, a row each.
  const rows = [
    effects.map((effect) => effect.magnitude),
    effects.map((effect) => effect.digits),
    effects.map((effect) => effect.name),
  ];
  for (const row of rows) {
    row.forEach((pair, index) => {
      for (const target of pair) target.dataset.effect = String(index);
      grid.append(...pair, cell(), cell());
    });
  }
  choose(0);

  /** Shows `channel`'s effects, under its labels, and sends the reader's changes on it. */
  function choose(channel: number) {
    channels[chosen].digit.classList.remove("chosen");
    channels[chosen].digit.setAttribute("aria-pressed", "false");
    chosen = channel;
    channels[chosen].digit.classList.add("chosen");
    channels[chosen].digit.setAttribute("aria-pressed", "true");
    effects.forEach((effect, index) => {
      const label = labelOf(index, chosen);
      effect.name.forEach((target, position) => (target.textContent = label.name[position]));
      for (const target of [...effect.magnitude, ...effect.digits, ...effect.name]) {
        target.title = `${label.word}, CC ${EFFECTS[index].controller}`;
      }
      show(effect, values[chosen][index]);
    });
  }

  /** The reader's choice, which the chosen channel then keeps. */
  function pick(channel: number) {
    picked = true;
    choose(channel);
  }

  function show(effect: { value: number; digits: HTMLElement[]; magnitude: HTMLElement[] }, value: number) {
    effect.value = value;
    const [high, low] = hex(value);
    effect.digits[0].textContent = high;
    effect.digits[1].textContent = low;
    for (const target of effect.magnitude) target.style.setProperty("--level", String(value / 0x7f));
  }

  function hear(bytes: number[]) {
    const [status, data1 = 0, data2 = 0] = bytes;
    const channel = status & 0x0f;
    if (status >> 4 === 0x9 && data2 > 0) {
      const sounding = channels[channel];
      sounding.digit.classList.add("sounding");
      clearTimeout(sounding.timer);
      sounding.timer = window.setTimeout(() => sounding.digit.classList.remove("sounding"), SOUNDING_MS);
    } else if (status >> 4 === 0xb) {
      if (channel === ACID && data1 === RESET_ALL_CONTROLLERS) {
        EFFECTS.forEach(({ acid }, index) => {
          if (acid) values[channel][index] = acid.load;
        });
      } else {
        const index = EFFECTS.findIndex(({ controller }) => controller === data1);
        if (index < 0) return;
        values[channel][index] = data2;
      }
      if (channel === chosen || !picked) choose(channel);
    }
  }

  window.addEventListener("message", (event) => {
    if (event.origin !== location.origin || event.data?.type !== "orcvs.midi") return;
    if (Array.isArray(event.data.bytes)) hear(event.data.bytes);
  });

  // The reader's change, sent as a console sends one, so the synth plays it and `hear` shows it.
  // The display follows that round trip, so it relies on `synth-host.js` and the listener above both
  // taking `orcvs.midi` that this window posts to itself, as well as a console's.
  function send(index: number, value: number) {
    if (value === effects[index].value) return;
    const bytes = [0xb0 | chosen, EFFECTS[index].controller, value];
    const message = { type: "orcvs.midi", bytes, time: performance.timeOrigin + performance.now() };
    window.postMessage(message, location.origin);
  }

  // The Cursor: one of the value Cells, which typing writes as the Grid's Cursor does. It shows
  // while the Sound Module has the keyboard; Escape, or a click on the console, gives it back.
  const digits = effects.flatMap((effect) => effect.digits);
  let cursor = 0;
  grid.tabIndex = -1;
  const place = (at: number) => {
    digits[cursor].classList.remove("cursor");
    cursor = Math.max(0, Math.min(digits.length - 1, at));
    digits[cursor].classList.add("cursor");
  };
  grid.addEventListener("focus", () => place(cursor));
  grid.addEventListener("blur", () => digits[cursor].classList.remove("cursor"));
  grid.addEventListener("keydown", (event) => {
    if (event.metaKey || event.ctrlKey || event.altKey) return;
    if (event.key === "Escape") grid.blur();
    else if (event.key === "ArrowLeft") place(cursor - 1);
    else if (event.key === "ArrowRight") place(cursor + 1);
    else if (/^[0-9a-f]$/i.test(event.key)) {
      // A hex digit replaces the one under the Cursor, which steps east. A value past `7F`, which no
      // Control Change carries, is refused.
      const index = Math.floor(cursor / 2);
      const pair = [...hex(effects[index].value)];
      pair[cursor % 2] = event.key.toUpperCase();
      const value = parseInt(pair.join(""), 16);
      if (value > 0x7f) return;
      // Shown at once as well as on the round trip, so the next digit typed reads this one.
      picked = true;
      send(index, value);
      values[chosen][index] = value;
      show(effects[index], value);
      place(cursor + 1);
    } else return;
    event.preventDefault();
  });

  // A press on a value takes the keyboard and puts the Cursor there, and a press on a channel
  // chooses it; anywhere else the console keeps the keyboard.
  grid.addEventListener("pointerdown", (event) => {
    const channel = channels.findIndex(({ digit }) => digit === event.target);
    if (channel >= 0) {
      event.preventDefault();
      pick(channel);
      return;
    }
    const digit = digits.indexOf(event.target as HTMLSpanElement);
    if (digit < 0) return;
    event.preventDefault();
    place(digit);
    grid.focus();
  });

  module.append(grid);
}
