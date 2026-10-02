// Interfaz de la App Alumno, compartida por la app nativa (apps/alumno: Windows y Android) y la
// versión web (apps/alumno-web). Cada una la arranca con `iniciarApp` y su propio núcleo.

import "@rlp/ui-comun/tema.css";
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";
import { opcionesPython } from "./lib/app.svelte";
import { configurarBackend } from "./lib/backend";
import { opciones, type OpcionesApp } from "./lib/opciones";

export type { Backend } from "./lib/backend";
export { backend } from "./lib/backend";
export { app, avisar, curso, prepararCierre, python } from "./lib/app.svelte";
export { conDialogo } from "./lib/dialogos";
export type { OpcionesApp } from "./lib/opciones";
export type * from "./lib/tipos";

export function iniciarApp(o: OpcionesApp, objetivo = document.getElementById("app")!) {
  Object.assign(opciones, o);
  configurarBackend(o.backend);
  opcionesPython.puenteEntrada = o.puenteEntrada;
  // Sin menú contextual del navegador (el editor tiene el suyo, sin "Pegar").
  document.addEventListener("contextmenu", (e) => {
    const t = e.target as HTMLElement;
    if (!t.closest("input, textarea")) e.preventDefault();
  });
  objetivo.replaceChildren(); // quita el "Cargando…" del HTML
  return mount(App, { target: objetivo });
}
