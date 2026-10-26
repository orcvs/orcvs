import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

// A deploy replaces the console a returning reader's service worker has cached. The console's
// files keep fixed names, so the worker must fetch them again rather than answer from its cache.
// The test stands in for a second deploy by changing the served files in place, as a build would:
// a new script, and a page that names the new script's integrity hash.

const SERVED = ["dist/play/index.html", "dist/play/console.js"];

test("a second deploy replaces the console a returning reader cached", async ({ page }) => {
  await page.goto("/play/");
  await expect(page.locator("#the_canvas_id")).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
  await page.reload();
  await expect.poll(() => page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);

  const first = SERVED.map((path) => readFileSync(path));
  try {
    const script = Buffer.concat([first[1], Buffer.from("// second deploy\n")]);
    const integrity = (bytes: Buffer) => `sha384-${createHash("sha384").update(bytes).digest("base64")}`;
    const html = first[0]
      .toString()
      .replaceAll(integrity(first[1]), integrity(script))
      .replace("</head>", '<meta name="deploy" content="second"></head>');
    writeFileSync(SERVED[1], script);
    writeFileSync(SERVED[0], html);

    await page.reload();
    expect(await page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
    await expect(page.locator('meta[name="deploy"]')).toHaveAttribute("content", "second");
    const served = await page.evaluate(async () => (await fetch("/play/console.js")).text());
    expect(served).toContain("// second deploy");
    // The new console starts, so the page and script the reader got belong to one deploy.
    await expect(page.locator("#the_canvas_id")).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
  } finally {
    SERVED.forEach((path, index) => writeFileSync(path, first[index]));
  }
});
