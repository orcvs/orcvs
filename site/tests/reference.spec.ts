import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";
import { send } from "./examples";

// `/reference/` is `docs/functions.md` rendered at build time, each Source drawn as a Grid and
// each starting Source opening in the page's small console.

const REFERENCE = readFileSync("../docs/functions.md", "utf8");

test("the reference is docs/functions.md, heading for heading", async ({ page }) => {
  await page.goto("/reference/");
  const headings = [...REFERENCE.matchAll(/^#{2,3} (.+)$/gm)].map((match) => match[1].replaceAll("`", ""));
  const shown = await page.locator(".sl-markdown-content").locator("h2, h3").allInnerTexts();
  expect(shown).toEqual(headings);
});

test("every example is a Grid, and every starting Source opens in the console", async ({ page }) => {
  await page.goto("/reference/");
  const blocks = [...REFERENCE.matchAll(/^```orcvs(.*)\n([\s\S]*?)^```$/gm)];
  await expect(page.locator(".orcvs-grid")).toHaveCount(blocks.length);
  const starting = blocks.filter((match) => match[1] === "").map((match) => match[2]);
  const sent = await page
    .locator(".orcvs-grid[data-source]")
    .evaluateAll((grids) => grids.map((grid) => (grid as HTMLElement).dataset.source));
  expect(sent).toEqual(starting);
});

test("a Grid draws one Cell per character", async ({ page }) => {
  await page.goto("/reference/");
  const add = page.locator(".orcvs-grid").first();
  await expect(add.locator("span")).toHaveText([".", "+", "0", "1", "0", "2"]);
  await expect(add.locator("span.function")).toHaveCount(2);
});

test("pressing an example opens it in the page's small console", async ({ page }) => {
  await page.goto("/reference/");
  await send(page, page.locator(".orcvs-grid[data-source]").first());
});

test("a console that does not start says so, and the next press starts another", async ({ page }) => {
  await page.clock.install();
  await page.route("**/play/?small", (route) => route.fulfill({ contentType: "text/html", body: "" }));
  await page.goto("/reference/");
  const grid = page.locator(".orcvs-grid[data-source]").first();
  await grid.click();
  await expect(page.locator("iframe.orcvs-small")).toHaveCount(1);
  await page.clock.fastForward(30_000);
  await expect(page.getByRole("status")).toHaveText("The console did not start.");
  await expect(page.locator("iframe.orcvs-small")).toHaveCount(0);

  await page.unroute("**/play/?small");
  await page.clock.resume();
  await send(page, grid);
  await expect(page.getByRole("status")).toHaveCount(0);
});

// Pagefind indexes words, so a spelling made only of symbols, such as `.+`, is not searchable;
// `.scratch/orcvs-site/issues/10-find-a-function-by-a-symbol-spelling.md` holds that case.
test("site search finds a Function by its name and by a spelling with a letter", async ({ page }) => {
  await page.goto("/reference/");
  const search = (query: string) =>
    page.evaluate(async (query) => {
      // @ts-expect-error: the bundle is built with the site and has no types here.
      const pagefind = await import("/pagefind/pagefind.js");
      const { results } = await pagefind.search(query);
      const pages = await Promise.all(results.map((result: { data(): Promise<unknown> }) => result.data()));
      return pages.flatMap((data) => (data as { sub_results: { url: string }[] }).sub_results.map((sub) => sub.url));
    }, query);
  expect(await search("Convert to Number")).toContain("/reference/#v-convert-to-number");
  expect(await search(".v")).toContain("/reference/#v-convert-to-number");
});
