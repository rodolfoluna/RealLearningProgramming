// Lo que cada cáscara de la App Alumno (nativa o web) le da a la interfaz compartida.

import type { PuenteEntrada } from "@rlp/python-worker";
import type { Backend } from "./backend";

export interface OpcionesApp {
  /** Núcleo: comandos de Tauri, WebAssembly (versión web) o simulación. */
  backend: () => Promise<Backend>;
  /** Entrada síncrona para `input()` cuando no hay SharedArrayBuffer (puente de la app nativa). */
  puenteEntrada?: PuenteEntrada;
  /** Al montar la interfaz (p. ej. escuchar el cierre de la ventana). */
  alIniciar?: () => void | Promise<void>;
  /** Autoprueba de extremo a extremo, si el núcleo indica una fase. */
  autoprueba?: (fase: string) => Promise<void>;
}

export const opciones: Partial<OpcionesApp> = {};
