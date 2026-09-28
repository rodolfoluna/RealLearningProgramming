// Compila el curso (curso/**/*.yaml + *.md) a JSON:
//  - packages/curso/generado/curso-alumno.json   (sin soluciones)
//  - packages/curso/generado/curso-profesor.json (con soluciones)
// Valida la estructura; las soluciones se validan ejecutándolas con scripts/validar_curso.py.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import yaml from "js-yaml";

const raiz = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const dirCurso = join(raiz, "curso");
const salida = join(raiz, "packages/curso/generado");
const errores = [];

const leerYaml = (ruta) => yaml.load(readFileSync(ruta, "utf8"));
const leerTexto = (ruta) => readFileSync(ruta, "utf8").replace(/\r\n/g, "\n");
const TIPOS = new Set(["codigo", "prediccion", "opcion_multiple"]);
const MODOS = new Set(["contiene", "exacta", "normalizada", "regex", "termina"]);
const ids = new Set();

function unico(id, donde) {
  if (!id || !/^[a-z0-9][a-z0-9-]*$/.test(id)) errores.push(`${donde}: id inválido "${id}" (usa minúsculas, números y guiones)`);
  if (ids.has(id)) errores.push(`${donde}: id repetido "${id}"`);
  ids.add(id);
}

function actividad(a, donde) {
  const d = `${donde} › ${a.id ?? "?"}`;
  unico(a.id, d);
  if (!a.titulo) errores.push(`${d}: falta título`);
  if (!TIPOS.has(a.tipo)) errores.push(`${d}: tipo inválido "${a.tipo}"`);
  if (!a.enunciado) errores.push(`${d}: falta enunciado`);
  const act = {
    id: a.id,
    titulo: a.titulo,
    tipo: a.tipo,
    dificultad: a.dificultad ?? 1,
    puntos: a.puntos ?? (a.tipo === "codigo" ? 10 : 5),
    enunciado: String(a.enunciado ?? "").trim(),
    pistas: a.pistas ?? [],
  };
  if (a.explicacion) act.explicacion = String(a.explicacion).trim();
  if (a.tipo === "codigo") {
    act.codigo_inicial = a.codigo_inicial ?? "";
    act.pruebas = a.pruebas ?? [];
    act.solucion = a.solucion;
    if (!act.pruebas.length) errores.push(`${d}: una actividad de código necesita pruebas`);
    if (!a.solucion) errores.push(`${d}: falta la solución`);
    for (const [i, p] of act.pruebas.entries()) {
      if (p.funcion === undefined && p.salida === undefined) errores.push(`${d}: prueba ${i + 1} sin "salida" ni "funcion"`);
      if (p.modo && !MODOS.has(p.modo)) errores.push(`${d}: prueba ${i + 1} con modo inválido "${p.modo}"`);
      if (p.entrada !== undefined) p.entrada = String(p.entrada);
      if (p.funcion !== undefined) {
        if (p.args !== undefined && !Array.isArray(p.args)) errores.push(`${d}: prueba ${i + 1}: "args" debe ser una lista`);
        if (p.kwargs !== undefined && (typeof p.kwargs !== "object" || Array.isArray(p.kwargs) || p.kwargs === null)) {
          errores.push(`${d}: prueba ${i + 1}: "kwargs" debe ser un diccionario`);
        }
      } else if (p.args !== undefined || p.kwargs !== undefined || p.esperado !== undefined) {
        errores.push(`${d}: prueba ${i + 1}: "args", "kwargs" y "esperado" solo van en pruebas de función`);
      }
    }
  } else if (a.tipo === "prediccion") {
    act.codigo = a.codigo;
    act.salida_esperada = a.salida_esperada;
    if (!a.codigo || a.salida_esperada === undefined) errores.push(`${d}: predicción necesita "codigo" y "salida_esperada"`);
  } else if (a.tipo === "opcion_multiple") {
    act.opciones = a.opciones ?? [];
    if (act.opciones.length < 2 || !act.opciones.some((o) => o.correcta)) {
      errores.push(`${d}: opción múltiple necesita al menos 2 opciones y una correcta`);
    }
  }
  return act;
}

const meta = leerYaml(join(dirCurso, "curso.yaml"));
const curso = { id: meta.id, titulo: meta.titulo, version: String(meta.version), descripcion: meta.descripcion ?? "", unidades: [] };
for (const [n, dirUnidad] of meta.unidades.entries()) {
  const base = join(dirCurso, dirUnidad);
  if (!existsSync(join(base, "unidad.yaml"))) {
    errores.push(`${dirUnidad}: falta unidad.yaml`);
    continue;
  }
  const u = leerYaml(join(base, "unidad.yaml"));
  unico(u.id, dirUnidad);
  const unidad = { id: u.id, numero: u.numero ?? n, titulo: u.titulo, descripcion: u.descripcion ?? "", objetivos: u.objetivos ?? [], lecciones: [] };
  for (const dirLeccion of u.lecciones ?? []) {
    const bl = join(base, dirLeccion);
    const donde = `${dirUnidad}/${dirLeccion}`;
    if (!existsSync(join(bl, "leccion.md"))) {
      errores.push(`${donde}: falta leccion.md`);
      continue;
    }
    const md = leerTexto(join(bl, "leccion.md"));
    const titulo = md.match(/^#\s+(.+)$/m)?.[1]?.trim() ?? dirLeccion;
    const lid = `${u.id}-${dirLeccion.replace(/^l?\d+[-_]?/, "").replace(/_/g, "-")}`;
    const rutaAct = join(bl, "actividades.yaml");
    const acts = existsSync(rutaAct) ? leerYaml(rutaAct) ?? [] : [];
    unidad.lecciones.push({ id: lid, titulo, contenido: md, actividades: acts.map((a) => actividad(a, donde)) });
  }
  curso.unidades.push(unidad);
}

if (errores.length) {
  console.error("Errores en el curso:\n - " + errores.join("\n - "));
  process.exit(1);
}

mkdirSync(salida, { recursive: true });
const sinSoluciones = JSON.parse(JSON.stringify(curso));
for (const u of sinSoluciones.unidades) for (const l of u.lecciones) for (const a of l.actividades) delete a.solucion;
writeFileSync(join(salida, "curso-profesor.json"), JSON.stringify(curso, null, 1));
writeFileSync(join(salida, "curso-alumno.json"), JSON.stringify(sinSoluciones));
const total = curso.unidades.reduce((s, u) => s + u.lecciones.reduce((t, l) => t + l.actividades.length, 0), 0);
console.log(`Curso "${curso.titulo}" v${curso.version}: ${curso.unidades.length} unidades, ${total} actividades.`);
