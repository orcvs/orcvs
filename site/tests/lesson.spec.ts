import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";
import { holdsStill, send } from "./examples";

// `/learn/` is `docs/first-result.md` rendered at build time, each of its Sources drawn as a Grid
// that opens in the page's small console.

const LESSON = readFileSync("../docs/first-result.md", "utf8");

test("the lesson is docs/first-result.md, heading for heading", async ({ page }) => {
  await page.goto("/learn/");
  const headings = [...LESSON.matchAll(/^## (.+)$/gm)].map((match) => match[1]);
  const shown = await page.locator(".sl-markdown-content").locator("h2").allInnerTexts();
  expect(shown).toEqual(headings);
});

test("each of the lesson's Sources is a Grid that opens it", async ({ page }) => {
  await page.goto("/learn/");
  const sources = [...LESSON.matchAll(/^```text\n([\s\S]*?)^```$/gm)].map((match) => match[1]);
  const sent = await page
    .locator(".orcvs-grid[data-source]")
    .evaluateAll((grids) => grids.map((grid) => (grid as HTMLElement).dataset.source));
  expect(sent).toEqual(sources);
});

test("pressing two Grids opens each in the one small console, stopped", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto("/learn/");
  const grids = page.locator(".orcvs-grid[data-source]");
  const frame = await send(page, grids.nth(0));
  const first = await frame.screenshot();
  await send(page, grids.nth(4));
  await expect(page.locator("iframe")).toHaveCount(1);
  expect((await frame.screenshot()).equals(first)).toBe(false);
  // Stopped: no Tick runs, so the Grid does not change.
  expect(await holdsStill(frame, page, 1_500)).toBe(true);
});
