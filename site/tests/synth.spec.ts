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
      /** Each message's planned instant, as the console stamped it. */
      planned: number[];
      /** When the synth plays each message, on the page's clock, and the context's render time then. */
      sounds: { at: number; behind: boolean }[];
    };
  }
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    if (window !== window.top) return;
    const spy: Window["synthSpy"] = { planned: [], sounds: [] };
    window.synthSpy = spy;
    const Context = window.AudioContext;
    window.AudioContext = class extends Context {
      constructor(options?: AudioContextOptions) {
        super(options);
        spy.context = this;
      }
    };
    const post = MessagePort.prototype.postMessage;
    MessagePort.prototype.postMessage = function (message: any, ...rest: any[]) {
      const context = spy.context;
      if (context && typeof message?.when === "number") {
        const playing = context.getOutputTimestamp();
        spy.sounds.push({
          at: performance.timeOrigin + playing.performanceTime! + (message.when - playing.contextTime!) * 1000,
          behind: message.when < context.currentTime,
        });
      }
      return post.call(this, message, ...(rest as [any]));
    };
    window.addEventListener("message", (event) => {
      if (event.data?.type === "orcvs.midi") spy.planned.push(event.data.time);
    });
  });
});

test("a launched console plays the page's synth at a constant offset from its Ticks", async ({ page }) => {
  await page.goto("/learn/first-result/");
  // Launch's script loads the synth; a console framed beside it is answered the same way.
  await expect.poll(() => page.evaluate(() => !!window.synthSpy.context)).toBe(true);
  await page.evaluate((source) => {
    const frame = document.createElement("iframe");
    frame.src = `/play/#source=${encodeURIComponent(source)}`;
    frame.style.cssText = "position:fixed;inset:0;width:640px;height:480px;z-index:10";
    document.body.append(frame);
  }, STEADY);
  const frame = page.frameLocator("iframe").locator("#the_canvas_id");
  await expect(frame).toHaveAttribute("width", /\d+/, { timeout: 30_000 });

  // The reader's gesture: a click on the console, then Space starts Playback.
  await frame.click({ position: { x: 300, y: 200 } });
  await page.keyboard.press("Space");
  await expect.poll(() => page.evaluate(() => window.synthSpy.context!.state)).toBe("running");
  await expect.poll(() => page.evaluate(() => window.synthSpy.sounds.length), { timeout: 10_000 }).toBeGreaterThan(24);

  const { planned, sounds } = await page.evaluate(() => window.synthSpy);
  // The first messages resume the context, which pairs no time with the page's clock until it plays.
  const steady = sounds.slice(8).map((sound, index) => ({ ...sound, planned: planned[index + 8] }));
  // Each message carries its Tick's planned instant, not when it was sent: the Ticks are exactly a
  // sixteenth apart however late the console's frames ran.
  // A Tick's Note Off and Note On are stamped a fraction of a millisecond apart, as each is posted.
  const ticks = steady.map(({ planned }) => planned).filter((time, index, all) => index === 0 || time - all[index - 1] > 1);
  for (let index = 1; index < ticks.length; index++) {
    expect(Math.abs(ticks[index] - ticks[index - 1] - 125)).toBeLessThan(1);
  }
  const offsets = steady.map(({ at, planned }) => at - planned);
  expect(Math.max(...offsets) - Math.min(...offsets)).toBeLessThan(2);
  for (const offset of offsets) expect(Math.abs(offset - DELAY_MS)).toBeLessThan(2);
  // Each reached the worklet before the context rendered its time, so none sounded late.
  expect(steady.filter((sound) => sound.behind)).toEqual([]);
});
