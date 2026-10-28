// The Sound Module (ADR 0088): sixteen channels and the Built-in Synth's eight effects, drawn as
// Cells. It shows what the page's `orcvs.midi` messages carry, whoever sent them, and sends the
// reader's changes as Control Change messages on the same page, so the synth and the Sound Module
// hear one stream.

/** How long a channel shows as sounding after a Note On. */
const SOUNDING_MS = 150;
/** How far the pointer moves, in pixels, to change a value by one. */
const DRAG_STEP = 2;

/** Each effect's two-Cell name and load value, in Control Change order from CC 1. */
const EFFECTS: [string, number][] = [
  ["MD", 0x00],
  ["OF", 0x00],
  ["LN", 0x7f],
  ["DL", 0x00],
  ["CH", 0x00],
  ["EQ", 0x40],
  ["R1", 0x00],
  ["R2", 0x00],
];
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
    grid.append(digit, cell());
    return { digit, timer: 0 };
  });
  // A blank row between the channels and the effects.
  for (let blank = 0; blank < COLUMNS; blank++) grid.append(cell());

  const effects = EFFECTS.map(([, value]) => ({
    value,
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
        target.title = `CC ${index + 1}`;
      }
      grid.append(...pair, cell(), cell());
    });
  }
  for (const effect of effects) show(effect, effect.value);

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
    } else if (status >> 4 === 0xb && data1 >= 1 && data1 <= EFFECTS.length) {
      show(effects[data1 - 1], data2);
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
    const clamped = Math.max(0, Math.min(0x7f, value));
    if (clamped === effects[index].value) return;
    const message = { type: "orcvs.midi", bytes: [0xb0, index + 1, clamped], time: performance.timeOrigin + performance.now() };
    window.postMessage(message, location.origin);
  }

  const effectOf = (target: EventTarget | null) => {
    const index = (target as HTMLElement | null)?.closest<HTMLElement>("[data-effect]")?.dataset.effect;
    return index === undefined ? undefined : Number(index);
  };

  // Dragging up raises a value and dragging down lowers it. The grid captures the pointer, so a
  // drag that crosses the console's frame keeps its events, and the capture ends with the drag.
  grid.addEventListener("pointerdown", (event) => {
    const index = effectOf(event.target);
    if (index === undefined) return;
    // The console keeps the keyboard.
    event.preventDefault();
    const from = { y: event.clientY, value: effects[index].value };
    const move = (moved: PointerEvent) => {
      if (moved.pointerId !== event.pointerId) return;
      send(index, from.value + Math.round((from.y - moved.clientY) / DRAG_STEP));
    };
    const end = () => {
      grid.removeEventListener("pointermove", move);
      grid.removeEventListener("lostpointercapture", end);
    };
    grid.addEventListener("pointermove", move);
    grid.addEventListener("lostpointercapture", end);
    grid.setPointerCapture(event.pointerId);
  });

  grid.addEventListener(
    "wheel",
    (event) => {
      const index = effectOf(event.target);
      if (index === undefined) return;
      event.preventDefault();
      send(index, effects[index].value + (event.deltaY < 0 ? 1 : -1));
    },
    { passive: false },
  );

  module.append(grid);
}
