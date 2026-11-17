import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

// `/learn/` is `docs/first-result.md` rendered at build time, each of its Sources drawn as a Grid
// and the Source it starts from opening in the page's small console.

const LESSON = readFileSync("../docs/first-result.md", "utf8");

test("the lesson is docs/first-result.md, heading for heading", async ({ page }) => {
  await page.goto("/learn/");
  const headings = [...LESSON.matchAll(/^## (.+)$/gm)].map((match) => match[1]);
  const shown = await page.locator(".sl-markdown-content").locator("h2").allInnerTexts();
  expect(shown).toEqual(headings);
});

// Pressing a Grid replaces the console's whole Source, so only the Source the lesson starts from
// opens; the Grids that show a result, or what to type, are drawn but do not open.
test("only the lesson's starting Source opens; the Grids along the way do not", async ({ page }) => {
  await page.goto("/learn/");
  const blocks = [...LESSON.matchAll(/^```text(.*)\n([\s\S]*?)^```$/gm)];
  await expect(page.locator(".orcvs-grid")).toHaveCount(blocks.length);
  const starting = blocks.filter((match) => match[1] === "").map((match) => match[2]);
  expect(starting).toEqual([".+0102\n"]);
  const sent = await page
    .locator(".orcvs-grid[data-source]")
    .evaluateAll((grids) => grids.map((grid) => (grid as HTMLElement).dataset.source));
  expect(sent).toEqual(starting);
});

test("a link to a section of another doc stays on the site", async ({ page }) => {
  await page.goto("/learn/");
  const link = page.locator('.sl-markdown-content a[href="/reference/#built-in-synth"]');
  await expect(link).toHaveCount(1);
  await link.click();
  await expect(page).toHaveURL(/\/reference\/#built-in-synth$/);
  await expect(page.locator("#built-in-synth")).toBeInViewport();
});
