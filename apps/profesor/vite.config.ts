import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

// COOP/COEP habilitan SharedArrayBuffer (input() síncrono en el worker de Python).
// En la app de escritorio los mismos encabezados los pone Tauri (tauri.conf.json).
const aislamiento = {
  "Cross-Origin-Opener-Policy": "same-origin",
  "Cross-Origin-Embedder-Policy": "require-corp",
};

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1421, strictPort: true, headers: aislamiento },
  preview: { port: 1421, strictPort: true, headers: aislamiento },
  worker: { format: "es" },
  build: { target: "es2022", chunkSizeWarningLimit: 2000 },
  envPrefix: ["VITE_", "TAURI_ENV_"],
});
