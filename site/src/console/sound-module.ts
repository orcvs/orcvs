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
/**
 * The Grid's width: sixteen channel lights of two Cells each, or four effects of eight, a name,
 * value and magnitude of two Cells each and two spacers, so each row of effects wraps on its own.
 */
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

/** `#rgb` or `#rrggbb` as `[r, g, b]`, each from 0 to 255. The build shortens six digits to three. */
function rgb(color: string): number[] {
  const digits = color.length === 4 ? [...color.slice(1)].map((digit) => digit + digit) : [1, 3, 5].map((at) => color.slice(at, at + 2));
  return digits.map((pair) => parseInt(pair, 16));
}

/** The relative luminance of `[r, g, b]`, each from 0 to 255. */
function luminance(color: number[]): number {
  const [r, g, b] = color.map((channel) => {
    const linear = channel / 255;
    return linear <= 0.04045 ? linear / 12.92 : ((linear + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

const contrast = (a: number[], b: number[]) => {
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (light + 0.05) / (dark + 0.05);
};

/**
 * Draws a light's digit in whichever of the ordinary colour and the Theme's `sound.channel` reads
 * better on its fill, which runs from the window colour to `sound.magnitude` with velocity. The
 * Theme's colours are hex.
 */
function ink(digit: HTMLElement, value: number) {
  const style = getComputedStyle(digit);
  const theme = (name: string) => rgb(style.getPropertyValue(name).trim());
  const ground = theme("--orcvs-window");
  const level = value / 0x7f;
  const lit = theme("--orcvs-sound-magnitude").map((channel, index) => channel * level + ground[index] * (1 - level));
  const channel = contrast(theme("--orcvs-sound-channel"), lit) > contrast(theme("--orcvs-ordinary"), lit);
  digit.style.color = channel ? "var(--orcvs-sound-channel)" : "";
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
  // A blank row between the lights and the effects.
  for (let blank = 0; blank < COLUMNS; blank++) grid.append(cell());

  const effects = EFFECTS.map(([name, value], index) => {
    const digits = [cell("number"), cell("number")];
    const magnitude = [cell("magnitude"), cell("magnitude")];
    const named = [...name].map((character) => cell("function", character));
    for (const target of [...named, ...digits, ...magnitude]) {
      target.dataset.effect = String(index);
      target.title = `CC ${index + 1}`;
    }
    grid.append(...named, ...digits, ...magnitude, cell(), cell());
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
      ink(light.pair[0], data2);
      clearTimeout(light.timer);
      light.timer = window.setTimeout(() => {
        fill(light.pair, 0);
        ink(light.pair[0], 0);
      }, LIGHT_MS);
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
