import { expect, test, type Page } from "@playwright/test";

// Pruebas de concepto: el editor bloquea todas las vías de pegado y registra copias y operaciones.

async function abrir(page: Page) {
  await page.goto("/");
  await page.waitForFunction(() => (window as any).rlp?.editor);
}

const texto = (page: Page) => page.evaluate(() => (window as any).rlp.editor.texto as string);
const registro = (page: Page) => page.evaluate(() => (window as any).rlp.registro);

test("Ctrl+V y Shift+Insert no pegan y se cuentan", async ({ page }) => {
  await abrir(page);
  await page.evaluate(() => navigator.clipboard.writeText("print('copiado de internet')"));
  await page.click(".cm-content");
  await page.keyboard.press("Control+End");
  await page.keyboard.press("Control+V");
  await page.keyboard.press("Shift+Insert");
  expect(await texto(page)).toBe("print('hola')\n");
  const r = await registro(page);
  expect(r.pegados.length).toBeGreaterThanOrEqual(1);
  expect(r.pegados.every((p: any) => p.permitido === false)).toBe(true);
});

test("evento paste sintético y drop se bloquean", async ({ page }) => {
  await abrir(page);
  await page.evaluate(() => {
    const cont = document.querySelector(".cm-content")!;
    const dt = new DataTransfer();
    dt.setData("text/plain", "x = 1\ny = 2\n");
    cont.dispatchEvent(new ClipboardEvent("paste", { clipboardData: dt, bubbles: true, cancelable: true }));
    const dt2 = new DataTransfer();
    dt2.setData("text/plain", "for i in range(10): print(i)");
    cont.dispatchEvent(new DragEvent("drop", { dataTransfer: dt2, bubbles: true, cancelable: true, clientX: 10, clientY: 10 }));
  });
  expect(await texto(page)).toBe("print('hola')\n");
  const r = await registro(page);
  expect(r.pegados.map((p: any) => p.via)).toEqual(expect.arrayContaining(["teclado", "soltar"]));
});

test("inserción masiva por teclado (autoescritor/IME) se rechaza", async ({ page }) => {
  await abrir(page);
  await page.click(".cm-content");
  await page.keyboard.press("Control+End");
  await page.keyboard.insertText("while True: print('rapido')");
  expect(await texto(page)).toBe("print('hola')\n");
  expect((await registro(page)).sospechas.length).toBe(1);
});

test("tecleo normal se registra como operaciones reproducibles", async ({ page }) => {
  await abrir(page);
  await page.click(".cm-content");
  await page.keyboard.press("Control+End");
  await page.keyboard.type("x = 5\nif x > 3:\nprint(x)", { delay: 15 });
  await page.keyboard.press("Backspace");
  await page.evaluate(() => (window as any).rlp.editor.vaciarOperaciones());
  const final = await texto(page);
  expect(final).toContain("if x > 3:\n    print(x");
  const lotes = (await registro(page)).lotes;
  const ops = lotes.flatMap((l: any) => l.ops);
  expect(ops.length).toBeGreaterThan(10);
  // Reproducir las operaciones sobre el texto inicial da exactamente el texto final.
  const reproducido = ops.reduce((t: string, [, desde, hasta, ins]: any) => t.slice(0, desde) + ins + t.slice(hasta), "print('hola')\n");
  expect(reproducido).toBe(final);
  expect(ops.some((o: any) => o[4] === "i")).toBe(true); // sangría automática tras ':'
});

test("copiar se cuenta; la política 'propio' permite pegar solo lo copiado del editor", async ({ page }) => {
  await abrir(page);
  await page.evaluate(() => (window as any).rlp.fijarPolitica("propio"));
  await page.click(".cm-content");
  await page.keyboard.press("Control+A");
  await page.keyboard.press("Control+C");
  await page.keyboard.press("Control+End");
  await page.keyboard.press("Control+V");
  expect(await texto(page)).toBe("print('hola')\nprint('hola')\n");
  await page.evaluate(() => navigator.clipboard.writeText("print('de afuera')"));
  await page.keyboard.press("Control+V");
  expect(await texto(page)).toBe("print('hola')\nprint('hola')\n");
  const r = await registro(page);
  expect(r.copias.length).toBe(1);
  expect(r.pegados.map((p: any) => p.permitido)).toEqual([true, false]);
});

test("el menú contextual no ofrece pegar", async ({ page }) => {
  await abrir(page);
  await page.click(".cm-content", { button: "right" });
  const pegar = page.getByRole("menuitem", { name: "Pegar" });
  await expect(pegar).toBeDisabled();
});
