import { expect, test } from "@playwright/test";

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
      /** Each message the page passes the worklet: its planned instant, when the synth plays it on
       * the page's clock, how long after its planned instant it reached the page, and its MIDI
       * status byte. */
      sounds: { planned: number; at: number; late: number; status: number }[];
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
    const spy: Window["synthSpy"] = { sounds: [] };
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
      if (event.data?.type === "orcvs.midi") spy.planned = event.data.time;
    });
    const post = MessagePort.prototype.postMessage;
    MessagePort.prototype.postMessage = function (message: any, ...rest: any[]) {
      const context = spy.context;
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
  await page.goto("/");
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
  // The Note On on channel `00`, not only the Note Off that follows once the audio runs.
  await expect
    .poll(() => page.evaluate(() => window.synthSpy.sounds.map(({ status }) => status)))
    .toContain(0x90);
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
