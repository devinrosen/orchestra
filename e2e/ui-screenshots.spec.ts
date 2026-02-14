import { test, expect } from "@playwright/test";
import { tauriMockScript } from "./tauri-mocks";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SCREENSHOT_DIR = path.resolve(__dirname, "../../test-results/screenshots");

const allPages = [
  { nav: "Library", file: "library" },
  { nav: "Statistics", file: "statistics" },
  { nav: "Playlists", file: "playlists" },
  { nav: "Sync Profiles", file: "profiles" },
  { nav: "Devices", file: "devices" },
  { nav: "Settings", file: "settings" },
] as const;

// Filter pages via PAGES env var: PAGES=library,settings npx playwright test
const pageFilter = process.env.PAGES?.split(",").map((s) => s.trim().toLowerCase());
const pages = pageFilter
  ? allPages.filter((p) => pageFilter.includes(p.file))
  : allPages;

test.beforeEach(async ({ page }) => {
  await page.addInitScript(tauriMockScript);
});

test("screenshot all pages", async ({ page }) => {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });

  await page.goto("/");

  // Wait for sidebar to render
  await page.waitForSelector(".sidebar .nav-item", { timeout: 10_000 });

  for (const p of pages) {
    // Click the nav button
    await page.click(`.nav-item:has-text("${p.nav}")`);

    // Wait for content to settle
    await page.waitForTimeout(500);

    // Wait for any loading states to clear
    await page.waitForFunction(
      () => !document.querySelector(".content")?.textContent?.includes("Loading"),
      { timeout: 5_000 },
    ).catch(() => {
      // Some pages may not have a loading state — that's fine
    });

    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, `${p.file}.png`),
      fullPage: false,
    });
  }

  // Verify all screenshots were created
  for (const p of pages) {
    const screenshotPath = path.join(SCREENSHOT_DIR, `${p.file}.png`);
    expect(fs.existsSync(screenshotPath)).toBe(true);
  }
});
