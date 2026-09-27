// Markdown de lecciones y enunciados, con resaltado de Python igual al del editor.

import { classHighlighter, highlightCode } from "@lezer/highlight";
import { parser } from "@lezer/python";
import MarkdownIt from "markdown-it";

function escapar(t: string): string {
  return t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

export function resaltarPython(codigo: string): string {
  let html = "";
  highlightCode(
    codigo,
    parser.parse(codigo),
    classHighlighter,
    (texto, clases) => {
      html += clases ? `<span class="${clases}">${escapar(texto)}</span>` : escapar(texto);
    },
    () => (html += "\n"),
  );
  return html;
}

export interface BloqueCodigo {
  codigo: string;
  ejecutable: boolean;
}

/** Convierte Markdown a HTML. Los bloques ```python se resaltan y se numeran en `bloques`. */
export function renderizarMarkdown(fuente: string, conBotonProbar: boolean): { html: string; bloques: BloqueCodigo[] } {
  const bloques: BloqueCodigo[] = [];
  const md = new MarkdownIt({ html: false, linkify: false, typographer: true });
  md.renderer.rules.fence = (tokens, idx) => {
    const t = tokens[idx];
    const lenguaje = t.info.trim().split(/\s+/)[0];
    const codigo = t.content.replace(/\n$/, "");
    const esPython = lenguaje === "python";
    const i = bloques.push({ codigo, ejecutable: esPython }) - 1;
    const cuerpo = esPython ? resaltarPython(codigo) : escapar(codigo);
    const boton =
      esPython && conBotonProbar
        ? `<button type="button" class="probar" data-bloque="${i}" title="Ejecutar este ejemplo">▶ Probar</button>`
        : "";
    return `<div class="bloque-codigo ${esPython ? "python" : "texto"}"><pre><code>${cuerpo}</code></pre>${boton}</div>`;
  };
  return { html: md.render(fuente), bloques };
}
