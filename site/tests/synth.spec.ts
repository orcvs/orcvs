import { expect, test, type Locator, type Page } from "@playwright/test";

// The Built-in Synth: a console a page frames plays the page's synth, each message a
// fixed delay after its Tick's planned instant. The page's own clock and the audio context's are
// compared through what the page passes the synth's AudioWorklet, which a spy records.

// A note on channel `08` every Tick: sixteenths at the default 120 BPM.
const STEADY = "~*0101\n  !~0850f401\n";
// `synth-host.js`'s delay, the 50 ms target.
const DELAY_MS = 50;

declare global {
  interface Window {
    synthSpy: {
      context?: AudioContext;
      /** The planned instant of the `orcvs.midi` the page is handling. */
      planned?: number;
      /** The audio context's state when the page's first `orcvs.midi` reached it. */
      firstMessageFound?: AudioContextState;
      /** Each message the page passes the worklet: its planned instant, when the synth plays it on
       * the page's clock, how long after its planned instant it reached the page, and its MIDI
       * status byte. */
      sounds: { planned: number; at: number; late: number; status: number }[];
      /** The MIDI status byte of every message the page passes the worklet, paired or not: messages
       * held while the audio starts reach the worklet together, after the last one's planned
       * instant, so only one of them pairs with it. */
      statuses: number[];
    };
  }
}

// The timing test measures how late the console's frames send each Tick, so it does not share the
// runner with this file's other console.
test.describe.configure({ mode: "serial" });

// Playwright's Chromium starts audio without a gesture; a reader's browser waits for one.
test.use({ launchOptions: { ignoreDefaultArgs: ["--autoplay-policy=no-user-gesture-required"] } });

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    if (window !== window.top) return;
    const spy: Window["synthSpy"] = { sounds: [], statuses: [] };
    window.synthSpy = spy;
    const Context = window.AudioContext;
    window.AudioContext = class extends Context {
      constructor(options?: AudioContextOptions) {
        super(options);
        spy.context = this;
      }
    };
    // Registered before the page's own listener, which passes the message on to the worklet before
    // the event is done, so each post below pairs with the message that caused it.
    window.addEventListener("message", (event) => {
      if (event.data?.type !== "orcvs.midi") return;
      spy.planned = event.data.time;
      spy.firstMessageFound ??= spy.context?.state;
    });
    const post = MessagePort.prototype.postMessage;
    MessagePort.prototype.postMessage = function (message: any, ...rest: any[]) {
      const context = spy.context;
      if (typeof message?.when === "number") spy.statuses.push(message.bytes[0]);
      if (context && typeof message?.when === "number" && spy.planned !== undefined) {
        const playing = context.getOutputTimestamp();
        spy.sounds.push({
          planned: spy.planned,
          at: performance.timeOrigin + playing.performanceTime! + (message.when - playing.contextTime!) * 1000,
          late: performance.timeOrigin + performance.now() - spy.planned,
          status: message.bytes[0],
        });
        spy.planned = undefined;
      }
      return post.call(this, message, ...(rest as [any]));
    };
  });
});

test("the site's console sounds the first note of a run while the page's audio starts", async ({ page }) => {
  // The Beat is twelve Cells wide, so the Cursor the click below places lands on empty Cells. The
  // Source a first visit opens fills that point with Comments, where typed Functions do nothing.
  await page.goto("/?example=beat");
  const frame = page.frameLocator('iframe[title="Orcvs console"]').locator("#the_canvas_id");
  await expect(frame).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
  await expect.poll(() => page.evaluate(() => window.synthSpy.context?.state)).toBe("suspended");

  // One Timed Play on Tick 0, typed at the Cursor: `**` above `!~0064C404`.
  await frame.click({ position: { x: 300, y: 200 } });
  await page.keyboard.type("**");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.type("!~0064C404");
  await page.keyboard.press("Space");
  // The Note On on channel `00`, not only the Note Off that follows once the audio runs. The
  // Source the console opens may play on Tick 0 too, so its messages are held with this one.
  await expect.poll(() => page.evaluate(() => window.synthSpy.statuses)).toContain(0x90);
});

// Where the Panel puts `P`, from `console/src/console/panel.rs`, at a device scale where a Cell is
// a whole number of pixels, as `Desktop Chrome`'s 1 is: at a fractional scale the console floors
// each row to whole pixels and the rows come out shorter.
// A Cell's height in points, `CELL_SIZE.y`.
const CELL_HEIGHT = 18;
// With Info shown and Diagnostics closed, as a first visit opens, the Panel spans six Cell rows: a
// row of room, the readouts, a row of room, Info's Cells, its labels, and a row of room. P is in
// the second from the Panel's top, so its centre is four and a half rows above the bottom edge.
const P_ROWS_ABOVE_BOTTOM = 4.5;
// P spans 24 to 36: the Panel's 8-point inset and its 10-point left pad, snapped up to the next
// 12-point Cell column.
const P_CENTRE_X = 30;

// The reader's first gesture starts Playback, so Tick 0 waits for the page's audio to start. Space
// and the Panel's `P` make the same request of the console.
const firstPlays: { gesture: string; play: (page: Page, frame: Locator) => Promise<void> }[] = [
  {
    gesture: "Space",
    play: async (page, frame) => {
      await frame.focus();
      await page.keyboard.press("Space");
    },
  },
  {
    gesture: "a click on P",
    // `P` is the first Cell of the Panel's readout row. The canvas has no DOM to find it by, so the
    // point is fixed; a click that misses it wakes the audio but starts no Playback, and the poll
    // below says so. `console::kittest_tests::info::the_site_suites_click_on_p_lands_on_the_play_cell`
    // fails a Panel change that moves P off this point.
    play: async (_page, frame) => {
      const box = await frame.boundingBox();
      await frame.click({ position: { x: P_CENTRE_X, y: box!.height - P_ROWS_ABOVE_BOTTOM * CELL_HEIGHT } });
    },
  },
];

for (const { gesture, play } of firstPlays) {
  test(`a first visit opens Knobs, and its first Tick sounds on time after ${gesture}`, async ({ page }) => {
    await page.goto("/");
    const frame = page.frameLocator('iframe[title="Orcvs console"]').locator("#the_canvas_id");
    await expect(frame).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
    await expect.poll(() => page.evaluate(() => window.synthSpy.context?.state)).toBe("suspended");

    // Nothing is stored, so the console opens `examples/knobs.orcvs`: the bassline's accented A1 on
    // the acid channel `09` leads Tick 0, ahead of the knobs' Control Changes.
    await play(page, frame);
    await expect
      .poll(() => page.evaluate(() => window.synthSpy.sounds.length), {
        message: `${gesture} started no Playback; a click at a fixed point misses P once the Panel's layout moves`,
      })
      .toBeGreaterThan(0);

    // Tick 0 waited for the audio, so its messages found it running and sound when they were
    // planned to, rather than wait behind its start-up.
    expect(await page.evaluate(() => window.synthSpy.firstMessageFound)).toBe("running");
    expect(await page.evaluate(() => window.synthSpy.sounds[0].status)).toBe(0x99);
  });
}

test("a click in the console starts the page's audio before Space", async ({ page }) => {
  await page.goto("/");
  const frame = page.frameLocator('iframe[title="Orcvs console"]').locator("#the_canvas_id");
  await expect(frame).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
  await expect.poll(() => page.evaluate(() => window.synthSpy.context?.state)).toBe("suspended");

  await frame.click({ position: { x: 300, y: 200 } });

  await expect.poll(() => page.evaluate(() => window.synthSpy.context?.state)).toBe("running");
});

test("a framed console plays the page's synth at a constant offset from its Ticks", async ({ page }) => {
  await page.goto("/learn/");
  // The first press of a Grid starts the small console and loads the page's synth; a console framed
  // beside it is answered the same way.
  await expect.poll(() => page.evaluate(() => !!window.synthSpy.context)).toBe(false);
  await page.locator(".orcvs-grid[data-source]").first().click();
  await expect.poll(() => page.evaluate(() => !!window.synthSpy.context)).toBe(true);
  await page.evaluate((source) => {
    const frame = document.createElement("iframe");
    frame.src = `/play/#source=${encodeURIComponent(source)}`;
    frame.title = "Steady";
    frame.style.cssText = "position:fixed;inset:0;width:640px;height:480px;z-index:10";
    document.body.append(frame);
  }, STEADY);
  const frame = page.frameLocator('iframe[title="Steady"]').locator("#the_canvas_id");
  await expect(frame).toHaveAttribute("width", /\d+/, { timeout: 30_000 });

  // The reader's gesture: a click on the console, then Space starts Playback.
  await frame.click({ position: { x: 300, y: 200 } });
  await page.keyboard.press("Space");
  await expect.poll(() => page.evaluate(() => window.synthSpy.context!.state)).toBe("running");
  await expect.poll(() => page.evaluate(() => window.synthSpy.sounds.length), { timeout: 10_000 }).toBeGreaterThan(24);

  const { sounds } = await page.evaluate(() => window.synthSpy);
  // The first messages of a run arrive while the context starts, which pairs no time with the
  // page's clock until it plays.
  const steady = sounds.slice(4);
  // Each message carries its Tick's planned instant, not when it was sent: the Ticks are a
  // sixteenth apart however late the console's frames ran. A Tick's Note Off and Note On are
  // stamped a fraction of a millisecond apart, as each is posted.
  const ticks = steady.map(({ planned }) => planned).filter((time, index, all) => index === 0 || time - all[index - 1] > 2);
  for (let index = 1; index < ticks.length; index++) {
    expect(Math.abs(ticks[index] - ticks[index - 1] - 125)).toBeLessThan(2);
  }
  for (const { at, planned } of steady) expect(Math.abs(at - planned - DELAY_MS)).toBeLessThan(2);
  // Each reached the page well within the delay, so none sounds late for the console's sake. How far
  // ahead of its output the context renders belongs to the audio device, and a headless runner's
  // renders further ahead than a reader's.
  expect(steady.filter(({ late }) => late >= DELAY_MS)).toEqual([]);
});

// Safari resumes audio only inside the reader's gesture, so a tap in the console's frame resumes
// the page's audio itself rather than leaving it to the first note the tap leads to.
test.describe("on a phone", () => {
  test.use({ viewport: { width: 390, height: 664 }, hasTouch: true });

  test("a tap in the console's frame starts the page's audio before any note", async ({ page }) => {
    await page.goto("/");
    const frame = page.frameLocator('iframe[title="Orcvs console"]').locator("#the_canvas_id");
    await expect(frame).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
    await expect.poll(() => page.evaluate(() => window.synthSpy.context?.state)).toBe("suspended");

    // A Cell of the Grid, well above the Panel and its Play control.
    await frame.tap({ position: { x: 200, y: 200 } });
    await expect.poll(() => page.evaluate(() => window.synthSpy.context?.state)).toBe("running");
    expect(await page.evaluate(() => window.synthSpy.sounds.length), "the tap started Playback").toBe(0);
  });
});
