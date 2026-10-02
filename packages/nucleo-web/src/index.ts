// Núcleo de la App Alumno para el navegador: cada método corre en el Worker (worker.ts) y
// devuelve una promesa. Antes hay que compilarlo con `pnpm wasm`.

import type { ServicioNucleo } from "./servicio";

export { OTRA_PESTANA } from "./servicio";
export type { GrupoInstalado, MetaPerfil } from "./tipos";

type Remoto<T> = {
  [K in keyof T]: T[K] extends (...args: infer A) => infer R ? (...args: A) => Promise<Awaited<R>> : never;
};

export type NucleoWeb = Remoto<ServicioNucleo>;

interface Pendiente {
  resolver: (v: unknown) => void;
  rechazar: (e: Error) => void;
}

export function crearNucleoWeb(worker?: Worker): NucleoWeb {
  const w = worker ?? new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
  const pendientes = new Map<number, Pendiente>();
  let siguiente = 0;
  w.onmessage = (e: MessageEvent<{ id: number; ok: boolean; valor?: unknown; error?: string }>) => {
    const p = pendientes.get(e.data.id);
    pendientes.delete(e.data.id);
    if (e.data.ok) p?.resolver(e.data.valor);
    else p?.rechazar(new Error(e.data.error));
  };
  w.onerror = (e) => {
    for (const p of pendientes.values()) p.rechazar(new Error(`El núcleo web no pudo iniciar: ${e.message}`));
    pendientes.clear();
  };
  return new Proxy({} as NucleoWeb, {
    get(_, metodo) {
      // Que el objeto no parezca una promesa (`await nucleo` no debe llamar al Worker).
      if (typeof metodo !== "string" || metodo === "then") return undefined;
      return (...args: unknown[]) =>
        new Promise((resolver, rechazar) => {
          const id = ++siguiente;
          pendientes.set(id, { resolver, rechazar });
          w.postMessage({ id, metodo, args });
        });
    },
  });
}
