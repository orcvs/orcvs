import { readFileSync } from "node:fs";
import { expect, test, type Locator, type Page } from "@playwright/test";

// The first lesson and the console it embeds. The console draws on a canvas with no DOM, so what
// it shows is compared as screenshots of the frame, and keys reach it as real input.

const LESSON = "/learn/first-result/";

/** The lesson's console that opens the finished Source. */
function finished(page: Page): Locator {
  return page.locator("figure.orcvs-console", { hasText: "The finished Source" });
}

async function launch(figure: Locator): Promise<Locator> {
  await figure.getByRole("button", { name: "Launch example" }).click();
  // The frame takes the button's place; a pointer left over it would hover a Cell.
  await figure.page().mouse.move(0, 0);
  const frame = figure.locator("iframe");
  // eframe sets the canvas's `width` attribute when it starts.
  await expect(frame.contentFrame().locator("#the_canvas_id")).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
  await expect.poll(() => holdsStill(frame, figure.page(), 250)).toBe(true);
  return frame;
}

/** Two screenshots of `frame` taken `ms` apart are the same picture. */
async function holdsStill(frame: Locator, page: Page, ms: number): Promise<boolean> {
  const before = await frame.screenshot();
  await page.waitForTimeout(ms);
  return before.equals(await frame.screenshot());
}

test("Launch opens the lesson's Source in the console, stopped", async ({ page }) => {
  await page.goto(LESSON);
  await expect(page.locator("iframe")).toHaveCount(0);
  const figure = finished(page);
  const frame = await launch(figure);
  expect(await frame.getAttribute("src")).toBe(`/play/#source=${encodeURIComponent(".+0105\n\n\n~.0804\n")}`);
  // Only the console the reader launched starts.
  await expect(page.locator("iframe")).toHaveCount(1);
  // Stopped: no Tick runs, so the Grid does not change.
  expect(await holdsStill(frame, page, 1_500)).toBe(true);
});

test("a console that fails to load leaves the lesson and says so", async ({ page }) => {
  await page.clock.install();
  await page.route("**/play/console_bg.wasm", (route) => route.abort());
  await page.goto(LESSON);
  const figure = finished(page);
  await figure.getByRole("button", { name: "Launch example" }).click();
  // `runFor`, not `fastForward`: the component's check reschedules itself every 250 ms.
  await page.clock.runFor(21_000);
  await expect(figure.getByRole("alert")).toBeVisible();
  await expect(figure.getByRole("alert")).toContainText("could not start");
  await expect(figure.locator("iframe")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Write an Addition" })).toBeVisible();
});

// Pressing Launch is the reader's click on the console: the console takes the keyboard when it
// starts. Until then, and after a click outside it, keys belong to the page.
test("keys scroll the page until the console is clicked, and return to it after", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto(LESSON);
  const scrollY = () => page.evaluate(() => window.scrollY);
  const moves = async (frame: Locator) => {
    // A console that just lost or gained focus repaints once; wait for it.
    await expect.poll(() => holdsStill(frame, page, 250)).toBe(true);
    const before = await frame.screenshot();
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowDown");
    await page.waitForTimeout(250);
    return !before.equals(await frame.screenshot());
  };

  // Before a click: the page scrolls.
  await page.keyboard.press("PageDown");
  await expect.poll(scrollY).toBeGreaterThan(0);

  // After one click: arrow keys move the Cursor, and the page does not scroll.
  const frame = await launch(finished(page));
  const launchedY = await scrollY();
  expect(await moves(frame)).toBe(true);
  await page.keyboard.press("PageDown");
  expect(await scrollY()).toBe(launchedY);

  // A click outside gives the keys back to the page: the Cursor stays and the page scrolls.
  await page.locator("main p").first().click();
  expect(await moves(frame)).toBe(false);
  const outsideY = await scrollY();
  await page.keyboard.press("PageDown");
  await expect.poll(scrollY).toBeGreaterThan(outsideY);

  // A click inside takes them again.
  await frame.scrollIntoViewIfNeeded();
  await frame.click({ position: { x: 40, y: 60 } });
  await page.mouse.move(0, 0);
  expect(await moves(frame)).toBe(true);
});

// The console keeps the keyboard when Tab is pressed in it, so this is expected to fail until
// `.scratch/orcvs-site/issues/11-give-keys-back-to-the-page-on-tab.md` is done; Playwright then
// reports it as passing unexpectedly, which is the signal to drop `test.fail`.
test("Tab in the console gives keys back to the page", async ({ page }) => {
  test.fail();
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto(LESSON);
  const frame = await launch(finished(page));
  await expect.poll(() => holdsStill(frame, page, 250)).toBe(true);
  const before = await frame.screenshot();
  await page.keyboard.press("Tab");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowDown");
  await page.waitForTimeout(250);
  expect((await frame.screenshot()).equals(before)).toBe(true);
});

test("a narrow screen offers the full console instead of an embed", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(LESSON);
  const figure = finished(page);
  await expect(figure.getByRole("button", { name: "Launch example" })).toBeHidden();
  await expect(figure.getByRole("link", { name: "Open full console" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
});

test("the lesson's steps are those of docs/first-result.md", () => {
  const blocks = (path: string) =>
    [...readFileSync(path, "utf8").matchAll(/^```text\n([\s\S]*?)^```$/gm)].map((match) => match[1]);
  const doc = blocks("../docs/first-result.md");
  expect(doc.length).toBeGreaterThan(0);
  expect(blocks("src/content/docs/learn/first-result.mdx")).toEqual(doc);
});
