// Estado global de la App Alumno.

import type { Curso } from "@rlp/curso";
import cursoJson from "@rlp/curso/alumno.json";
import type { PoliticaPegado } from "@rlp/editor";
import { EjecutorPython, type OpcionesEjecutor } from "@rlp/python-worker";
import { backend } from "./backend";
import type { EstadoActividad, EstadoAlumno, EstadoApp } from "./tipos";

export const curso = cursoJson as Curso;

export type Seleccion =
  | { tipo: "inicio" }
  | { tipo: "leccion"; id: string }
  | { tipo: "actividad"; id: string }
  | { tipo: "estadisticas" };

export const app = $state({
  vista: "cargando" as "cargando" | "inicio" | "principal",
  estadoApp: null as EstadoApp | null,
  alumno: null as EstadoAlumno | null,
  seleccion: { tipo: "inicio" } as Seleccion,
  aviso: "" as string,
  tema: (localStorageSeguro("rlp-tema") ?? "sistema") as "sistema" | "claro" | "oscuro",
});

/** Pyodide se sirve junto a la app; con `base: "./"` (versión web) la ruta debe ser absoluta. */
function rutaPyodide(): string {
  const base = import.meta.env.BASE_URL;
  return base.startsWith("/") ? `${base}pyodide/` : new URL(`${base}pyodide/`, location.href).href;
}

/** Opciones del intérprete: `iniciarApp` agrega el puente de entrada de la app nativa. */
export const opcionesPython: OpcionesEjecutor = { indexURL: rutaPyodide(), timeoutPruebaMs: 4000 };

/** Intérprete de Python compartido (un solo worker con Pyodide). */
export const python = new EjecutorPython(opcionesPython);

function localStorageSeguro(clave: string): string | null {
  try {
    return localStorage.getItem(clave);
  } catch {
    return null;
  }
}

export function aplicarTema(tema: "sistema" | "claro" | "oscuro") {
  app.tema = tema;
  if (tema === "sistema") document.documentElement.removeAttribute("data-tema");
  else document.documentElement.setAttribute("data-tema", tema);
  try {
    localStorage.setItem("rlp-tema", tema);
  } catch {
    /* sin almacenamiento: solo esta sesión */
  }
}

export async function cargarEstadoApp() {
  app.estadoApp = await (await backend()).estadoApp();
}

export function entrar(estado: EstadoAlumno) {
  app.alumno = estado;
  app.seleccion = { tipo: "inicio" };
  app.vista = "principal";
  void python.iniciar().catch(() => undefined);
}

export async function salir() {
  await (await backend()).cerrarSesion();
  app.alumno = null;
  app.vista = "inicio";
  await cargarEstadoApp();
}

/** Celular o tableta con la app nativa (Android): barra de teclas, QR, "Guardar como". */
export function esMovil(): boolean {
  const p = app.estadoApp?.plataforma;
  return p === "android" || p === "ios";
}

/** Versión web (PWA): los datos viven en el navegador y las entregas se descargan. */
export function esWeb(): boolean {
  return app.estadoApp?.plataforma === "web";
}

/** Exportar a un archivo con nombre ("Guardar como" o descarga) en lugar de elegir una carpeta. */
export function exportarConNombre(): boolean {
  return esMovil() || esWeb();
}

/** Unirse al grupo escaneando el QR que muestra la App Profesor. */
export function puedeEscanearQr(): boolean {
  if (esMovil()) return true;
  return esWeb() && typeof navigator !== "undefined" && !!navigator.mediaDevices?.getUserMedia;
}

/** Dónde se guardan los perfiles, para los textos de la interfaz. */
export function lugarDeDatos(): string {
  return esWeb() ? "este navegador" : esMovil() ? "la app" : "la carpeta de la app";
}

/** Pantalla táctil sin teclado físico (muestra la barra de teclas de código). */
export function esTactil(): boolean {
  if (esMovil()) return true;
  try {
    return matchMedia("(pointer: coarse)").matches;
  } catch {
    return false;
  }
}

/** Fecha para nombres de archivo, igual que en Rust: AAAAMMDD-HHMM. */
export function fechaArchivo(d = new Date()): string {
  const dos = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}${dos(d.getMonth() + 1)}${dos(d.getDate())}-${dos(d.getHours())}${dos(d.getMinutes())}`;
}

export function politicaPegado(): PoliticaPegado {
  return app.alumno?.grupo?.politicas.pegado ?? app.estadoApp?.grupo?.politicas.pegado ?? "bloquear";
}

export function registrarSalidas(): boolean {
  return app.alumno?.grupo?.politicas.registrar_salidas ?? true;
}

export function actualizarActividad(id: string, estado: EstadoActividad) {
  if (app.alumno) app.alumno.actividades[id] = estado;
}

let temporizadorEstadisticas: ReturnType<typeof setTimeout> | null = null;

/** Recalcula las estadísticas (agrupando llamadas seguidas). */
export function refrescarEstadisticas() {
  if (temporizadorEstadisticas) clearTimeout(temporizadorEstadisticas);
  temporizadorEstadisticas = setTimeout(async () => {
    temporizadorEstadisticas = null;
    if (!app.alumno) return;
    try {
      app.alumno.estadisticas = await (await backend()).estadisticas();
    } catch {
      /* se reintentará con el siguiente evento */
    }
  }, 250);
}

/** Registra un evento informativo y actualiza los contadores. */
export async function registrarEvento(tipo: string, actividad: string | null, datos: Record<string, unknown> = {}) {
  try {
    await (await backend()).registrarEvento(tipo, actividad, datos);
  } finally {
    refrescarEstadisticas();
  }
}

/** Tareas a completar antes de cerrar la app (p. ej. guardar lo tecleado en el editor). */
export const alCerrar = new Set<() => Promise<unknown>>();

export async function prepararCierre() {
  await Promise.allSettled([...alCerrar].map((f) => f()));
}

let temporizadorAviso: ReturnType<typeof setTimeout> | null = null;

export function avisar(texto: string, ms = 3500) {
  app.aviso = texto;
  if (temporizadorAviso) clearTimeout(temporizadorAviso);
  temporizadorAviso = setTimeout(() => (app.aviso = ""), ms);
}
