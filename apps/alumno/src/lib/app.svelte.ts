// Estado global de la App Alumno.

import type { Curso } from "@rlp/curso";
import cursoJson from "@rlp/curso/alumno.json";
import type { PoliticaPegado } from "@rlp/editor";
import { EjecutorPython, type PuenteEntrada } from "@rlp/python-worker";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
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

/** Intérprete de Python compartido (un solo worker con Pyodide). */
export const python = new EjecutorPython({ indexURL: "/pyodide/", timeoutPruebaMs: 4000, puenteEntrada: puente() });

/** Entrada síncrona vía Rust para WebView sin SharedArrayBuffer (p. ej. WebKitGTK). */
function puente(): PuenteEntrada | undefined {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return undefined;
  return {
    url: convertFileSrc("", "rlpentrada"),
    enviar: (linea) => invoke("entrada_enviar", { linea }),
    cancelar: () => invoke("entrada_cancelar"),
  };
}

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

let temporizadorAviso: ReturnType<typeof setTimeout> | null = null;

export function avisar(texto: string, ms = 3500) {
  app.aviso = texto;
  if (temporizadorAviso) clearTimeout(temporizadorAviso);
  temporizadorAviso = setTimeout(() => (app.aviso = ""), ms);
}
