import { expect, test, type Page } from "@playwright/test";

// What the deployed site must always do: every top-level page loads with no page errors, and the
// console at `/play/` starts. The console draws on a canvas with no DOM, so "starts" is the canvas
// given a `width` attribute, which eframe sets once it runs and a page whose WASM failed never has.

function collectErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  return errors;
}

for (const path of ["/", "/learn/", "/reference/", "/about/"]) {
  test(`${path} loads with no errors`, async ({ page }) => {
    const errors = collectErrors(page);
    const response = await page.goto(path);
    expect(response?.status()).toBe(200);
    await expect(page.locator("main")).toBeVisible();
    expect(errors).toEqual([]);
  });
}

async function consoleStarts(page: Page) {
  await expect(page.locator("#the_canvas_id")).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
}

test("/play/ starts the console and registers its service worker", async ({ page }) => {
  const errors = collectErrors(page);
  await page.goto("/play/");
  await consoleStarts(page);
  await expect
    .poll(() => page.evaluate(async () => (await navigator.serviceWorker.getRegistrations()).length))
    .toBe(1);
  expect(errors).toEqual([]);
});

// `console/index.html` decides whether to register the worker before the console reads its URL,
// so each link form `console/src/linked_source.rs` opens is checked here too.
for (const link of ["/play/?example=cell-tracker", "/play/#source=.%2B0102%0A"]) {
  test(`${link} starts and registers no service worker`, async ({ page }) => {
    const errors = collectErrors(page);
    await page.goto(link);
    await consoleStarts(page);
    expect(await page.evaluate(async () => (await navigator.serviceWorker.getRegistrations()).length)).toBe(0);
    expect(errors).toEqual([]);
  });
}
