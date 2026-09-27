import { expect, test, type Page } from "@playwright/test";

// Pruebas de concepto: Pyodide en un Web Worker con input() síncrono, interrupción y pruebas.

async function abrir(page: Page) {
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => (window as any).rlp?.aislado())).toBe(true);
  await page.evaluate(() => (window as any).rlp.ejecutor.iniciar());
}

test("ejecuta un programa con input() y salida en vivo", async ({ page }) => {
  await abrir(page);
  const resultado = await page.evaluate(async () => {
    const { ejecutor } = (window as any).rlp;
    let salida = "";
    const r = await ejecutor.ejecutar('nombre = input("¿Cómo te llamas? ")\nprint(f"Hola, {nombre}!")\n', {
      salida: (t: string) => (salida += t),
      entradaSolicitada: () => setTimeout(() => ejecutor.enviarEntrada("Ana"), 50),
    });
    return { r, salida };
  });
  expect(resultado.r.estado).toBe("ok");
  expect(resultado.salida).toContain("¿Cómo te llamas? ");
  expect(resultado.salida).toContain("Hola, Ana!");
});

test("detiene un ciclo infinito", async ({ page }) => {
  await abrir(page);
  const r = await page.evaluate(async () => {
    const { ejecutor } = (window as any).rlp;
    const p = ejecutor.ejecutar("while True:\n    pass\n");
    setTimeout(() => ejecutor.detener(), 300);
    const r1 = await p;
    // El intérprete sigue disponible después.
    let salida = "";
    const r2 = await ejecutor.ejecutar("print(2 + 2)", { salida: (t: string) => (salida += t) });
    return { r1, r2, salida };
  });
  expect(r.r1.estado).toBe("detenido");
  expect(r.r2.estado).toBe("ok");
  expect(r.salida.trim()).toBe("4");
});

test("detiene un programa que espera input()", async ({ page }) => {
  await abrir(page);
  const r = await page.evaluate(async () => {
    const { ejecutor } = (window as any).rlp;
    const p = ejecutor.ejecutar("x = input('dato: ')\nprint(x)");
    setTimeout(() => ejecutor.detener(), 300);
    return p;
  });
  expect(r.estado).toBe("detenido");
});

test("time.sleep funciona y es interrumpible", async ({ page }) => {
  await abrir(page);
  const r = await page.evaluate(async () => {
    const { ejecutor } = (window as any).rlp;
    const t0 = performance.now();
    const corto = await ejecutor.ejecutar("import time\ntime.sleep(0.3)\nprint('ok')");
    const dur = performance.now() - t0;
    const p = ejecutor.ejecutar("import time\ntime.sleep(30)");
    setTimeout(() => ejecutor.detener(), 200);
    const largo = await p;
    return { corto, dur, largo };
  });
  expect(r.corto.estado).toBe("ok");
  expect(r.dur).toBeGreaterThanOrEqual(250);
  expect(r.largo.estado).toBe("detenido");
});

test("reporta errores en español con línea", async ({ page }) => {
  await abrir(page);
  const r = await page.evaluate(() =>
    (window as any).rlp.ejecutor.ejecutar('edad = input("Edad: ")\nprint(edad + 1)', {
      entradaSolicitada: () => setTimeout(() => (window as any).rlp.ejecutor.enviarEntrada("15"), 20),
    }),
  );
  expect(r.estado).toBe("error");
  expect(r.error.tipo).toBe("TypeError");
  expect(r.error.linea).toBe(2);
  expect(r.error.explicacion).toMatch(/texto|str\(\)/);
});

test("revisa sintaxis sin ejecutar", async ({ page }) => {
  await abrir(page);
  const e = await page.evaluate(() => (window as any).rlp.ejecutor.sintaxis("if 3 > 2\n    print(1)"));
  expect(e.tipo).toBe("SyntaxError");
  expect(e.linea).toBe(1);
  expect(e.explicacion).toContain("':'");
});

test("corre pruebas automáticas y corta las que no terminan", async ({ page }) => {
  await abrir(page);
  const r = await page.evaluate(() =>
    (window as any).rlp.ejecutor.probar("n = int(input())\nwhile n != 0:\n    n = n - 2\nprint('fin', n)", [
      { entrada: "4", salida: ["fin 0"] },
      { entrada: "3", salida: ["fin"] },
      { entrada: "6", salida: ["fin 0"] },
    ]),
  );
  expect(r.total).toBe(3);
  expect(r.resultados[0].paso).toBe(true);
  expect(r.resultados[1].tiempo_agotado).toBe(true);
  expect(r.resultados[2].paso).toBe(true);
});
