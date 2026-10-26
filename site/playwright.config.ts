import { defineConfig, devices } from "@playwright/test";

// Smoke tests of the built site with the console in it, served the way it is deployed:
// `mise run site_build` first, so `dist/` holds both.
export default defineConfig({
  testDir: "tests",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  reporter: process.env.CI ? "github" : "list",
  use: { baseURL: "http://127.0.0.1:4321" },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    // `--ignore-lock` keeps the server in the foreground: without it, Astro moves a preview it
    // thinks an agent started into the background, and Playwright sees the process exit.
    command: "npx astro preview --host 127.0.0.1 --port 4321 --ignore-lock",
    url: "http://127.0.0.1:4321/",
    reuseExistingServer: !process.env.CI,
  },
});
