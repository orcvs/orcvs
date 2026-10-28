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
      /** Whether channel `00` showed as sounding, each time its class changed. */
      sounding: boolean[];
    };
  }
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    if (window !== window.top) return;
    const spy: Window["soundSpy"] = { node: false, played: [], sounding: [] };
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

  await expect(page.locator(".orcvs-sound-module .channel")).toHaveText([..."0123456789ABCDEF"]);
  await expect(page.locator(".orcvs-sound-module .function")).toHaveText(
    [..."MDOFLNDLCHEQR1R2"],
  );
  const values = await Promise.all([0, 1, 2, 3, 4, 5, 6, 7].map((index) => valueOf(page, index)));
  expect(values).toEqual(["00", "00", "7F", "00", "00", "40", "00", "00"]);
});

test("a Note On from the console shows its channel as sounding, then idle", async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => {
    const channel = document.querySelectorAll(".orcvs-sound-module .channel")[0];
    const observer = new MutationObserver(() => window.soundSpy.sounding.push(channel.classList.contains("sounding")));
    observer.observe(channel, { attributes: true, attributeFilter: ["class"] });
  });

  // Channel `00`.
  await play(page, "!~0064C404");

  await expect.poll(() => page.evaluate(() => window.soundSpy.sounding.slice(0, 2))).toEqual([true, false]);
});

test("a Control Change from the console shows as its effect's value", async ({ page }) => {
  await page.goto("/");

  // Channel `05`, CC 1, `7F`: any channel sets the effect.
  await play(page, "!c05017F");

  await expect.poll(() => valueOf(page, 0)).toBe("7F");
  expect(await magnitudeOf(page, 0)).toBe("1");
});

test("typing on a value writes it as the Grid does, and sends its Control Change", async ({ page }) => {
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => window.soundSpy.node)).toBe(true);

  // `MD`'s first digit takes the Cursor.
  const first = page.locator('.orcvs-sound-module .number[data-effect="0"]').first();
  await first.click();
  await expect(first).toHaveClass(/cursor/);

  // `6` then `4` write `64` and step east; `8` would make `84`, past `7F`, and is refused.
  await page.keyboard.type("6");
  await expect.poll(() => valueOf(page, 0)).toBe("60");
  await page.keyboard.type("4");
  await expect.poll(() => valueOf(page, 0)).toBe("64");
  await expect.poll(() => page.evaluate(() => window.soundSpy.played.at(-1))).toEqual([0xb0, 1, 0x64]);
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.type("8");
  expect(await valueOf(page, 0)).toBe("64");

  // Past `MD`'s last digit, the Cursor steps onto `OF`'s first.
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.type("2");
  await expect.poll(() => valueOf(page, 1)).toBe("20");

  await page.keyboard.press("Escape");
  await expect(page.locator(".orcvs-sound-module .cursor")).toHaveCount(0);
});
