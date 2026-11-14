// The Sound Module (ADR 0088): sixteen channels and the Built-in Synth's eight effects, drawn as
// Cells. Each channel has its own effects; the effects show and send those of the chosen channel.
// It shows what the page's `orcvs.midi` messages carry, whoever sent them, and sends the reader's
// changes as Control Change messages on the same page, so the synth and the Sound Module hear one
// stream.

/** How long a channel shows as sounding after a Note On. */
const SOUNDING_MS = 150;

/**
 * Each effect's two-Cell name, the word its Cells show on hover, its controller and its load value,
 * in Control Change order. The word tells `CH` (chorus) from a channel.
 */
const EFFECTS: [string, string, number, number][] = [
  ["MD", "Modulation", 1, 0x00],
  ["EQ", "EQ", 74, 0x40],
  ["LN", "Length", 75, 0x7f],
  ["OF", "Offset", 85, 0x00],
  ["R1", "Reverb 1", 91, 0x00],
  ["R2", "Reverb 2", 92, 0x00],
  ["CH", "Chorus", 93, 0x00],
  ["DL", "Delay", 94, 0x00],
];
/** The acid voice's channel, where CC 74 and 75 are its cutoff and decay, which load at `30`. */
const ACID = 0x8;
const ACID_LOADS: Record<number, number> = { 74: 0x30, 75: 0x30 };
/** Each channel's effects at load, by channel, then in `EFFECTS` order. */
const loads = () =>
  Array.from({ length: 16 }, (_, channel) =>
    EFFECTS.map(([, , controller, value]) => (channel === ACID ? (ACID_LOADS[controller] ?? value) : value)),
  );
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
    grid.append(digit, cell());
    return { digit, timer: 0 };
  });
  // Every channel's effect values, and the channel the effects show and send.
  const values = loads();
  let chosen = 0;
  // A blank row between the channels and the effects.
  for (let blank = 0; blank < COLUMNS; blank++) grid.append(cell());

  const effects = EFFECTS.map(() => ({
    value: 0,
    magnitude: [cell("magnitude"), cell("magnitude")],
    digits: [cell("number"), cell("number")],
  }));
  // Each effect's column: its magnitude, then its value, then its name, a row each.
  const rows = [
    effects.map((effect) => effect.magnitude),
    effects.map((effect) => effect.digits),
    EFFECTS.map(([name]) => [...name].map((character) => cell("function", character))),
  ];
  for (const row of rows) {
    row.forEach((pair, index) => {
      for (const target of pair) {
        target.dataset.effect = String(index);
        target.title = `${EFFECTS[index][1]}, CC ${EFFECTS[index][2]}`;
      }
      grid.append(...pair, cell(), cell());
    });
  }
  choose(0);

  /** Shows `channel`'s effects and sends the reader's changes on it. */
  function choose(channel: number) {
    channels[chosen].digit.classList.remove("chosen");
    chosen = channel;
    channels[chosen].digit.classList.add("chosen");
    effects.forEach((effect, index) => show(effect, values[chosen][index]));
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
      const index = EFFECTS.findIndex(([, , controller]) => controller === data1);
      if (index < 0) return;
      values[channel][index] = data2;
      if (channel === chosen) show(effects[index], data2);
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
    const bytes = [0xb0 | chosen, EFFECTS[index][2], value];
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
      choose(channel);
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
