import { existsSync, readFileSync, statSync } from "node:fs";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { extname, join, resolve } from "node:path";
import { expect, test } from "@playwright/test";

// La PWA publicada tal como quedaría en GitHub Pages: archivos estáticos en una subcarpeta y sin
// encabezados COOP/COEP del servidor. El service worker debe aislar la página (input()), guardar
// todo para usarla sin conexión y el núcleo debe seguir funcionando sin red.

const dist = resolve("apps/alumno-web/dist");
test.skip(!existsSync(join(dist, "sw.js")), "Compila la versión web: pnpm wasm && pnpm --filter @rlp/alumno-web build");

const tipos: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript",
  ".mjs": "text/javascript",
  ".css": "text/css",
  ".wasm": "application/wasm",
  ".json": "application/json",
  ".webmanifest": "application/manifest+json",
  ".png": "image/png",
  ".zip": "application/zip",
};

let servidor: Server;
let base = "";

test.beforeAll(async () => {
  servidor = createServer((pedido, respuesta) => {
    const ruta = decodeURIComponent(new URL(pedido.url ?? "/", "http://x").pathname);
    let archivo = resolve(dist, "." + ruta.replace(/^\/rlp/, ""));
    if (!ruta.startsWith("/rlp/") || !archivo.startsWith(dist)) return void respuesta.writeHead(404).end();
    if (existsSync(archivo) && statSync(archivo).isDirectory()) archivo = join(archivo, "index.html");
    if (!existsSync(archivo)) return void respuesta.writeHead(404).end();
    respuesta.writeHead(200, { "Content-Type": tipos[extname(archivo)] ?? "application/octet-stream" });
    respuesta.end(readFileSync(archivo));
  });
  await new Promise<void>((listo) => servidor.listen(0, "127.0.0.1", listo));
  base = `http://127.0.0.1:${(servidor.address() as AddressInfo).port}/rlp/`;
});

test.afterAll(() => servidor?.close());

test("PWA: aislada sin encabezados del servidor, instalable y sin conexión", async ({ page, context }) => {
  test.setTimeout(180_000);
  await page.goto(base);
  // Primera visita: el service worker recarga la página una vez, ya aislada.
  await page.waitForFunction(() => crossOriginIsolated === true, null, { timeout: 60_000 });
  await expect(page.getByRole("tab", { name: "Soy nuevo" })).toBeVisible();
  const manifiesto = await page.evaluate(async () => {
    const enlace = document.querySelector<HTMLLinkElement>('link[rel="manifest"]')!;
    return (await fetch(enlace.href)).json();
  });
  expect(manifiesto.display).toBe("standalone");
  expect(manifiesto.icons.length).toBeGreaterThanOrEqual(2);
  await page.waitForFunction(() => localStorage.getItem("rlp-sin-conexion"), null, { timeout: 120_000 });

  // Sin red: la app, el núcleo (wasm) y Python vienen de la caché del service worker.
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByRole("tab", { name: "Soy nuevo" })).toBeVisible();
  expect(await page.evaluate(() => crossOriginIsolated)).toBe(true);
  await page.getByRole("tab", { name: "Soy nuevo" }).click();
  await page.getByLabel("Número de control").fill("21340600");
  await page.getByLabel("Nombre completo").fill("Laura Sin Red");
  await page.getByLabel("Contraseña", { exact: true }).fill("contraseña-web");
  await page.getByLabel("Repite la contraseña").fill("contraseña-web");
  await page.getByRole("button", { name: "Crear mi perfil" }).click();
  await page.getByLabel("Ya lo anoté en un lugar seguro").check();
  await page.getByRole("button", { name: "Empezar el curso" }).click();
  await page.locator('[data-actividad="u0-hola-mundo"]').click();
  await page.locator("[data-editor] .cm-content").click();
  await page.keyboard.press("Control+A");
  await page.keyboard.press("Delete");
  await page.keyboard.type('n = input("Nombre: ")\nprint("Hola,", n)', { delay: 10 });
  await page.getByRole("button", { name: "▶ Ejecutar" }).click();
  const dato = page.getByLabel("Dato para el programa");
  await expect(dato).toBeVisible({ timeout: 60_000 });
  await dato.fill("sin red");
  await dato.press("Enter");
  await expect(page.locator("[data-consola]")).toContainText("Hola, sin red");
  await page.screenshot({ path: "tests/e2e/capturas/web-03-sin-conexion.png" });
  await context.setOffline(false);
});
