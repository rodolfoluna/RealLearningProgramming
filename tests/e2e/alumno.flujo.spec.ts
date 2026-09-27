import { expect, test, type Page } from "@playwright/test";

// Interfaz de la App Alumno con el núcleo simulado (en la app real, los mismos comandos van a Rust).

const capturas = "tests/e2e/capturas";

async function registrarse(page: Page) {
  await page.goto("/");
  await page.getByRole("tab", { name: "Soy nuevo" }).click();
  await page.getByLabel("Número de control").fill("21340001");
  await page.getByLabel("Nombre completo").fill("Ana López García");
  await page.getByLabel("Contraseña", { exact: true }).fill("contraseña-segura");
  await page.getByLabel("Repite la contraseña").fill("contraseña-segura");
  await page.screenshot({ path: `${capturas}/alumno-01-registro.png` });
  await page.getByRole("button", { name: "Crear mi perfil" }).click();
  await expect(page.getByText("Tu código de recuperación")).toBeVisible();
  await page.screenshot({ path: `${capturas}/alumno-02-codigo-recuperacion.png` });
  await page.getByLabel("Ya lo anoté en un lugar seguro").check();
  await page.getByRole("button", { name: "Empezar el curso" }).click();
  await expect(page.getByRole("heading", { name: /Hola, Ana/ })).toBeVisible();
}

test("registro, actividad de código, pegado bloqueado y estadísticas", async ({ page }) => {
  await registrarse(page);
  await page.screenshot({ path: `${capturas}/alumno-03-bienvenida.png` });

  // Lección con ejemplo ejecutable.
  await page.getByRole("button", { name: /Tu primer programa/ }).first().click();
  await page.getByRole("button", { name: "▶ Probar" }).first().click();
  await expect(page.locator("[data-consola]")).toContainText("Hola, mundo", { timeout: 60_000 });
  await page.screenshot({ path: `${capturas}/alumno-04-leccion.png` });

  // Actividad "Hola, mundo".
  await page.locator('[data-actividad="u0-hola-mundo"]').click();
  const editor = page.locator("[data-editor] .cm-content");
  await editor.click();
  await page.keyboard.press("Control+End");

  // Intentar pegar: no se inserta y el contador sube.
  await page.evaluate(() => navigator.clipboard.writeText('print("Hola, mundo")'));
  await page.keyboard.press("Control+V");
  await expect(page.locator('[data-contador="pegados"] strong')).toHaveText("1");
  await expect(page.getByText(/Pegar está deshabilitado/)).toBeVisible();

  await page.keyboard.type('print("Hola, mundo")', { delay: 20 });
  await page.getByRole("button", { name: "▶ Ejecutar" }).click();
  await expect(page.locator("[data-consola]")).toContainText("Hola, mundo");
  await page.getByRole("button", { name: "✔ Probar" }).click();
  await expect(page.getByText("¡Todas las pruebas pasaron!")).toBeVisible();
  await expect(page.getByText("✓ Completada")).toBeVisible();
  await page.screenshot({ path: `${capturas}/alumno-05-actividad-completada.png` });

  // Error explicado en español.
  await page.locator('[data-actividad="u0-corrige-errores"]').click();
  await page.getByRole("button", { name: "▶ Ejecutar" }).click();
  await expect(page.getByText(/Error en la línea 1/)).toBeVisible();
  await expect(page.getByText(/falta la comilla de cierre/)).toBeVisible();
  await page.screenshot({ path: `${capturas}/alumno-06-error-en-espanol.png` });

  // Predicción.
  await page.locator('[data-actividad="u0-predice-comentarios"]').click();
  await page.getByLabel("¿Qué mostrará en la consola?").fill("Dos\nTres Cuatro\n\nSeis");
  await page.getByRole("button", { name: "Verificar" }).click();
  await expect(page.getByText("✔ ¡Correcto!")).toBeVisible();

  // Estadísticas visibles para el alumno.
  await page.locator('[data-contador="pegados"]').click();
  await expect(page.getByRole("heading", { name: "Mis estadísticas" })).toBeVisible();
  await expect(page.locator(".tarjeta", { hasText: "Intentos de pegar" }).locator(".valor")).toHaveText("1");
  await page.screenshot({ path: `${capturas}/alumno-07-estadisticas.png` });
});

test("programa interactivo con input() en la consola", async ({ page }) => {
  await registrarse(page);
  await page.locator('[data-actividad="u0-hola-mundo"]').click();
  const editor = page.locator("[data-editor] .cm-content");
  await editor.click();
  await page.keyboard.press("Control+A");
  await page.keyboard.press("Delete");
  await page.keyboard.type('nombre = input("Nombre: ")\nprint("Hola,", nombre)', { delay: 10 });
  await page.getByRole("button", { name: "▶ Ejecutar" }).click();
  const dato = page.getByLabel("Dato para el programa");
  await expect(dato).toBeVisible({ timeout: 60_000 });
  await dato.fill("Luis");
  await dato.press("Enter");
  await expect(page.locator("[data-consola]")).toContainText("Hola, Luis");
  await page.screenshot({ path: `${capturas}/alumno-08-input.png` });
});

test("diseño en pantalla de celular", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await registrarse(page);
  await page.getByRole("button", { name: "Temario" }).click();
  await page.locator('[data-actividad="u0-hola-mundo"]').click();
  await expect(page.getByRole("button", { name: "Enunciado" })).toBeVisible();
  await page.screenshot({ path: `${capturas}/alumno-09-celular.png` });
});
