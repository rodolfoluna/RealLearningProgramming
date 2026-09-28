// Editor de código del alumno (CodeMirror 6) con control de portapapeles e historial de edición.

import { closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
import {
  cursorCharLeft,
  cursorCharRight,
  cursorLineDown,
  cursorLineUp,
  defaultKeymap,
  history,
  historyKeymap,
  indentLess,
  indentMore,
  undo,
} from "@codemirror/commands";
import { python } from "@codemirror/lang-python";
import {
  bracketMatching,
  HighlightStyle,
  indentOnInput,
  indentUnit,
  syntaxHighlighting,
} from "@codemirror/language";
import { lintGutter, setDiagnostics, type Diagnostic } from "@codemirror/lint";
import { highlightSelectionMatches, searchKeymap } from "@codemirror/search";
import { Compartment, EditorSelection, EditorState, type Extension } from "@codemirror/state";
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
  type Command,
} from "@codemirror/view";
import { tags as t } from "@lezer/highlight";
import { menuContextual } from "./menu";
import { AcumuladorOperaciones, type LoteOperaciones } from "./operaciones";
import {
  controlPortapapeles,
  type EventoCopia,
  type EventoInsercionSospechosa,
  type EventoPegado,
  type PoliticaPegado,
} from "./pegado";

export * from "./operaciones";
export * from "./pegado";

const estiloSintaxis = HighlightStyle.define([
  { tag: [t.keyword, t.controlKeyword, t.operatorKeyword, t.definitionKeyword], color: "var(--sx-palabra)", fontWeight: "600" },
  { tag: [t.string, t.special(t.string)], color: "var(--sx-cadena)" },
  { tag: [t.number, t.bool, t.null], color: "var(--sx-numero)" },
  { tag: t.comment, color: "var(--sx-comentario)", fontStyle: "italic" },
  { tag: [t.function(t.variableName), t.function(t.propertyName)], color: "var(--sx-funcion)" },
  { tag: t.definition(t.variableName), color: "var(--sx-definicion)" },
  { tag: [t.className, t.definition(t.className)], color: "var(--sx-clase)" },
  { tag: [t.operator, t.punctuation, t.bracket], color: "var(--sx-operador)" },
  { tag: t.self, color: "var(--sx-palabra)" },
]);

const tema = EditorView.theme({
  "&": { height: "100%", fontSize: "var(--editor-fuente, 15px)", backgroundColor: "var(--editor-fondo)", color: "var(--editor-texto)" },
  ".cm-scroller": { fontFamily: "var(--fuente-codigo)", lineHeight: "1.55" },
  ".cm-content": { caretColor: "var(--editor-cursor)", padding: "8px 0" },
  ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--editor-cursor)", borderLeftWidth: "2px" },
  ".cm-gutters": { backgroundColor: "var(--editor-margen)", color: "var(--editor-numeros)", border: "none" },
  ".cm-activeLine": { backgroundColor: "var(--editor-linea-activa)" },
  ".cm-activeLineGutter": { backgroundColor: "var(--editor-linea-activa)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": { backgroundColor: "var(--editor-seleccion) !important" },
  ".cm-matchingBracket": { backgroundColor: "var(--editor-pareja)", outline: "none" },
  "&.cm-focused": { outline: "none" },
});

/** Tab: sangría a múltiplo de 4 en el cursor; con selección, sangra las líneas. */
const tabulador: Command = (view) => {
  if (view.state.readOnly) return false;
  if (view.state.selection.ranges.some((r) => !r.empty)) return indentMore(view);
  view.dispatch(
    view.state.changeByRange((r) => {
      const linea = view.state.doc.lineAt(r.head);
      const col = r.head - linea.from;
      const espacios = " ".repeat(4 - (col % 4));
      return { changes: { from: r.head, insert: espacios }, range: EditorSelection.cursor(r.head + espacios.length) };
    }),
    { userEvent: "input.indent", scrollIntoView: true },
  );
  return true;
};

export interface OpcionesEditor {
  padre: HTMLElement;
  texto: string;
  soloLectura?: boolean;
  /** Política de pegado vigente (se consulta en cada intento). */
  politica?: () => PoliticaPegado;
  alCambiar?: (texto: string) => void;
  /** Lotes de operaciones para el historial verificable (solo en editores editables). */
  alOperaciones?: (lote: LoteOperaciones) => void;
  alPegar?: (e: EventoPegado) => void;
  alCopiar?: (e: EventoCopia) => void;
  alSospechar?: (e: EventoInsercionSospechosa) => void;
  atajos?: { tecla: string; accion: () => void }[];
  intervaloOperacionesMs?: number;
}

export type AccionTecla = "tab" | "izquierda" | "derecha" | "arriba" | "abajo" | "deshacer";

export class EditorCodigo {
  readonly view: EditorView;
  private readonly acumulador?: AcumuladorOperaciones;
  private readonly extensiones: Extension[];
  private readonly soloLectura = new Compartment();

  constructor(private readonly opciones: OpcionesEditor) {
    if (opciones.alOperaciones) {
      this.acumulador = new AcumuladorOperaciones(opciones.alOperaciones, opciones.intervaloOperacionesMs);
    }
    const atajosExtra = (opciones.atajos ?? []).map(({ tecla, accion }) => ({
      key: tecla,
      preventDefault: true,
      run: () => {
        accion();
        return true;
      },
    }));
    this.extensiones = [
      lineNumbers(),
      highlightActiveLineGutter(),
      highlightActiveLine(),
      history(),
      drawSelection(),
      indentOnInput(),
      bracketMatching(),
      closeBrackets(),
      highlightSelectionMatches(),
      python(),
      indentUnit.of("    "),
      EditorState.tabSize.of(4),
      syntaxHighlighting(estiloSintaxis),
      lintGutter(),
      keymap.of([
        ...atajosExtra,
        { key: "Tab", run: tabulador },
        { key: "Shift-Tab", run: indentLess },
        ...closeBracketsKeymap,
        ...defaultKeymap,
        ...historyKeymap,
        ...searchKeymap,
      ]),
      EditorView.contentAttributes.of({
        autocorrect: "off",
        autocapitalize: "off",
        spellcheck: "false",
        autocomplete: "off",
        "aria-label": "Editor de código",
      }),
      controlPortapapeles({
        politica: opciones.politica ?? (() => "bloquear"),
        alPegar: opciones.alPegar,
        alCopiar: opciones.alCopiar,
        alSospechar: opciones.alSospechar,
      }),
      menuContextual(),
      tema,
      this.soloLectura.of([EditorState.readOnly.of(!!opciones.soloLectura), EditorView.editable.of(!opciones.soloLectura)]),
      EditorView.updateListener.of((u) => {
        if (u.docChanged) opciones.alCambiar?.(u.state.doc.toString());
      }),
    ];
    if (this.acumulador) this.extensiones.push(this.acumulador.extension());
    this.view = new EditorView({
      parent: opciones.padre,
      state: EditorState.create({ doc: opciones.texto, extensions: this.extensiones }),
    });
  }

  get texto(): string {
    return this.view.state.doc.toString();
  }

  /** Reemplaza el documento sin registrar operaciones (cargar actividad, reiniciar). */
  establecerTexto(texto: string): void {
    this.acumulador?.descartar();
    this.view.setState(EditorState.create({ doc: texto, extensions: this.extensiones }));
  }

  /** Entrega de inmediato las operaciones pendientes. */
  vaciarOperaciones(): void {
    this.acumulador?.vaciar();
  }

  establecerSoloLectura(valor: boolean): void {
    this.view.dispatch({
      effects: this.soloLectura.reconfigure([EditorState.readOnly.of(valor), EditorView.editable.of(!valor)]),
    });
  }

  marcarError(linea: number | null, columna: number | null, mensaje: string): void {
    if (!linea || linea < 1 || linea > this.view.state.doc.lines) {
      this.limpiarErrores();
      return;
    }
    const l = this.view.state.doc.line(linea);
    const desde = columna && columna > 0 ? Math.min(l.from + columna - 1, l.to) : l.from;
    const diag: Diagnostic = { from: desde, to: Math.max(desde, l.to), severity: "error", message: mensaje };
    this.view.dispatch(setDiagnostics(this.view.state, [diag]));
  }

  limpiarErrores(): void {
    this.view.dispatch(setDiagnostics(this.view.state, []));
  }

  irALinea(linea: number): void {
    if (linea < 1 || linea > this.view.state.doc.lines) return;
    const l = this.view.state.doc.line(linea);
    this.view.dispatch({ selection: { anchor: l.to }, scrollIntoView: true });
    this.view.focus();
  }

  enfocar(): void {
    this.view.focus();
  }

  /**
   * Inserta texto como si se tecleara (barra de teclas en pantallas táctiles): pasa por los
   * mismos filtros y queda en el historial como tecleo.
   */
  teclear(texto: string): void {
    if (this.view.state.readOnly) return;
    this.view.dispatch(this.view.state.replaceSelection(texto), { userEvent: "input.type", scrollIntoView: true });
    this.view.focus();
  }

  /** Teclas especiales de la barra táctil. */
  accion(nombre: AccionTecla): void {
    const comandos = {
      tab: tabulador,
      izquierda: cursorCharLeft,
      derecha: cursorCharRight,
      arriba: cursorLineUp,
      abajo: cursorLineDown,
      deshacer: undo,
    } as const;
    comandos[nombre](this.view);
    this.view.focus();
  }

  destruir(): void {
    this.acumulador?.vaciar();
    this.view.destroy();
  }
}

/** Visor de solo lectura (profesor, ejemplos). Copiar se reporta si se da `alCopiar`. */
export function crearVisor(padre: HTMLElement, texto: string, alCopiar?: (e: EventoCopia) => void): EditorCodigo {
  return new EditorCodigo({ padre, texto, soloLectura: true, alCopiar });
}
