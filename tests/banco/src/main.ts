// Página mínima que expone el editor y el ejecutor de Python a las pruebas de Playwright.
import { EditorCodigo, type LoteOperaciones, type PoliticaPegado } from "@rlp/editor";
import { EjecutorPython } from "@rlp/python-worker";

const registro = {
  pegados: [] as unknown[],
  copias: [] as unknown[],
  sospechas: [] as unknown[],
  lotes: [] as LoteOperaciones[],
};
let politica: PoliticaPegado = "bloquear";

const editor = new EditorCodigo({
  padre: document.getElementById("editor")!,
  texto: "print('hola')\n",
  politica: () => politica,
  alPegar: (e) => registro.pegados.push(e),
  alCopiar: (e) => registro.copias.push(e),
  alSospechar: (e) => registro.sospechas.push(e),
  alOperaciones: (l) => registro.lotes.push(l),
  intervaloOperacionesMs: 200,
});

const ejecutor = new EjecutorPython({ indexURL: "/pyodide/", timeoutPruebaMs: 1500, graciaMs: 800 });

Object.assign(window, {
  rlp: {
    editor,
    ejecutor,
    registro,
    aislado: () => globalThis.crossOriginIsolated,
    fijarPolitica: (p: PoliticaPegado) => (politica = p),
  },
});
