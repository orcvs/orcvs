import { appendFileSync, readFileSync, writeFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

// A deploy replaces the console a returning reader's service worker has cached. The console's
// files keep fixed names, so the worker must fetch them again rather than answer from its cache.
// The test stands in for a second deploy by changing the served files in place. The changed script
// no longer matches the integrity hash in `index.html`, so the console is not expected to start.

const SERVED = ["dist/play/index.html", "dist/play/console.js"];

test("a second deploy replaces the console a returning reader cached", async ({ page }) => {
  await page.goto("/play/");
  await expect(page.locator("#the_canvas_id")).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
  await page.reload();
  await expect.poll(() => page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);

  const first = SERVED.map((path) => readFileSync(path));
  try {
    writeFileSync(SERVED[0], first[0].toString().replace("</head>", '<meta name="deploy" content="second"></head>'));
    appendFileSync(SERVED[1], "// second deploy\n");

    await page.reload();
    expect(await page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
    await expect(page.locator('meta[name="deploy"]')).toHaveAttribute("content", "second");
    const script = await page.evaluate(async () => (await fetch("/play/console.js")).text());
    expect(script).toContain("// second deploy");
  } finally {
    SERVED.forEach((path, index) => writeFileSync(path, first[index]));
  }
});
