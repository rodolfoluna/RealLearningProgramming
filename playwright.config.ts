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
      command: "pnpm --filter @rlp/alumno-web exec vite --port 1422 --strictPort",
      url: "http://localhost:1422",
      reuseExistingServer: true,
      timeout: 60_000,
    },
    // Sitio web compilado (apps/alumno-web/dist), para WebKit: ver el proyecto "web-webkit".
    ...(process.env.PLAYWRIGHT_WEBKIT
      ? [
          {
            command: "pnpm --filter @rlp/alumno-web exec vite preview --port 1423 --strictPort",
            url: "http://localhost:1423",
            reuseExistingServer: true,
            timeout: 60_000,
          },
        ]
      : []),
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
      // Versión web (PWA): núcleo real en WebAssembly e IndexedDB (requiere `pnpm wasm`).
      name: "web",
      testMatch: /web\..*spec\.ts/,
      use: { baseURL: "http://localhost:1422", viewport: { width: 1366, height: 800 } },
    },
    // La versión web también en WebKit (el motor de Safari), si está instalado (CI). Usa el sitio
    // compilado, que es lo que reciben los alumnos: el servidor de desarrollo de Vite sirve los
    // módulos del worker de Python (/@fs/…?import&raw) de una forma que WebKit bloquea bajo COEP.
    ...(process.env.PLAYWRIGHT_WEBKIT
      ? [
          {
            name: "web-webkit",
            testMatch: /web\..*spec\.ts/,
            use: {
              browserName: "webkit" as const,
              baseURL: "http://localhost:1423",
              viewport: { width: 1366, height: 800 },
              permissions: [],
              launchOptions: {},
            },
          },
        ]
      : []),
    {
      name: "profesor",
      testMatch: /profesor\..*spec\.ts/,
      use: { baseURL: "http://localhost:1421", viewport: { width: 1440, height: 900 } },
    },
  ],
});
