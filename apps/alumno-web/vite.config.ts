import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

// COOP/COEP habilitan SharedArrayBuffer (input() síncrono en el worker de Python). En el sitio
// publicado los agrega el service worker, así que sirve en cualquier hosting.
const aislamiento = {
  "Cross-Origin-Opener-Policy": "same-origin",
  "Cross-Origin-Embedder-Policy": "require-corp",
};

export default defineConfig({
  // Rutas relativas: el sitio funciona en la raíz de un dominio o en una subcarpeta (GitHub Pages).
  base: "./",
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1422, strictPort: true, headers: aislamiento },
  preview: { port: 1422, strictPort: true, headers: aislamiento },
  worker: { format: "es" },
  build: { target: "es2022", chunkSizeWarningLimit: 2000 },
});
