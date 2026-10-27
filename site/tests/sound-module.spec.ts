import { expect, test, type Page } from "@playwright/test";

// The Sound Module below the console on `/`: it shows the console's notes and Control Changes, and
// sends the reader's changes to the page's Built-in Synth.

declare global {
  interface Window {
    soundSpy: {
      /** Whether the synth's AudioWorklet node exists, so the page's synth plays what it hears. */
      node: boolean;
      /** Each MIDI message the page passes the synth's worklet. */
      played: number[][];
      /** Each `--level` a channel light took. */
      levels: string[];
    };
  }
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    if (window !== window.top) return;
    const spy: Window["soundSpy"] = { node: false, played: [], levels: [] };
    window.soundSpy = spy;
    const Node = window.AudioWorkletNode;
    window.AudioWorkletNode = class extends Node {
      constructor(...args: ConstructorParameters<typeof AudioWorkletNode>) {
        super(...args);
        spy.node = true;
      }
    };
    const post = MessagePort.prototype.postMessage;
    MessagePort.prototype.postMessage = function (message: any, ...rest: any[]) {
      if (Array.isArray(message?.bytes)) spy.played.push(message.bytes);
      return post.call(this, message, ...(rest as [any]));
    };
  });
});

// Types `expression` under a Bang at the console's Cursor and starts Playback.
async function play(page: Page, expression: string) {
  const frame = page.frameLocator('iframe[title="Orcvs console"]').locator("#the_canvas_id");
  await expect(frame).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
  await frame.click({ position: { x: 300, y: 200 } });
  await page.keyboard.type("**");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.type(expression);
  await page.keyboard.press("Space");
}

/** The text of effect `index`'s two value Cells. */
const valueOf = (page: Page, index: number) =>
  page.locator(`.orcvs-sound-module .number[data-effect="${index}"]`).allTextContents().then((cells) => cells.join(""));

/** The fill of effect `index`'s magnitude, from 0 to 1. */
const magnitudeOf = (page: Page, index: number) =>
  page
    .locator(`.orcvs-sound-module .magnitude[data-effect="${index}"]`)
    .first()
    .evaluate((cell) => (cell as HTMLElement).style.getPropertyValue("--level"));

test("the Sound Module draws its lights and effects at their load values", async ({ page }) => {
  await page.goto("/");

  await expect(page.locator(".orcvs-sound-module .light")).toHaveCount(32);
  await expect(page.locator(".orcvs-sound-module .function")).toHaveText(
    [..."MDOFLNDLCHEQR1R2"],
  );
  const values = await Promise.all([0, 1, 2, 3, 4, 5, 6, 7].map((index) => valueOf(page, index)));
  expect(values).toEqual(["00", "00", "7F", "00", "00", "40", "00", "00"]);
});

test("a Note On from the console fills its channel's light at its velocity", async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => {
    const observer = new MutationObserver((records) => {
      for (const { target } of records) window.soundSpy.levels.push((target as HTMLElement).style.getPropertyValue("--level"));
    });
    const light = document.querySelectorAll(".orcvs-sound-module .light")[0];
    observer.observe(light, { attributes: true, attributeFilter: ["style"] });
  });

  // Channel `00`, velocity `64`.
  await play(page, "!~0064C404");

  await expect.poll(() => page.evaluate(() => window.soundSpy.levels)).toContain(String(0x64 / 0x7f));
});

test("a Control Change from the console shows as its effect's value", async ({ page }) => {
  await page.goto("/");

  // Channel `05`, CC 1, `7F`: any channel sets the effect.
  await play(page, "!c05017F");

  await expect.poll(() => valueOf(page, 0)).toBe("7F");
  expect(await magnitudeOf(page, 0)).toBe("1");
});

test("dragging up on an effect sends its Control Change to the synth", async ({ page }) => {
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => window.soundSpy.node)).toBe(true);

  const magnitude = await page.locator('.orcvs-sound-module .magnitude[data-effect="0"]').first().boundingBox();
  const x = magnitude!.x + magnitude!.width / 2;
  const y = magnitude!.y + magnitude!.height / 2;
  await page.mouse.move(x, y);
  await page.mouse.down();
  // Two pixels a step: twenty steps up.
  await page.mouse.move(x, y - 40, { steps: 8 });
  await page.mouse.up();

  await expect.poll(() => valueOf(page, 0)).toBe("14");
  await expect.poll(() => page.evaluate(() => window.soundSpy.played.at(-1))).toEqual([0xb0, 1, 0x14]);
});

test("a lit channel's digit takes the Theme colour that reads on its fill", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto("/");

  const digits = await page.evaluate(async () => {
    const lights = document.querySelectorAll<HTMLElement>(".orcvs-sound-module .light");
    const time = performance.timeOrigin + performance.now();
    window.postMessage({ type: "orcvs.midi", bytes: [0x93, 60, 0x7f], time }, location.origin);
    window.postMessage({ type: "orcvs.midi", bytes: [0x94, 60, 0x01], time }, location.origin);
    await new Promise((resolve) => setTimeout(resolve, 20));
    return [lights[6].style.color, lights[8].style.color];
  });

  // A full fill of Bang green takes `sound.channel`; a faint one keeps the ordinary colour.
  expect(digits).toEqual(["var(--orcvs-sound-channel)", ""]);
});
