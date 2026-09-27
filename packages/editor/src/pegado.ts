// Control de copiar/pegar en el editor del alumno.
//
// Todas las vías de pegado terminan en una de estas barreras:
//  1. Evento DOM `paste` (Ctrl+V, Shift+Insert, menú del sistema, historial de portapapeles Win+V).
//  2. Evento DOM `drop` (arrastrar texto desde otra app).
//  3. `beforeinput` con inputType de pegado (navegadores/teclados que no disparan `paste`).
//  4. Filtro de transacciones: rechaza `input.paste`/`input.drop` y las inserciones "de teclado"
//     demasiado grandes para ser tecleo humano (teclados virtuales con portapapeles, autoescritores).

import { Annotation, EditorState, Transaction, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

export type PoliticaPegado = "bloquear" | "propio";
export type ViaPegado = "teclado" | "soltar" | "entrada" | "ime";

export interface EventoPegado {
  chars: number;
  /** El texto coincide con la última copia hecha en este mismo editor. */
  interno: boolean;
  permitido: boolean;
  via: ViaPegado;
}

export interface EventoCopia {
  chars: number;
  cortar: boolean;
}

export interface EventoInsercionSospechosa {
  chars: number;
  userEvent: string;
}

export interface OpcionesControl {
  politica: () => PoliticaPegado;
  alPegar?: (e: EventoPegado) => void;
  alCopiar?: (e: EventoCopia) => void;
  alSospechar?: (e: EventoInsercionSospechosa) => void;
  /** Máximo de caracteres visibles que una sola transacción de tecleo puede insertar. */
  umbralInsercion?: number;
}

/** Marca transacciones de pegado que la política sí permite. */
export const pegadoPermitido = Annotation.define<boolean>();

/** Hash rápido (cyrb53) para comparar textos sin guardarlos. */
export function hashTexto(texto: string): number {
  let h1 = 0xdeadbeef;
  let h2 = 0x41c6ce57;
  for (let i = 0; i < texto.length; i++) {
    const c = texto.charCodeAt(i);
    h1 = Math.imul(h1 ^ c, 2654435761);
    h2 = Math.imul(h2 ^ c, 1597334677);
  }
  h1 = Math.imul(h1 ^ (h1 >>> 16), 2246822507) ^ Math.imul(h2 ^ (h2 >>> 13), 3266489909);
  h2 = Math.imul(h2 ^ (h2 >>> 16), 2246822507) ^ Math.imul(h1 ^ (h1 >>> 13), 3266489909);
  return 4294967296 * (2097151 & h2) + (h1 >>> 0);
}

export function normalizarSaltos(texto: string): string {
  return texto.replace(/\r\n?/g, "\n");
}

/**
 * ¿Una inserción "de teclado" es demasiado grande para ser tecleo humano?
 * Los espacios cuentan poco (autoindentación); los teclados de celular que escriben palabras
 * completas (deslizar) se toleran si son solo letras.
 */
export function esInsercionSospechosa(insertado: string, userEvent: string, umbral = 3): boolean {
  const visibles = insertado.replace(/\s/g, "");
  if (visibles.length <= umbral) return false;
  if (userEvent.startsWith("input.type.compose") && /^\p{L}{1,20}$/u.test(visibles) && !insertado.includes("\n")) {
    return false;
  }
  return true;
}

/** Texto que se copiaría con la selección actual (igual que CodeMirror: línea completa si no hay selección). */
export function textoSeleccionado(state: EditorState): { texto: string; lineas: boolean } {
  const rangos = state.selection.ranges;
  if (rangos.some((r) => !r.empty)) {
    return {
      texto: rangos
        .filter((r) => !r.empty)
        .map((r) => state.sliceDoc(r.from, r.to))
        .join(state.lineBreak),
      lineas: false,
    };
  }
  const vistos = new Set<number>();
  const partes: string[] = [];
  for (const r of rangos) {
    const linea = state.doc.lineAt(r.head);
    if (vistos.has(linea.number)) continue;
    vistos.add(linea.number);
    partes.push(linea.text);
  }
  return { texto: partes.join(state.lineBreak) + state.lineBreak, lineas: true };
}

export function controlPortapapeles(opciones: OpcionesControl): Extension {
  const umbral = opciones.umbralInsercion ?? 3;
  let ultimaCopia: number | null = null;
  let ultimoReporte = { t: 0, chars: -1 };

  const reportarPegado = (texto: string, via: ViaPegado, permitido: boolean, interno: boolean) => {
    // Un mismo intento puede llegar por dos barreras (p. ej. `paste` y `beforeinput`) casi a la vez.
    const ahora = Date.now();
    if (ahora - ultimoReporte.t < 30 && ultimoReporte.chars === texto.length && via !== "soltar") return;
    ultimoReporte = { t: ahora, chars: texto.length };
    opciones.alPegar?.({ chars: texto.length, interno, permitido, via });
  };

  const copiar = (event: ClipboardEvent, view: EditorView, cortar: boolean) => {
    const { texto, lineas } = textoSeleccionado(view.state);
    if (!event.clipboardData) return false;
    event.preventDefault();
    event.clipboardData.clearData();
    event.clipboardData.setData("text/plain", texto);
    ultimaCopia = hashTexto(normalizarSaltos(texto));
    opciones.alCopiar?.({ chars: texto.length, cortar });
    if (cortar && !view.state.readOnly) {
      if (lineas) {
        const cambios = view.state.selection.ranges.map((r) => {
          const l = view.state.doc.lineAt(r.head);
          return { from: l.from, to: Math.min(view.state.doc.length, l.to + 1) };
        });
        view.dispatch({ changes: cambios, userEvent: "delete.cut", scrollIntoView: true });
      } else {
        view.dispatch(view.state.replaceSelection(""), { userEvent: "delete.cut", scrollIntoView: true });
      }
    }
    return true;
  };

  return [
    EditorView.domEventHandlers({
      paste(event, view) {
        event.preventDefault();
        const texto = normalizarSaltos(event.clipboardData?.getData("text/plain") ?? "");
        const interno = texto.length > 0 && ultimaCopia !== null && hashTexto(texto) === ultimaCopia;
        const permitido = interno && opciones.politica() === "propio" && !view.state.readOnly;
        reportarPegado(texto, "teclado", permitido, interno);
        if (permitido) {
          view.dispatch(view.state.replaceSelection(texto), {
            annotations: pegadoPermitido.of(true),
            userEvent: "input.paste",
            scrollIntoView: true,
          });
        }
        return true;
      },
      drop(event, view) {
        event.preventDefault();
        const texto = normalizarSaltos(event.dataTransfer?.getData("text/plain") ?? "");
        const seleccion = textoSeleccionado(view.state).texto;
        // Arrastrar la propia selección dentro del editor no es un pegado: solo se ignora.
        if (texto && texto !== normalizarSaltos(seleccion)) {
          reportarPegado(texto, "soltar", false, false);
        }
        return true;
      },
      dragover(event) {
        if (event.dataTransfer) event.dataTransfer.dropEffect = "none";
        event.preventDefault();
        return true;
      },
      beforeinput(event) {
        const tipo = event.inputType;
        if (tipo.startsWith("insertFromPaste") || tipo === "insertFromDrop" || tipo === "insertFromYank") {
          event.preventDefault();
          const texto = normalizarSaltos(event.dataTransfer?.getData("text/plain") ?? event.data ?? "");
          reportarPegado(texto, "entrada", false, false);
          return true;
        }
        return false;
      },
      copy(event, view) {
        return copiar(event, view, false);
      },
      cut(event, view) {
        return copiar(event, view, true);
      },
    }),
    EditorState.transactionFilter.of((tr) => {
      if (!tr.docChanged || tr.annotation(pegadoPermitido)) return tr;
      if (tr.isUserEvent("input.paste") || tr.isUserEvent("input.drop")) {
        let texto = "";
        tr.changes.iterChanges((_a, _b, _c, _d, ins) => (texto += ins.toString()));
        reportarPegado(texto, tr.isUserEvent("input.drop") ? "soltar" : "teclado", false, false);
        return [];
      }
      if (tr.isUserEvent("input.type") || tr.isUserEvent("input.complete")) {
        let texto = "";
        tr.changes.iterChanges((_a, _b, _c, _d, ins) => (texto += ins.toString()));
        const userEvent = tr.annotation(Transaction.userEvent) ?? "";
        if (esInsercionSospechosa(texto, userEvent, umbral)) {
          opciones.alSospechar?.({ chars: texto.length, userEvent });
          reportarPegado(texto, "ime", false, false);
          return [];
        }
      }
      return tr;
    }),
  ];
}
