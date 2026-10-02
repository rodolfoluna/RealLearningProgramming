import { existsSync } from "node:fs";
import { expect, test } from "@playwright/test";

// Núcleo de la versión web en un navegador real: Worker con WebAssembly, perfiles en IndexedDB,
// recarga de la página y un perfil abierto en una sola pestaña.

test.skip(!existsSync("packages/nucleo-web/generado/rlp_web_bg.wasm"), "Compila el núcleo web con `pnpm wasm`");

const INICIAL = "# Escribe tu programa\n";
const clave = { tipo: "contrasena", contrasena: "clave-julia" };

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ConNucleo = Window & { nucleo: any };

test("núcleo web: registro, recarga, sesión en una sola pestaña y exportación", async ({ page, context }) => {
  await page.goto("/nucleo.html");
  const perfil = await page.evaluate(async (INICIAL) => {
    const n = (window as unknown as ConNucleo).nucleo;
    const r = await n.registrar("21340400", "Julia Web", "clave-julia");
    await n.sesion("abrir_actividad", { id: "u1-a1", codigo_inicial: INICIAL });
    const g = await n.sesion("guardar_edicion", {
      id: "u1-a1",
      lote: { t0: Date.now(), ops: [[0, INICIAL.length, INICIAL.length, "y", "t"]] },
      texto: INICIAL + "y",
    });
    if (!g.ok) throw new Error("no coincidió el texto");
    return r.estado.perfil.perfil_id as string;
  }, INICIAL);

  // Al recargar, el perfil sigue en IndexedDB y se vuelve a abrir con la contraseña.
  await page.reload();
  const despues = await page.evaluate(
    async ({ perfil, clave }) => {
      const n = (window as unknown as ConNucleo).nucleo;
      const inicio = await n.inicio();
      const estado = await n.iniciarSesion(perfil, clave);
      const entrega = await n.exportar();
      return {
        perfiles: inicio.perfiles.length,
        codigo: estado.actividades["u1-a1"].codigo,
        entrega: entrega.nombre,
        bytes: entrega.archivo.length,
      };
    },
    { perfil, clave },
  );
  expect(despues.perfiles).toBe(1);
  expect(despues.codigo).toBe(INICIAL + "y");
  expect(despues.entrega).toMatch(/^21340400_.*\.rlp$/);
  expect(despues.bytes).toBeGreaterThan(1000);

  // Una segunda pestaña no puede abrir el mismo perfil al mismo tiempo.
  const otra = await context.newPage();
  await otra.goto("/nucleo.html");
  const error = await otra.evaluate(
    ({ perfil, clave }) =>
      (window as unknown as ConNucleo).nucleo.iniciarSesion(perfil, clave).then(
        () => "se abrió",
        (e: Error) => e.message,
      ),
    { perfil, clave },
  );
  expect(error).toContain("otra pestaña");
});
