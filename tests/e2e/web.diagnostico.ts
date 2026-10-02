import { expect, type Locator, type Page } from "@playwright/test";

/** Errores de la consola del navegador, para explicar una falla. */
export function registrarErrores(page: Page): string[] {
  const errores: string[] = [];
  page.on("console", (m) => m.type() === "error" && errores.push(m.text()));
  page.on("pageerror", (e) => errores.push(e.message));
  return errores;
}

/** Espera el campo de `input()`; si no aparece, la falla dice por qué (aislamiento, consola, errores). */
export async function esperarEntrada(page: Page, dato: Locator, errores: string[]) {
  try {
    await expect(dato).toBeVisible({ timeout: 60_000 });
  } catch (e) {
    const estado = await page.evaluate(() => ({
      aislada: globalThis.crossOriginIsolated,
      sharedArrayBuffer: typeof SharedArrayBuffer,
      consola: document.querySelector("[data-consola]")?.textContent?.slice(0, 500),
    }));
    throw new Error(`${(e as Error).message}\nDiagnóstico: ${JSON.stringify({ ...estado, errores: errores.slice(-10) })}`);
  }
}
