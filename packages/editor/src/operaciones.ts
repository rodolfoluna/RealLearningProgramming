// Captura de operaciones de edición para el historial verificable.
//
// Cada operación es [dt, desde, hasta, insertado, origen]: reemplaza el rango [desde, hasta)
// del texto por `insertado`. Las posiciones son offsets UTF-16 (como las cadenas de JS y
// CodeMirror) y cada operación se aplica sobre el resultado de la anterior. El núcleo en Rust
// reproduce exactamente la misma secuencia para validar que el código final salió del editor.

import { Transaction, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

/** t tecleo · i automático (sangría/nueva línea) · d borrado · u deshacer · r rehacer · p pegado permitido · o otro */
export type Origen = "t" | "i" | "d" | "u" | "r" | "p" | "o";
export type Operacion = [dt: number, desde: number, hasta: number, insertado: string, origen: Origen];

export function origenDe(userEvent: string | undefined): Origen {
  if (!userEvent) return "o";
  if (userEvent.startsWith("input.type")) return "t";
  if (userEvent.startsWith("input.paste")) return "p";
  if (userEvent.startsWith("delete")) return "d";
  if (userEvent === "undo" || userEvent.startsWith("undo.")) return "u";
  if (userEvent === "redo" || userEvent.startsWith("redo.")) return "r";
  if (userEvent.startsWith("input") || userEvent.startsWith("indent")) return "i";
  return "o";
}

/** Convierte una transacción en operaciones secuenciales (de la última a la primera). */
export function operacionesDe(tr: Transaction, dt: number): Operacion[] {
  const origen = origenDe(tr.annotation(Transaction.userEvent));
  const ops: Operacion[] = [];
  tr.changes.iterChanges((desde, hasta, _b, _c, insertado) => {
    ops.push([dt, desde, hasta, insertado.toString(), origen]);
  });
  // Aplicadas en orden inverso, las posiciones anteriores no se desplazan.
  return ops.reverse();
}

/** Aplica operaciones a un texto (misma semántica que el núcleo en Rust). */
export function aplicarOperaciones(texto: string, ops: Operacion[]): string {
  let t = texto;
  for (const [, desde, hasta, insertado] of ops) {
    if (desde < 0 || hasta < desde || hasta > t.length) throw new Error("Operación fuera de rango");
    t = t.slice(0, desde) + insertado + t.slice(hasta);
  }
  return t;
}

export interface LoteOperaciones {
  /** Momento (epoch ms) de referencia; cada op guarda su desfase en ms. */
  t0: number;
  ops: Operacion[];
}

/**
 * Acumula operaciones y las entrega en lotes (cada `intervaloMs` o al llamar `vaciar`).
 */
export class AcumuladorOperaciones {
  private lote: LoteOperaciones | null = null;
  private temporizador: ReturnType<typeof setTimeout> | null = null;

  constructor(
    private readonly entregar: (lote: LoteOperaciones) => void,
    private readonly intervaloMs = 2000,
  ) {}

  extension(): Extension {
    return EditorView.updateListener.of((u) => {
      for (const tr of u.transactions) {
        if (!tr.docChanged) continue;
        const ahora = Date.now();
        if (!this.lote) this.lote = { t0: ahora, ops: [] };
        this.lote.ops.push(...operacionesDe(tr, ahora - this.lote.t0));
      }
      if (this.lote && !this.temporizador) {
        this.temporizador = setTimeout(() => this.vaciar(), this.intervaloMs);
      }
    });
  }

  get pendientes(): number {
    return this.lote?.ops.length ?? 0;
  }

  vaciar(): void {
    if (this.temporizador) clearTimeout(this.temporizador);
    this.temporizador = null;
    const lote = this.lote;
    this.lote = null;
    if (lote && lote.ops.length) this.entregar(lote);
  }

  descartar(): void {
    if (this.temporizador) clearTimeout(this.temporizador);
    this.temporizador = null;
    this.lote = null;
  }
}
