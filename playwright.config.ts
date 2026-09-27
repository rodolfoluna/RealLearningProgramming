import { defineConfig } from "@playwright/test";

// Chromium es el motor de WebView2 (Windows) y de Android System WebView.
export default defineConfig({
  testDir: "tests/e2e",
  timeout: 90_000,
  expect: { timeout: 20_000 },
  fullyParallel: false,
  workers: 1,
  reporter: [["list"]],
  use: {
    browserName: "chromium",
    permissions: ["clipboard-read", "clipboard-write"],
    launchOptions: process.env.PLAYWRIGHT_CHROMIUM ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM } : {},
  },
  webServer: [
    {
      command: "pnpm --filter @rlp/banco exec vite --port 5199 --strictPort",
      url: "http://localhost:5199",
      reuseExistingServer: true,
      timeout: 60_000,
    },
    {
      command: "pnpm --filter @rlp/alumno exec vite --port 1420 --strictPort",
      url: "http://localhost:1420",
      reuseExistingServer: true,
      timeout: 60_000,
    },
    {
      command: "pnpm --filter @rlp/profesor exec vite --port 1421 --strictPort",
      url: "http://localhost:1421",
      reuseExistingServer: true,
      timeout: 60_000,
    },
  ],
  projects: [
    { name: "banco", testMatch: /banco\..*spec\.ts/, use: { baseURL: "http://localhost:5199" } },
    {
      name: "alumno",
      testMatch: /alumno\..*spec\.ts/,
      use: { baseURL: "http://localhost:1420", viewport: { width: 1366, height: 800 } },
    },
    {
      name: "profesor",
      testMatch: /profesor\..*spec\.ts/,
      use: { baseURL: "http://localhost:1421", viewport: { width: 1440, height: 900 } },
    },
  ],
});
