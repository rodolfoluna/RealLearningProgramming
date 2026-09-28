import { expect, test } from "@playwright/test";

// Interfaz de la App Profesor con datos de demostración (la verificación real está en Rust).

const capturas = "tests/e2e/capturas";

test("desbloquear, tablero, detalle con integridad y grupos", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Contraseña de tus llaves").fill("incorrecta");
  await page.getByRole("button", { name: "Desbloquear" }).click();
  await expect(page.getByRole("main").getByText(/incorrect/)).toBeVisible();
  await page.getByLabel("Contraseña de tus llaves").fill("profesor1234");
  await page.getByRole("button", { name: "Desbloquear" }).click();

  await expect(page.getByText("alumnos con entregas")).toBeVisible();
  await expect(page.locator("tr[data-alumno]")).toHaveCount(4);
  await page.screenshot({ path: `${capturas}/profesor-01-tablero.png` });
  await page.getByLabel("Ver mapa de actividades").check();
  await page.screenshot({ path: `${capturas}/profesor-02-mapa.png` });

  // Entrega alterada: aparece en rojo con la explicación.
  await page.locator('tr[data-alumno="21340004"]').click();
  await expect(page.getByText("el código entregado NO coincide")).toBeVisible();
  await page.screenshot({ path: `${capturas}/profesor-03-integridad-rojo.png` });

  await page.getByRole("button", { name: "← Tablero" }).click();
  await page.locator('tr[data-alumno="21340001"]').click();
  await expect(page.getByRole("tab", { name: /Actividades/ })).toBeVisible();
  await page.getByRole("button", { name: /Suma de dos números/ }).click();
  await expect(page.locator(".visor .cm-content")).toContainText("La suma es");
  await page.getByRole("button", { name: /Volver a correr las pruebas/ }).click();
  await expect(page.getByText("Pasaron 3 de 3")).toBeVisible({ timeout: 60_000 });
  await page.getByLabel("Calificación").fill("10");
  await page.getByRole("button", { name: "Guardar" }).click();
  await expect(page.getByText("Calificación guardada.")).toBeVisible();
  await page.screenshot({ path: `${capturas}/profesor-04-actividad.png` });

  await page.getByRole("button", { name: "👥 Grupos" }).click();
  await page.getByRole("button", { name: "＋ Nuevo grupo" }).click();
  await page.getByLabel("Nombre del grupo").fill("Programación 1B");
  await page.screenshot({ path: `${capturas}/profesor-05-nuevo-grupo.png` });
  await page.getByRole("button", { name: "Crear grupo" }).click();
  await expect(page.getByRole("heading", { name: "Programación 1B" })).toBeVisible();
});

test("reproducir cómo se escribió el código", async ({ page }) => {
  await page.goto("/?desbloqueado");
  // Bruno tiene intentos de pegar: aparecen como marcas en la línea de tiempo.
  await page.locator('tr[data-alumno="21340002"]').click();
  await page.getByRole("tab", { name: /Actividades/ }).click();
  await page.getByRole("button", { name: /Suma de dos números/ }).click();
  await page.locator("[data-ver-reproduccion]").click();
  const dialogo = page.getByRole("dialog");
  await expect(dialogo.getByRole("heading", { name: /Cómo escribió: Suma de dos números/ })).toBeVisible();
  await expect(dialogo.locator(".marca.pegado")).toHaveCount(1);
  await dialogo.getByLabel("Velocidad").selectOption("100");
  await dialogo.locator("[data-reproducir]").click();
  await expect(dialogo.locator("[data-coincide]")).toBeVisible({ timeout: 30_000 });
  await expect(dialogo.locator("[data-reproductor-visor] .cm-content")).toContainText("La suma es");
  await page.screenshot({ path: `${capturas}/profesor-06-reproductor.png` });

  // Saltar a la mitad reconstruye el texto parcial.
  await dialogo.getByLabel("Posición en el historial").fill("0");
  await expect(dialogo.locator("[data-coincide]")).toHaveCount(0);
});

test("retroalimentación para el grupo y archivo de acceso", async ({ page }) => {
  await page.goto("/?desbloqueado");
  await page.getByRole("button", { name: /Retroalimentación/ }).click();
  await expect(page.getByText(/Aún no has calificado/)).toBeVisible();

  await page.locator('tr[data-alumno="21340001"]').click();
  await page.getByRole("tab", { name: /Actividades/ }).click();
  await page.getByRole("button", { name: /Suma de dos números/ }).click();
  await page.getByLabel("Calificación").fill("9");
  await page.getByRole("button", { name: "Guardar" }).click();
  await expect(page.getByText("Calificación guardada.")).toBeVisible();

  await page.getByRole("button", { name: /Archivo de acceso/ }).click();
  await page.getByRole("button", { name: "Crear archivo" }).click();
  await expect(page.locator("[data-temporal]")).toHaveText("ABCD-EFGH-JKMN");
  await page.screenshot({ path: `${capturas}/profesor-07-acceso.png` });
  await page.getByRole("button", { name: "Listo" }).click();

  await page.getByRole("button", { name: "← Tablero" }).click();
  await page.getByRole("button", { name: /Retroalimentación/ }).click();
  await expect(page.getByText(/Retroalimentación para 1 alumno/)).toBeVisible();
});
