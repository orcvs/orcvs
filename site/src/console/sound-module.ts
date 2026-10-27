// The Sound Module (ADR 0088): sixteen channel lights and the Built-in Synth's eight effects, drawn
// as Cells. It shows what the page's `orcvs.midi` messages carry, whoever sent them, and sends the
// reader's changes as Control Change messages on the same page, so the synth and the Sound Module
// hear one stream.

/** How long a channel light stays filled after a Note On. */
const LIGHT_MS = 150;
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
const EFFECTS_PER_ROW = 4;
/** An effect's name, value and magnitude, two Cells each, and a spacer. */
const EFFECT_CELLS = 7;
/** A row of channel lights, two Cells each, and the Grid's width. */
const COLUMNS = 32;

const hex = (value: number) => value.toString(16).toUpperCase().padStart(2, "0");

function cell(role = "", text = ""): HTMLSpanElement {
  const span = document.createElement("span");
  if (role) span.className = role;
  span.textContent = text;
  return span;
}

/** Fills `cells` with the magnitude colour at `value`'s fraction of `7F`. */
function fill(cells: HTMLElement[], value: number) {
  for (const target of cells) target.style.setProperty("--level", String(value / 0x7f));
}

/** Draws the Sound Module into `module`. */
export function soundModule(module: HTMLElement) {
  const grid = document.createElement("div");
  grid.className = "orcvs-grid orcvs-sound-module";
  grid.style.setProperty("--columns", String(COLUMNS));

  const lights = Array.from({ length: 16 }, (_, channel) => {
    const pair = [cell("light", channel.toString(16).toUpperCase()), cell("light")];
    grid.append(...pair);
    return { pair, timer: 0 };
  });

  const effects = EFFECTS.map(([name, value], index) => {
    const digits = [cell("number"), cell("number")];
    const magnitude = [cell("magnitude"), cell("magnitude")];
    const named = [...name].map((character) => cell("function", character));
    for (const target of [...named, ...digits, ...magnitude]) {
      target.dataset.effect = String(index);
      target.title = `CC ${index + 1}`;
    }
    grid.append(...named, ...digits, ...magnitude, cell());
    if (index % EFFECTS_PER_ROW === EFFECTS_PER_ROW - 1) {
      for (let blank = EFFECTS_PER_ROW * EFFECT_CELLS; blank < COLUMNS; blank++) grid.append(cell());
    }
    const effect = { value, digits, magnitude };
    show(effect, value);
    return effect;
  });

  function show(effect: { value: number; digits: HTMLElement[]; magnitude: HTMLElement[] }, value: number) {
    effect.value = value;
    const [high, low] = hex(value);
    effect.digits[0].textContent = high;
    effect.digits[1].textContent = low;
    fill(effect.magnitude, value);
  }

  function hear(bytes: number[]) {
    const [status, data1 = 0, data2 = 0] = bytes;
    const channel = status & 0x0f;
    if (status >> 4 === 0x9 && data2 > 0) {
      const light = lights[channel];
      fill(light.pair, data2);
      clearTimeout(light.timer);
      light.timer = window.setTimeout(() => fill(light.pair, 0), LIGHT_MS);
    } else if (status >> 4 === 0xb && data1 >= 1 && data1 <= EFFECTS.length) {
      show(effects[data1 - 1], data2);
    }
  }

  window.addEventListener("message", (event) => {
    if (event.origin !== location.origin || event.data?.type !== "orcvs.midi") return;
    if (Array.isArray(event.data.bytes)) hear(event.data.bytes);
  });

  // The reader's change, sent as a console sends one, so the synth plays it and `hear` shows it.
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

  // Dragging up raises a value and dragging down lowers it.
  grid.addEventListener("pointerdown", (event) => {
    const index = effectOf(event.target);
    if (index === undefined) return;
    // The console keeps the keyboard.
    event.preventDefault();
    const from = { y: event.clientY, value: effects[index].value };
    const move = (moved: PointerEvent) => send(index, from.value + Math.round((from.y - moved.clientY) / DRAG_STEP));
    const end = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", end);
      window.removeEventListener("pointercancel", end);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", end);
    window.addEventListener("pointercancel", end);
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
