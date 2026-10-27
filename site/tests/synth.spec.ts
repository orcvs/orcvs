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
       * the page's clock, and whether the context had already rendered that time. */
      sounds: { planned: number; at: number; behind: boolean }[];
    };
  }
}

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
          behind: message.when < context.currentTime,
        });
        spy.planned = undefined;
      }
      return post.call(this, message, ...(rest as [any]));
    };
  });
});

test("a launched console plays the page's synth at a constant offset from its Ticks", async ({ page }) => {
  await page.goto("/learn/first-result/");
  // The first launch loads the page's synth; a console framed beside it is answered the same way.
  await expect.poll(() => page.evaluate(() => !!window.synthSpy.context)).toBe(false);
  await page.getByRole("button", { name: "Launch example" }).first().click();
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
  // Each reached the worklet before the context rendered its time, so none sounded late.
  expect(steady.filter((sound) => sound.behind)).toEqual([]);
});
