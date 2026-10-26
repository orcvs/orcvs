import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

// Cloudflare's build reads the Node version from `.node-version`, and every other build from the
// `mise.toml` pin, so the two must name the same version.
test("Cloudflare's build runs the Node version mise pins", () => {
  const pin = readFileSync("../mise.toml", "utf8").match(/^node = "(.+)"$/m)?.[1];
  expect(pin).toBeDefined();
  expect(readFileSync(".node-version", "utf8").trim()).toBe(pin);
});
