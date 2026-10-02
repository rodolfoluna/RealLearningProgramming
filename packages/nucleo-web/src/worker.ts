// Worker del núcleo web: carga rlp-web (WebAssembly), abre IndexedDB y atiende las llamadas en
// orden. La derivación de llaves (Argon2id) no bloquea la interfaz.

import iniciar, { Nucleo } from "../generado/rlp_web.js";
import { abrirAlmacen } from "./idb";
import { METODOS_PUBLICOS, ServicioNucleo } from "./servicio";

interface Llamada {
  id: number;
  metodo: keyof ServicioNucleo;
  args: unknown[];
}

const servicio = (async () => {
  await iniciar();
  return new ServicioNucleo(new Nucleo(), await abrirAlmacen());
})();

let cola: Promise<unknown> = Promise.resolve();

self.onmessage = (e: MessageEvent<Llamada>) => {
  const { id, metodo, args } = e.data;
  cola = cola.then(async () => {
    try {
      const s = await servicio;
      if (!METODOS_PUBLICOS.has(metodo)) throw new Error(`Operación desconocida: ${String(metodo)}`);
      const valor = await (s[metodo] as (...a: unknown[]) => unknown).apply(s, args);
      self.postMessage({ id, ok: true, valor });
    } catch (err) {
      self.postMessage({ id, ok: false, error: err instanceof Error ? err.message : String(err) });
    }
  });
};
