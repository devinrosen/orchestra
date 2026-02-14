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

// Optional label for iterative screenshots: LABEL=after-fix npx playwright test
const label = process.env.LABEL?.trim();

/** Get the next available filename, incrementing if it already exists.
 *  library.png → library-2.png → library-3.png → ... */
function nextScreenshotPath(base: string): string {
  const filePath = path.join(SCREENSHOT_DIR, `${base}.png`);
  if (!fs.existsSync(filePath)) return filePath;
  let n = 2;
  while (fs.existsSync(path.join(SCREENSHOT_DIR, `${base}-${n}.png`))) n++;
  return path.join(SCREENSHOT_DIR, `${base}-${n}.png`);
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(tauriMockScript);
});

test("screenshot all pages", async ({ page }) => {
  // Clean previous screenshots when running without a label (fresh run)
  if (!label && fs.existsSync(SCREENSHOT_DIR)) {
    fs.rmSync(SCREENSHOT_DIR, { recursive: true });
  }
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });

  await page.goto("/");

  // Wait for sidebar to render
  await page.waitForSelector(".sidebar .nav-item", { timeout: 10_000 });

  const taken: string[] = [];

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

    // Build filename: base name, with optional label, auto-incrementing
    const base = label ? `${p.file}-${label}` : p.file;
    const screenshotPath = nextScreenshotPath(base);

    await page.screenshot({
      path: screenshotPath,
      fullPage: false,
    });

    taken.push(screenshotPath);
  }

  // Verify all screenshots were created
  for (const screenshotPath of taken) {
    expect(fs.existsSync(screenshotPath)).toBe(true);
  }
});
