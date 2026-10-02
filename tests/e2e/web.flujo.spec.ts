import { existsSync, readFileSync } from "node:fs";
import { expect, test, type Page } from "@playwright/test";

// Versión web (PWA) de la App Alumno con el núcleo real: Rust en WebAssembly y perfiles en
// IndexedDB. La interfaz es la misma que la de la app nativa.

test.skip(!existsSync("packages/nucleo-web/generado/rlp_web_bg.wasm"), "Compila el núcleo web con `pnpm wasm`");

const capturas = "tests/e2e/capturas";

async function registrarse(page: Page, control: string, nombre: string) {
  await page.goto("/");
  await page.getByRole("tab", { name: "Soy nuevo" }).click();
  await page.getByLabel("Número de control").fill(control);
  await page.getByLabel("Nombre completo").fill(nombre);
  await page.getByLabel("Contraseña", { exact: true }).fill("contraseña-web");
  await page.getByLabel("Repite la contraseña").fill("contraseña-web");
  await page.getByRole("button", { name: "Crear mi perfil" }).click();
  await expect(page.getByText("Tu código de recuperación")).toBeVisible({ timeout: 30_000 });
  await page.getByLabel("Ya lo anoté en un lugar seguro").check();
  await page.getByRole("button", { name: "Empezar el curso" }).click();
  await expect(page.getByRole("heading", { name: new RegExp(`Hola, ${nombre.split(" ")[0]}`) })).toBeVisible();
}

test("registro, programa con input(), recarga y entrega descargada", async ({ page }) => {
  await registrarse(page, "21340500", "Karla Web");
  await page.locator('[data-actividad="u0-hola-mundo"]').click();
  const editor = page.locator("[data-editor] .cm-content");
  await editor.click();
  await page.keyboard.press("Control+A");
  await page.keyboard.press("Delete");
  await page.keyboard.type('nombre = input("Nombre: ")\nprint("Hola,", nombre)', { delay: 10 });
  await page.getByRole("button", { name: "▶ Ejecutar" }).click();
  const dato = page.getByLabel("Dato para el programa");
  await expect(dato).toBeVisible({ timeout: 60_000 });
  await dato.fill("Web");
  await dato.press("Enter");
  await expect(page.locator("[data-consola]")).toContainText("Hola, Web");
  await page.screenshot({ path: `${capturas}/web-01-input.png` });

  // Recargar: el perfil y su código siguen en el navegador.
  await page.waitForTimeout(1500); // el editor guarda en lotes
  await page.reload();
  await expect(page.getByText(/se guardan cifrados en este navegador/)).toBeVisible();
  await page.getByRole("button", { name: /Karla Web/ }).click();
  await page.getByLabel("Contraseña").fill("contraseña-web");
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.locator('[data-actividad="u0-hola-mundo"]').click();
  await expect(page.locator("[data-editor] .cm-content")).toContainText('print("Hola,", nombre)');

  // Exportar descarga el .rlp.
  const descarga = page.waitForEvent("download");
  await page.getByRole("button", { name: /Karla Web|^K/ }).first().click();
  await page.getByRole("menuitem", { name: /Exportar entrega/ }).click();
  const archivo = await descarga;
  expect(archivo.suggestedFilename()).toMatch(/^21340500_\d{8}-\d{4}\.rlp$/);
  const ruta = await archivo.path();
  expect(readFileSync(ruta!).subarray(0, 2).toString()).toBe("PK"); // zip
  await expect(page.getByText(/Se descargó tu entrega/)).toBeVisible();
  await page.screenshot({ path: `${capturas}/web-02-exportar.png` });
});
