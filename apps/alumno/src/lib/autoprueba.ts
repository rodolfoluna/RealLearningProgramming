// Autoprueba de extremo a extremo con el núcleo real (solo compilaciones de desarrollo con
// RLP_AUTOPRUEBA="alumno:<carpeta compartida>"). La usa scripts/autoprueba.sh.

import { ubicar } from "@rlp/curso";
import { invoke } from "@tauri-apps/api/core";
import { backend } from "./backend";
import { curso, python } from "./app.svelte";

type Paso = { paso: string; ok: boolean; detalle?: unknown };

export async function autoprueba(fase: string) {
  const compartida = fase.slice(fase.indexOf(":") + 1);
  const pasos: Paso[] = [];
  let ok = true;
  const paso = async (nombre: string, fn: () => Promise<unknown> | unknown) => {
    try {
      pasos.push({ paso: nombre, ok: true, detalle: await fn() });
    } catch (e) {
      ok = false;
      pasos.push({ paso: nombre, ok: false, detalle: String((e as Error)?.message ?? e) });
      throw e;
    }
  };
  try {
    const b = await backend();
    await paso("modo de entrada de Python", () => {
      // "memoria" con SharedArrayBuffer (WebView2/Chromium) o "puente" vía Rust (WebKitGTK).
      if (python.modo === "limitado") throw new Error("sin SharedArrayBuffer ni puente: input() no funcionaría");
      return python.modo;
    });
    await paso("Python con input()", async () => {
      await python.iniciar();
      let salida = "";
      const r = await python.ejecutar('n = input("Nombre: ")\nprint("Hola,", n)', {
        salida: (t) => (salida += t),
        entradaSolicitada: () => setTimeout(() => python.enviarEntrada("Ana"), 10),
      });
      if (r.estado !== "ok" || !salida.includes("Hola, Ana")) throw new Error(`salida: ${salida}`);
      return python.version;
    });
    await paso("importar grupo", () => b.importarGrupo(`${compartida}/grupo_Autoprueba.rlpg`));
    await paso("registrar alumno", async () => (await b.registrar("21349999", "Alumna de Prueba", "clave-de-prueba")).codigo.length);
    const { actividad } = ubicar(curso, "u1-suma-dos-numeros")!;
    const inicial = actividad.codigo_inicial ?? "";
    const codigo = 'a = int(input("Primer número: "))\nb = int(input("Segundo número: "))\nprint("La suma es", a + b)\n';
    await paso("abrir actividad", () => b.abrirActividad(actividad.id, inicial));
    await paso("teclear el código (operaciones)", async () => {
      let pos = inicial.length;
      const ops = [...codigo].map((c, i) => {
        const op = [i * 150, pos, pos, c, "t"] as [number, number, number, string, "t"];
        pos += c.length;
        return op;
      });
      const r = await b.guardarEdicion(actividad.id, { t0: Date.now(), ops }, inicial + codigo);
      if (!r.ok) throw new Error("el historial no coincide con el editor");
      return r.codigo.length;
    });
    await paso("pruebas automáticas", async () => {
      const r = await python.probar(inicial + codigo, actividad.pruebas ?? []);
      if (r.pasadas !== r.total) throw new Error(JSON.stringify(r.resultados.map((x) => x.mensaje)));
      const e = await b.registrarPruebas(actividad.id, r.pasadas, r.total, actividad.puntos);
      if (!e.completada) throw new Error("no quedó completada");
      return `${r.pasadas}/${r.total}`;
    });
    await paso("eventos de copiar/pegar", async () => {
      await b.registrarEvento("copia", actividad.id, { chars: 10, origen: "editor" });
      await b.registrarEvento("pegado", actividad.id, { chars: 40, via: "teclado", interno: false, permitido: false });
      const est = await b.estadisticas();
      if (est.global.pegados_intentos !== 1 || est.global.copias !== 1) throw new Error(JSON.stringify(est.global));
      return est.global;
    });
    await paso("detener un ciclo infinito", async () => {
      const p = python.ejecutar("while True:\n    pass\n");
      setTimeout(() => python.detener(), 300);
      const r = await p;
      if (r.estado !== "detenido") throw new Error(r.estado);
      return r.estado;
    });
    await paso("exportar entrega", () => b.exportar(compartida));
  } catch {
    /* el paso fallido ya quedó registrado */
  }
  await invoke("autoprueba_fin", { resultado: { ok, fase, pasos } });
}
