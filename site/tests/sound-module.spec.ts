import { expect, test, type Page } from "@playwright/test";

// The Sound Module below the console on `/`: it shows the console's notes and Control Changes, and
// sends the reader's changes to the page's Built-in Synth, each on the channel it chose.

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

// An empty Source, so only the expression `play` types sends anything, and `play`'s click lands
// on an empty Cell whatever example a first visit opens.
const EMPTY = "/#source=";

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
    [..."MDEQLNOFR1R2CHDL"],
  );
  const values = await Promise.all([0, 1, 2, 3, 4, 5, 6, 7].map((index) => valueOf(page, index)));
  expect(values).toEqual(["00", "40", "7F", "00", "00", "00", "00", "00"]);
  await expect(page.locator(".orcvs-sound-module .channel.chosen")).toHaveText("0");
});

test("each effect's Cells name the effect and its Control Change on hover", async ({ page }) => {
  await page.goto("/");

  const titles = await page
    .locator(".orcvs-sound-module .function")
    .evaluateAll((cells) => cells.filter((_, index) => index % 2 === 0).map((cell) => (cell as HTMLElement).title));
  expect(titles).toEqual([
    "Modulation, CC 1",
    "EQ, CC 74",
    "Length, CC 75",
    "Offset, CC 85",
    "Reverb 1, CC 91",
    "Reverb 2, CC 92",
    "Chorus, CC 93",
    "Delay, CC 94",
  ]);
  await expect(page.locator('.orcvs-sound-module .magnitude[data-effect="6"]').first()).toHaveAttribute("title", "Chorus, CC 93");
});

test("a Note On from the console shows its channel as sounding, then idle", async ({ page }) => {
  await page.goto(EMPTY);
  await page.evaluate(() => {
    const channel = document.querySelectorAll(".orcvs-sound-module .channel")[0];
    const observer = new MutationObserver(() => window.soundSpy.sounding.push(channel.classList.contains("sounding")));
    observer.observe(channel, { attributes: true, attributeFilter: ["class"] });
  });

  // Channel `00`.
  await play(page, "!~0064C404");

  await expect.poll(() => page.evaluate(() => window.soundSpy.sounding.slice(0, 2))).toEqual([true, false]);
});

test("a Control Change from the console shows as its effect's value on its channel", async ({ page }) => {
  await page.goto(EMPTY);

  // Channel `05`, CC 1, `7F`: the effect is channel `05`'s, so channel `00` keeps `00`.
  await page.locator(".orcvs-sound-module .channel").nth(5).click();
  await expect(page.locator(".orcvs-sound-module .channel.chosen")).toHaveText("5");
  await play(page, "!c05017F");

  await expect.poll(() => valueOf(page, 0)).toBe("7F");
  expect(await magnitudeOf(page, 0)).toBe("1");
  await page.locator(".orcvs-sound-module .channel").nth(0).click();
  expect(await valueOf(page, 0)).toBe("00");
});

test("the acid voice's channel shows its cutoff and decay where EQ and LN stand", async ({ page }) => {
  await page.goto("/");

  await page.locator(".orcvs-sound-module .channel").nth(8).click();

  // CC 74 and 75 on `08` are the acid voice's cutoff and decay, which load at `30`.
  await expect(page.locator(".orcvs-sound-module .function")).toHaveText([..."MDCUDCOFR1R2CHDL"]);
  await expect(page.locator('.orcvs-sound-module .function[data-effect="1"]').first()).toHaveAttribute("title", "Cutoff, CC 74");
  await expect(page.locator('.orcvs-sound-module .function[data-effect="2"]').first()).toHaveAttribute("title", "Decay, CC 75");
  expect(await valueOf(page, 1)).toBe("30");
  expect(await valueOf(page, 2)).toBe("30");
});

test("Reset All Controllers clears Modulation and leaves the other effects, as the synth does", async ({ page }) => {
  await page.goto("/");
  await page.locator(".orcvs-sound-module .channel").nth(8).click();

  const post = (bytes: number[]) =>
    page.evaluate((bytes) => window.postMessage({ type: "orcvs.midi", bytes, time: 0 }, location.origin), bytes);
  await post([0xb8, 1, 0x7f]);
  await post([0xb8, 74, 0x7f]);
  await post([0xb8, 91, 0x40]);
  await expect.poll(() => valueOf(page, 4)).toBe("40");
  // Stop sends Reset All Controllers on every channel; the chosen channel stays where it is.
  await post([0xb8, 121, 0]);
  await post([0xbf, 121, 0]);

  await expect.poll(() => valueOf(page, 0)).toBe("00");
  expect(await valueOf(page, 1)).toBe("7F");
  expect(await valueOf(page, 4)).toBe("40");
  await expect(page.locator(".orcvs-sound-module .channel.chosen")).toHaveText("8");
});

test("Reset All Controllers moves no choice the effects followed", async ({ page }) => {
  await page.goto("/");

  // Nothing is picked, so `MD` on `08` takes the choice there; Stop's reset on `0F` leaves it.
  const post = (bytes: number[]) =>
    page.evaluate((bytes) => window.postMessage({ type: "orcvs.midi", bytes, time: 0 }, location.origin), bytes);
  await post([0xb8, 1, 0x7f]);
  await expect(page.locator(".orcvs-sound-module .channel.chosen")).toHaveText("8");
  await post([0xbf, 121, 0]);
  // A Note On after the reset, which moves no choice, shows when the reset has been heard.
  await post([0x90, 60, 100]);

  await expect(page.locator(".orcvs-sound-module .channel").nth(0)).toHaveClass(/sounding/);
  await expect(page.locator(".orcvs-sound-module .channel.chosen")).toHaveText("8");
});

test("until the reader chooses a channel, the effects follow the channel a Control Change is on", async ({ page }) => {
  await page.goto(EMPTY);

  await play(page, "!c05017F");

  await expect(page.locator(".orcvs-sound-module .channel.chosen")).toHaveText("5");
  await expect.poll(() => valueOf(page, 0)).toBe("7F");
});

test("the keyboard chooses a channel", async ({ page }) => {
  await page.goto("/");

  await page.locator(".orcvs-sound-module .channel").nth(9).focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(".orcvs-sound-module .channel.chosen")).toHaveText("9");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press(" ");
  await expect(page.locator(".orcvs-sound-module .channel.chosen")).toHaveText("A");
  await expect(page.locator(".orcvs-sound-module .channel.chosen")).toHaveAttribute("aria-pressed", "true");

  // A hex digit typed on a channel writes no value.
  await page.keyboard.type("4");
  expect(await valueOf(page, 0)).toBe("00");
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

  // Past `MD`'s last digit, the Cursor steps onto `EQ`'s first.
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.type("2");
  await expect.poll(() => valueOf(page, 1)).toBe("20");

  await page.keyboard.press("Escape");
  await expect(page.locator(".orcvs-sound-module .cursor")).toHaveCount(0);
});

test("typing on a value sends its Control Change on the chosen channel", async ({ page }) => {
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => window.soundSpy.node)).toBe(true);

  // Channel `09`'s `R1` takes `40`; channel `00`'s keeps its own.
  await page.locator(".orcvs-sound-module .channel").nth(9).click();
  await page.locator('.orcvs-sound-module .number[data-effect="4"]').first().click();
  await page.keyboard.type("40");
  await expect.poll(() => page.evaluate(() => window.soundSpy.played.at(-1))).toEqual([0xb9, 91, 0x40]);

  await page.locator(".orcvs-sound-module .channel").nth(0).click();
  expect(await valueOf(page, 4)).toBe("00");
  await page.locator(".orcvs-sound-module .channel").nth(9).click();
  expect(await valueOf(page, 4)).toBe("40");
});
