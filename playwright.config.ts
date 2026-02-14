import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  outputDir: "../test-results",
  timeout: 30_000,
  use: {
    baseURL: "http://localhost:1420",
    headless: true,
    viewport: { width: 1280, height: 800 },
    screenshot: "off",
  },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],
  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: true,
    timeout: 30_000,
  },
});
