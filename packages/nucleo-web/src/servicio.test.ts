// Núcleo web real (WebAssembly) con IndexedDB simulado: registro, "recarga" de la página,
// sesión en una sola pestaña y entrega exportada. Requiere `pnpm wasm`.
import "fake-indexeddb/auto";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { abrirAlmacen } from "./idb";
import { aBase64, deBase64, OTRA_PESTANA, ServicioNucleo, type Candados } from "./servicio";

const wasm = fileURLToPath(new URL("../generado/rlp_web_bg.wasm", import.meta.url));

/** Candados compartidos entre "pestañas" (instancias del servicio). */
function candadosCompartidos(): Candados {
  const tomados = new Set<string>();
  return {
    async tomar(nombre) {
      if (tomados.has(nombre)) return null;
      tomados.add(nombre);
      return () => tomados.delete(nombre);
    },
  };
}

const INICIAL = "# Escribe tu programa\n";

describe.skipIf(!existsSync(wasm))("núcleo web con IndexedDB", () => {
  it("convierte bytes a base64 y de vuelta", () => {
    const bytes = new Uint8Array(100_000).map((_, i) => (i * 7) % 256);
    expect(deBase64(aBase64(bytes))).toEqual(bytes);
  });

  it("registra, guarda, recarga y abre el perfil en una sola pestaña", async () => {
    const mod = await import("../generado/rlp_web.js");
    mod.initSync({ module: readFileSync(wasm) });
    const candados = candadosCompartidos();
    const pestana = async () => new ServicioNucleo(new mod.Nucleo(), await abrirAlmacen("prueba"), candados);

    const a = await pestana();
    expect((await a.inicio()).perfiles).toEqual([]);
    const r = await a.registrar("21340300", "Iris Web", "clave-iris");
    expect(r.codigo).toHaveLength(24);
    const perfil = r.estado.perfil.perfil_id;
    await a.sesion("abrir_actividad", { id: "u1-a1", codigo_inicial: INICIAL });
    const texto = `${INICIAL}x`;
    const g = await a.sesion<{ ok: boolean }>("guardar_edicion", {
      id: "u1-a1",
      lote: { t0: Date.now(), ops: [[0, INICIAL.length, INICIAL.length, "x", "t"]] },
      texto,
    });
    expect(g.ok).toBe(true);
    const { nombre, archivo } = await a.exportar();
    expect(nombre).toMatch(/^21340300_.*\.rlp$/);
    expect(archivo.length).toBeGreaterThan(1000);

    // Otra pestaña (o la misma página recargada) ve el perfil, pero no puede abrirlo a la vez.
    const b = await pestana();
    const inicio = await b.inicio();
    expect(inicio.perfiles).toHaveLength(1);
    await expect(b.iniciarSesion(perfil, { tipo: "contrasena", contrasena: "clave-iris" })).rejects.toThrow(
      OTRA_PESTANA,
    );
    await a.cerrarSesion();
    await expect(b.iniciarSesion(perfil, { tipo: "contrasena", contrasena: "otra-clave" })).rejects.toThrow(
      /incorrect/,
    );
    const estado = (await b.iniciarSesion(perfil, { tipo: "contrasena", contrasena: "clave-iris" })) as unknown as {
      actividades: Record<string, { codigo: string }>;
    };
    expect(estado.actividades["u1-a1"].codigo).toBe(texto);

    // La entrega exportada sirve para continuar en otro navegador.
    const otro = new ServicioNucleo(new mod.Nucleo(), await abrirAlmacen("otro-navegador"), candadosCompartidos());
    const restaurado = await otro.restaurar(archivo, { tipo: "contrasena", contrasena: "clave-iris" });
    expect(restaurado.perfil.perfil_id).toBe(perfil);
    expect((await otro.inicio()).perfiles).toHaveLength(1);
  }, 60_000);
});
