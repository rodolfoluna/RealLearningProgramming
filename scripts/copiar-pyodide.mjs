// Copia los archivos de Pyodide (CPython en WebAssembly) a public/pyodide de cada app,
// para que funcionen sin internet.
import { cpSync, existsSync, mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const raiz = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(join(raiz, "packages/python-worker/package.json"));
const origen = dirname(require.resolve("pyodide/package.json"));
const archivos = ["pyodide.mjs", "pyodide.asm.mjs", "pyodide.asm.wasm", "python_stdlib.zip", "pyodide-lock.json", "package.json"];
const destinos = process.argv.slice(2).length
  ? process.argv.slice(2)
  : ["apps/alumno/public/pyodide", "apps/alumno-web/public/pyodide", "apps/profesor/public/pyodide", "tests/banco/public/pyodide"];

for (const destino of destinos) {
  const dir = join(raiz, destino);
  mkdirSync(dir, { recursive: true });
  for (const a of archivos) {
    if (!existsSync(join(origen, a))) throw new Error(`Falta ${a} en ${origen}`);
    cpSync(join(origen, a), join(dir, a));
  }
  console.log(`Pyodide → ${destino}`);
}
