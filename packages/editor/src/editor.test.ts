import { EditorState } from "@codemirror/state";
import { describe, expect, it } from "vitest";
import { aplicarOperaciones, operacionesDe, origenDe, type Operacion } from "./operaciones";
import { controlPortapapeles, esInsercionSospechosa, hashTexto, normalizarSaltos, textoSeleccionado } from "./pegado";

describe("inserciones sospechosas", () => {
  it("el tecleo normal no es sospechoso", () => {
    expect(esInsercionSospechosa("a", "input.type")).toBe(false);
    expect(esInsercionSospechosa("()", "input.type")).toBe(false);
    expect(esInsercionSospechosa("\n    ", "input.type")).toBe(false);
  });
  it("bloques grandes o de varias líneas sí lo son", () => {
    expect(esInsercionSospechosa("print('hola')", "input.type")).toBe(true);
    expect(esInsercionSospechosa("x = 1\ny = 2", "input.type.compose")).toBe(true);
  });
  it("palabras completas de teclados de celular se toleran", () => {
    expect(esInsercionSospechosa("hola", "input.type.compose")).toBe(false);
    expect(esInsercionSospechosa("hola", "input.type")).toBe(true);
  });
});

describe("filtro de transacciones", () => {
  const eventos: string[] = [];
  const estado = () =>
    EditorState.create({
      doc: "abc",
      extensions: controlPortapapeles({ politica: () => "bloquear", alPegar: (e) => eventos.push(e.via), alSospechar: () => eventos.push("sospecha") }),
    });

  it("rechaza pegar aunque venga como transacción", () => {
    const tr = estado().update({ changes: { from: 3, insert: "print(1)" }, userEvent: "input.paste" });
    expect(tr.docChanged).toBe(false);
    expect(eventos).toContain("teclado");
  });
  it("rechaza tecleo masivo y permite tecleo normal", () => {
    expect(estado().update({ changes: { from: 3, insert: "while True: pass" }, userEvent: "input.type" }).docChanged).toBe(false);
    expect(eventos).toContain("sospecha");
    const tr = estado().update({ changes: { from: 3, insert: "d" }, userEvent: "input.type" });
    expect(tr.state.doc.toString()).toBe("abcd");
  });
  it("deshacer y cambios de la app no se filtran", () => {
    expect(estado().update({ changes: { from: 0, insert: "x = 1\ny = 2\n" }, userEvent: "undo" }).docChanged).toBe(true);
  });
});

describe("operaciones reproducibles", () => {
  it("una transacción con varios cambios se reproduce igual", () => {
    const s = EditorState.create({ doc: "uno dos tres" });
    const tr = s.update({
      changes: [
        { from: 0, to: 3, insert: "1" },
        { from: 8, to: 12, insert: "3!" },
      ],
      userEvent: "input.type",
    });
    const ops = operacionesDe(tr, 0);
    expect(aplicarOperaciones("uno dos tres", ops)).toBe(tr.state.doc.toString());
    expect(ops.every((o) => o[4] === "t")).toBe(true);
  });
  it("posiciones en UTF-16 (acentos y emoji)", () => {
    const ops: Operacion[] = [[0, 6, 6, "!", "t"], [0, 0, 1, "A", "t"]];
    expect(aplicarOperaciones("año 😀", ops)).toBe("Año 😀!");
    expect(() => aplicarOperaciones("abc", [[0, 2, 9, "x", "t"]])).toThrow();
  });
  it("origen de cada evento", () => {
    expect(origenDe("input.type.compose")).toBe("t");
    expect(origenDe("delete.backward")).toBe("d");
    expect(origenDe("undo")).toBe("u");
    expect(origenDe("redo")).toBe("r");
    expect(origenDe("input")).toBe("i");
    expect(origenDe("input.paste")).toBe("p");
    expect(origenDe(undefined)).toBe("o");
  });
});

describe("copiar", () => {
  it("sin selección se copia la línea completa", () => {
    const s = EditorState.create({ doc: "a = 1\nb = 2", selection: { anchor: 7 } });
    expect(textoSeleccionado(s)).toEqual({ texto: "b = 2\n", lineas: true });
  });
  it("el hash ignora el tipo de salto de línea", () => {
    expect(hashTexto(normalizarSaltos("a\r\nb"))).toBe(hashTexto("a\nb"));
  });
});
