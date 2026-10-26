import { expect, type Locator, type Page } from "@playwright/test";

// The page's small console (`src/console/small.ts`). It draws on a canvas with no DOM, so what it
// shows is compared as screenshots of its frame.

/** The page's small console, once a press of `grid` has started it and sent it the Source. */
export async function send(page: Page, grid: Locator): Promise<Locator> {
  await grid.click();
  const frame = page.locator("iframe.orcvs-small");
  await expect(frame).toHaveAttribute("src", "/play/?small");
  await expect(frame.contentFrame().locator("#the_canvas_id")).toHaveAttribute("width", /\d+/, { timeout: 30_000 });
  await page.mouse.move(0, 0);
  await expect.poll(() => holdsStill(frame, page, 250)).toBe(true);
  return frame;
}

/** Two screenshots of `frame` taken `ms` apart are the same picture. */
export async function holdsStill(frame: Locator, page: Page, ms: number): Promise<boolean> {
  const before = await frame.screenshot();
  await page.waitForTimeout(ms);
  return before.equals(await frame.screenshot());
}
