// Menú contextual propio del editor: sin la opción "Pegar" del sistema.

import { redo, selectAll, undo } from "@codemirror/commands";
import type { Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

interface Opcion {
  etiqueta: string;
  atajo?: string;
  deshabilitada?: boolean;
  nota?: string;
  accion?: (view: EditorView) => void;
}

let menuAbierto: HTMLElement | null = null;

function cerrar() {
  menuAbierto?.remove();
  menuAbierto = null;
}

function mostrar(x: number, y: number, view: EditorView) {
  cerrar();
  const soloLectura = view.state.readOnly;
  const opciones: Opcion[] = [
    { etiqueta: "Deshacer", atajo: "Ctrl+Z", deshabilitada: soloLectura, accion: (v) => undo(v) },
    { etiqueta: "Rehacer", atajo: "Ctrl+Y", deshabilitada: soloLectura, accion: (v) => redo(v) },
    { etiqueta: "Cortar", atajo: "Ctrl+X", deshabilitada: soloLectura, accion: () => document.execCommand("cut") },
    { etiqueta: "Copiar", atajo: "Ctrl+C", accion: () => document.execCommand("copy") },
    { etiqueta: "Pegar", deshabilitada: true, nota: "Pegar está deshabilitado: escribe tu código." },
    { etiqueta: "Seleccionar todo", atajo: "Ctrl+A", accion: (v) => selectAll(v) },
  ];
  const menu = document.createElement("div");
  menu.className = "rlp-menu-contextual";
  menu.setAttribute("role", "menu");
  for (const o of opciones) {
    const b = document.createElement("button");
    b.type = "button";
    b.setAttribute("role", "menuitem");
    b.disabled = !!o.deshabilitada;
    b.title = o.nota ?? "";
    b.innerHTML = `<span></span><kbd></kbd>`;
    (b.firstChild as HTMLElement).textContent = o.etiqueta;
    (b.lastChild as HTMLElement).textContent = o.atajo ?? "";
    b.addEventListener("mousedown", (e) => e.preventDefault()); // conserva la selección
    b.addEventListener("click", () => {
      cerrar();
      view.focus();
      o.accion?.(view);
    });
    menu.appendChild(b);
  }
  document.body.appendChild(menu);
  const r = menu.getBoundingClientRect();
  menu.style.left = `${Math.min(x, window.innerWidth - r.width - 4)}px`;
  menu.style.top = `${Math.min(y, window.innerHeight - r.height - 4)}px`;
  menuAbierto = menu;
  setTimeout(() => {
    const fuera = (e: Event) => {
      if (!menu.contains(e.target as Node)) {
        cerrar();
        document.removeEventListener("mousedown", fuera, true);
        document.removeEventListener("keydown", fuera, true);
      }
    };
    document.addEventListener("mousedown", fuera, true);
    document.addEventListener("keydown", fuera, true);
  });
}

export function menuContextual(): Extension {
  return EditorView.domEventHandlers({
    contextmenu(event, view) {
      event.preventDefault();
      mostrar(event.clientX, event.clientY, view);
      return true;
    },
  });
}
