/// <reference lib="webworker" />
// Web Worker que aloja Pyodide (CPython en WebAssembly) y ejecuta el código del alumno.
// Corre en su propio hilo: un ciclo infinito no congela la interfaz y el hilo principal
// puede interrumpirlo (búfer de interrupción) o terminarlo por completo.

import type { PyodideAPI } from "pyodide";
import harnessPy from "./harness.py?raw";
import erroresPy from "./errores_es.py?raw";
import { CONTROL, type MensajeAlWorker, type MensajeDelWorker } from "./tipos";

declare const self: DedicatedWorkerGlobalScope;

const LIMITE_SALIDA = 200_000; // caracteres por ejecución
const decodificador = new TextDecoder();

let pyodide: PyodideAPI;
let harness: any;
let control: Int32Array;
let datos: Uint8Array;
let interrupcion: Uint8Array;

let idActual = 0;
let pendiente: { out: string; err: string } = { out: "", err: "" };
let ultimoEnvio = 0;
let totalSalida = 0;
let excedida = false;

function enviar(m: MensajeDelWorker) {
  self.postMessage(m);
}

function vaciarSalida() {
  for (const flujo of ["out", "err"] as const) {
    if (pendiente[flujo]) {
      enviar({ tipo: "salida", id: idActual, texto: pendiente[flujo], flujo });
      pendiente[flujo] = "";
    }
  }
  ultimoEnvio = performance.now();
}

function escritor(flujo: "out" | "err") {
  return {
    write(buffer: Uint8Array): number {
      if (!excedida) {
        const texto = decodificador.decode(buffer, { stream: true });
        totalSalida += texto.length;
        pendiente[flujo] += texto;
        if (totalSalida > LIMITE_SALIDA) {
          excedida = true;
          interrupcion[0] = 2; // provoca KeyboardInterrupt en el programa
        }
        if (pendiente[flujo].length > 4096 || performance.now() - ultimoEnvio > 30) vaciarSalida();
      }
      return buffer.length;
    },
  };
}

function leerEntrada(): string | null {
  vaciarSalida();
  Atomics.store(control, CONTROL.ESTADO_ENTRADA, 0);
  enviar({ tipo: "entrada", id: idActual });
  Atomics.wait(control, CONTROL.ESTADO_ENTRADA, 0);
  if (Atomics.load(control, CONTROL.ESTADO_ENTRADA) !== 1) return null; // cancelada → EOF
  const n = Atomics.load(control, CONTROL.LONGITUD);
  const linea = decodificador.decode(datos.slice(0, n));
  return linea + "\n";
}

async function iniciar(indexURL: string) {
  const modulo = await import(/* @vite-ignore */ `${indexURL}pyodide.mjs`);
  pyodide = (await modulo.loadPyodide({ indexURL, fullStdLib: false })) as PyodideAPI;
  pyodide.setStdout(escritor("out"));
  pyodide.setStderr(escritor("err"));
  pyodide.setStdin({ stdin: leerEntrada });
  pyodide.setInterruptBuffer(interrupcion);

  pyodide.FS.mkdirTree("/rlp");
  pyodide.FS.writeFile("/rlp/harness.py", harnessPy);
  pyodide.FS.writeFile("/rlp/errores_es.py", erroresPy);
  pyodide.globals.set("_rlp_esperar", (segundos: number) => {
    Atomics.wait(control, CONTROL.DORMIR, 0, Math.max(0, segundos * 1000));
  });
  pyodide.globals.set("_rlp_notificar", (indice: number) => {
    enviar({ tipo: "prueba_inicio", id: idActual, indice });
  });
  pyodide.globals.set("_rlp_reportar", (resultado: string) => {
    enviar({ tipo: "prueba_fin", id: idActual, resultado });
  });
  pyodide.globals.set("_rlp_limpiar", () => {
    vaciarSalida();
    enviar({ tipo: "limpiar", id: idActual });
  });
  pyodide.runPython(`
import sys, time
sys.path.insert(0, "/rlp")
import harness

def _rlp_dormir(segundos):
    fin = time.monotonic() + max(0.0, float(segundos))
    while True:
        restante = fin - time.monotonic()
        if restante <= 0:
            break
        _rlp_esperar(min(restante, 0.05))  # al volver a Python se revisa la interrupción

harness.configurar(notificar=_rlp_notificar, reportar=_rlp_reportar, dormir=_rlp_dormir, limpiar=_rlp_limpiar)
`);
  harness = pyodide.pyimport("harness");
  enviar({ tipo: "listo", version: pyodide.version });
}

function prepararEjecucion(id: number) {
  idActual = id;
  pendiente = { out: "", err: "" };
  totalSalida = 0;
  excedida = false;
  ultimoEnvio = performance.now();
  interrupcion[0] = 0;
}

self.onmessage = async (evento: MessageEvent<MensajeAlWorker>) => {
  const m = evento.data;
  switch (m.tipo) {
    case "iniciar":
      control = new Int32Array(m.control, 0, 4);
      datos = new Uint8Array(m.control, CONTROL.DATOS, CONTROL.TAMANO_DATOS);
      interrupcion = new Uint8Array(m.interrupcion);
      try {
        await iniciar(m.indexURL);
      } catch (e) {
        enviar({ tipo: "error_inicio", mensaje: String((e as Error)?.message ?? e) });
      }
      break;
    case "ejecutar": {
      prepararEjecucion(m.id);
      let resultado: string;
      try {
        resultado = harness.ejecutar(m.codigo);
      } catch (e) {
        resultado = JSON.stringify({ estado: "detenido", error: null, duracion_ms: 0 });
      }
      vaciarSalida();
      enviar({ tipo: "fin", id: m.id, resultado, salida_excedida: excedida });
      break;
    }
    case "probar": {
      prepararEjecucion(m.id);
      let resultado: string;
      try {
        resultado = harness.probar(m.codigo, m.pruebas);
      } catch (e) {
        resultado = JSON.stringify({ pasadas: 0, total: 0, resultados: [], error: String(e) });
      }
      enviar({ tipo: "fin", id: m.id, resultado });
      break;
    }
    case "sintaxis": {
      prepararEjecucion(m.id);
      enviar({ tipo: "fin", id: m.id, resultado: harness.sintaxis(m.codigo) });
      break;
    }
  }
};
