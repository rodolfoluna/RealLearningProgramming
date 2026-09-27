// Autoprueba de extremo a extremo con el núcleo real (solo compilaciones de desarrollo).
//  - RLP_AUTOPRUEBA="profesor-grupo:<carpeta>"    crea llaves y grupo y exporta el .rlpg
//  - RLP_AUTOPRUEBA="profesor-importar:<carpeta>" importa y verifica la entrega del alumno

import { ubicar } from "@rlp/curso";
import { invoke } from "@tauri-apps/api/core";
import { backend } from "./backend";
import { codigosIniciales, curso, python } from "./app.svelte";

type Paso = { paso: string; ok: boolean; detalle?: unknown };
const CLAVE = "clave-del-profesor";

export async function autoprueba(fase: string) {
  const [nombre, compartida] = [fase.slice(0, fase.indexOf(":")), fase.slice(fase.indexOf(":") + 1)];
  const pasos: Paso[] = [];
  let ok = true;
  const paso = async (titulo: string, fn: () => Promise<unknown>) => {
    try {
      pasos.push({ paso: titulo, ok: true, detalle: await fn() });
    } catch (e) {
      ok = false;
      pasos.push({ paso: titulo, ok: false, detalle: String((e as Error)?.message ?? e) });
      throw e;
    }
  };
  try {
    const b = await backend();
    if (nombre === "profesor-grupo") {
      await paso("crear llaves", () => b.crearIdentidad("Profesor Autoprueba", CLAVE));
      await paso("respaldar llaves", () => b.respaldarIdentidad(compartida));
      const g = await b.crearGrupo({
        nombre: "Autoprueba",
        materia: "Fundamentos",
        periodo: "2026",
        regex_control: "\\d{8}",
        politicas: { pegado: "bloquear", registrar_salidas: true },
        coprofesores: [],
      });
      await paso("exportar grupo", () => b.exportarGrupo(g.grupo_id, compartida));
    } else {
      await paso("desbloquear", () => b.desbloquear(CLAVE));
      const r = await b.importarCarpeta(compartida, codigosIniciales);
      await paso("importar entregas", async () => {
        if (r.length !== 1 || r[0].error) throw new Error(JSON.stringify(r));
        return r[0].registro;
      });
      const d = await b.detalle(r[0].registro!.entrega_id);
      await paso("verificación", async () => {
        const malos = d.reporte.checks.filter((c) => c.nivel !== "verde" && !(c.id === "firma" && c.nivel === "amarillo"));
        if (malos.length) throw new Error(JSON.stringify(malos));
        return d.reporte.checks.map((c) => `${c.id}:${c.nivel}`);
      });
      await paso("estadísticas del alumno", async () => {
        const g = d.estadisticas.global;
        if (g.pegados_intentos !== 1 || g.copias !== 1 || g.teclas < 50) throw new Error(JSON.stringify(g));
        return g;
      });
      await paso("re-ejecutar pruebas del alumno", async () => {
        const { actividad } = ubicar(curso, "u1-suma-dos-numeros")!;
        const res = await python.probar(d.actividades[actividad.id].codigo, actividad.pruebas ?? []);
        if (res.pasadas !== res.total) throw new Error(JSON.stringify(res));
        return `${res.pasadas}/${res.total}`;
      });
      await paso("reproducción de la escritura", async () => {
        const l = await b.reproduccion(d.fila.entrega_id, "u1-suma-dos-numeros");
        let texto = "";
        for (const t of l.tramos) {
          texto = t.texto_inicial ?? "";
          for (const [, desde, hasta, insertado] of t.ops) texto = texto.slice(0, desde) + insertado + texto.slice(hasta);
        }
        if (l.avisos.length || !l.tramos.length || texto !== l.codigo_final) throw new Error(JSON.stringify(l.avisos));
        if (!l.marcas.some((m) => m.tipo === "pegado")) throw new Error("falta la marca del intento de pegar");
        return `${l.tramos.length} tramo(s), ${l.marcas.length} marca(s)`;
      });
      await paso("tablero", async () => {
        const t = await b.tablero(null);
        if (t.length !== 1 || t[0].numero_control !== "21349999") throw new Error(JSON.stringify(t));
        return t[0].nombre;
      });
    }
  } catch {
    /* ya registrado */
  }
  await invoke("autoprueba_fin", { resultado: { ok, fase, pasos } });
}
