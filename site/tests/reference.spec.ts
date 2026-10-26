import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

// `/reference/` is `docs/functions.md` rendered at build time, with "Launch example" after each
// example's starting Source.

const REFERENCE = readFileSync("../docs/functions.md", "utf8");

test("the reference is docs/functions.md, heading for heading", async ({ page }) => {
  await page.goto("/reference/");
  const headings = [...REFERENCE.matchAll(/^#{2,3} (.+)$/gm)].map((match) => match[1].replaceAll("`", ""));
  const shown = await page.locator(".sl-markdown-content").locator("h2, h3").allInnerTexts();
  expect(shown).toEqual(headings);
});

test("every example's starting Source can be launched", async ({ page }) => {
  await page.goto("/reference/");
  const starting = [...REFERENCE.matchAll(/^```orcvs\n([\s\S]*?)^```$/gm)].map(
    (match) => `/play/#source=${encodeURIComponent(match[1])}`,
  );
  const links = await page.locator("figure.orcvs-console").evaluateAll((figures) =>
    figures.map((figure) => (figure as HTMLElement).dataset.link),
  );
  expect(links).toEqual(starting);
});

test("Launch opens a Function's example in the console", async ({ page }) => {
  await page.goto("/reference/");
  const add = page.locator("figure.orcvs-console").first();
  await add.getByRole("button", { name: "Launch example" }).click();
  const frame = add.locator("iframe");
  await expect(frame).toHaveAttribute("src", `/play/#source=${encodeURIComponent(".+0102\n")}`);
  await expect(frame.contentFrame().locator("#the_canvas_id")).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
});

// Pagefind indexes words, so a spelling made only of symbols, such as `.+`, is not searchable;
// `.scratch/orcvs-site/issues/10-find-a-function-by-a-symbol-spelling.md` holds that case.
test("site search finds a Function by its name and by a spelling with a letter", async ({ page }) => {
  await page.goto("/");
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
