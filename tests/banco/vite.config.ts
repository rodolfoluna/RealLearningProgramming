import { defineConfig } from "vite";

// COOP/COEP habilitan SharedArrayBuffer (input() síncrono en el worker de Python).
const aislamiento = {
  "Cross-Origin-Opener-Policy": "same-origin",
  "Cross-Origin-Embedder-Policy": "require-corp",
};

export default defineConfig({
  server: { port: 5199, strictPort: true, headers: aislamiento },
  preview: { port: 5199, strictPort: true, headers: aislamiento },
  worker: { format: "es" },
});
