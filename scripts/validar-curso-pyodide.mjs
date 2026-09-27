// Valida las soluciones del curso en Pyodide (el mismo Python que corre en la app).
// Los códigos iniciales se revisan en CPython (scripts/validar_curso.py), porque algunos
// contienen ciclos infinitos a propósito.
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const raiz = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(join(raiz, "packages/python-worker/package.json"));
const { loadPyodide } = require("pyodide");
const src = join(raiz, "packages/python-worker/src");
const curso = JSON.parse(readFileSync(join(raiz, "packages/curso/generado/curso-profesor.json"), "utf8"));

const py = await loadPyodide();
py.FS.mkdirTree("/rlp");
py.FS.writeFile("/rlp/harness.py", readFileSync(join(src, "harness.py"), "utf8"));
py.FS.writeFile("/rlp/errores_es.py", readFileSync(join(src, "errores_es.py"), "utf8"));
py.runPython("import sys; sys.path.insert(0, '/rlp'); import harness");
const harness = py.pyimport("harness");
let salida = "";
py.setStdout({ batched: (t) => (salida += t + "\n") });

const errores = [];
let total = 0;
for (const u of curso.unidades)
  for (const l of u.lecciones)
    for (const a of l.actividades) {
      total++;
      if (a.tipo === "codigo") {
        const r = JSON.parse(harness.probar(a.solucion, JSON.stringify(a.pruebas)));
        for (const x of r.resultados) if (!x.paso) errores.push(`${a.id}: «${x.nombre}» ${x.mensaje}`);
      } else if (a.tipo === "prediccion") {
        salida = "";
        py.runPython(a.codigo, { globals: py.toPy({ __name__: "__main__" }) });
        const norm = (t) => harness.normalizar(t);
        if (norm(salida) !== norm(a.salida_esperada)) errores.push(`${a.id}: la predicción no coincide (${JSON.stringify(salida)})`);
      }
    }

if (errores.length) {
  console.error("Errores en Pyodide:\n - " + errores.join("\n - "));
  process.exit(1);
}
console.log(`Curso válido en Pyodide ${py.version}: ${total} actividades revisadas.`);
