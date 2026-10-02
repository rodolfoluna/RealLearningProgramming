import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig, type Plugin } from "vite";

const raiz = fileURLToPath(new URL(".", import.meta.url));

// COOP/COEP habilitan SharedArrayBuffer (input() síncrono en el worker de Python). En el sitio
// publicado los agrega el service worker, así que sirve en cualquier hosting.
const aislamiento = {
  "Cross-Origin-Opener-Policy": "same-origin",
  "Cross-Origin-Embedder-Policy": "require-corp",
};

function listar(dir: string): string[] {
  return readdirSync(dir).flatMap((nombre) => {
    const ruta = join(dir, nombre);
    return statSync(ruta).isDirectory() ? listar(ruta) : [ruta];
  });
}

/** Genera dist/sw.js a partir de la plantilla sw.js con los archivos y la versión de esta compilación. */
function serviceWorker(): Plugin {
  return {
    name: "rlp-service-worker",
    apply: "build",
    closeBundle() {
      const dist = join(raiz, "dist");
      const archivos = listar(dist)
        .map((r) => relative(dist, r).split(sep).join("/"))
        .filter((r) => r !== "sw.js")
        .sort();
      const app = archivos.filter((r) => !r.startsWith("pyodide/"));
      const pyodide = archivos.filter((r) => r.startsWith("pyodide/"));
      const huella = createHash("sha256");
      for (const r of app) huella.update(r).update(readFileSync(join(dist, r)));
      const versionPyodide = JSON.parse(readFileSync(join(dist, "pyodide/package.json"), "utf8")).version;
      const sw = readFileSync(join(raiz, "sw.js"), "utf8")
        .replace("__VERSION__", huella.digest("hex").slice(0, 12))
        .replace("__VERSION_PYODIDE__", versionPyodide)
        .replace("__ARCHIVOS_APP__", JSON.stringify(app))
        .replace("__ARCHIVOS_PYODIDE__", JSON.stringify(pyodide));
      writeFileSync(join(dist, "sw.js"), sw);
    },
  };
}

export default defineConfig({
  // Rutas relativas: el sitio funciona en la raíz de un dominio o en una subcarpeta (GitHub Pages).
  base: "./",
  plugins: [svelte(), serviceWorker()],
  clearScreen: false,
  server: { port: 1422, strictPort: true, headers: aislamiento },
  preview: { port: 1422, strictPort: true, headers: aislamiento },
  worker: { format: "es" },
  build: { target: "es2022", chunkSizeWarningLimit: 2000 },
});
