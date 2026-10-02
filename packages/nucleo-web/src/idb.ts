// Perfiles de la versión web en IndexedDB. Cada fila es la misma que en el `alumno.db` de la app
// nativa (ver crates/rlp-core/src/deposito.rs), con el perfil como primera parte de la llave.

import type { FilaActividad, FilaEvento, Lote, MetaPerfil } from "./tipos";

const VERSION = 1;

export interface Almacen {
  /** Metadatos de todos los perfiles guardados en este navegador. */
  metas(): Promise<MetaPerfil[]>;
  /** Contenido completo de un perfil, para abrirlo. */
  instantanea(perfil: string): Promise<Lote>;
  /** Escribe un diario de cambios en una sola transacción. */
  guardar(perfil: string, lote: Lote): Promise<void>;
  leerApp(clave: string): Promise<unknown>;
  escribirApp(clave: string, valor: unknown): Promise<void>;
}

function promesa<T>(peticion: IDBRequest<T>): Promise<T> {
  return new Promise((resolver, rechazar) => {
    peticion.onsuccess = () => resolver(peticion.result);
    peticion.onerror = () => rechazar(peticion.error);
  });
}

function terminar(tx: IDBTransaction): Promise<void> {
  return new Promise((resolver, rechazar) => {
    tx.oncomplete = () => resolver();
    tx.onerror = () => rechazar(tx.error);
    tx.onabort = () => rechazar(tx.error ?? new Error("transacción cancelada"));
  });
}

/** Todas las filas de un perfil: llaves de la forma [perfil, …]. */
function delPerfil(perfil: string): IDBKeyRange {
  return IDBKeyRange.bound([perfil], [perfil, []]);
}

export async function abrirAlmacen(nombre = "rlp-alumno"): Promise<Almacen> {
  const peticion = indexedDB.open(nombre, VERSION);
  peticion.onupgradeneeded = () => {
    const db = peticion.result;
    db.createObjectStore("meta", { keyPath: ["perfil", "clave"] });
    db.createObjectStore("actividades", { keyPath: ["perfil", "id"] });
    db.createObjectStore("eventos", { keyPath: ["perfil", "dispositivo", "seq"] });
    db.createObjectStore("app", { keyPath: "clave" });
  };
  const db = await promesa(peticion);

  return {
    async metas() {
      const filas = await promesa(db.transaction("meta").objectStore("meta").getAll());
      return filas
        .filter((f) => f.clave === "perfil")
        .map((f) => JSON.parse(f.valor) as MetaPerfil);
    },

    async instantanea(perfil) {
      const tx = db.transaction(["meta", "actividades", "eventos"]);
      const rango = delPerfil(perfil);
      const [meta, actividades, eventos] = await Promise.all([
        promesa(tx.objectStore("meta").getAll(rango)),
        promesa(tx.objectStore("actividades").getAll(rango)),
        promesa(tx.objectStore("eventos").getAll(rango)),
      ]);
      return {
        meta: Object.fromEntries(meta.map((f) => [f.clave, f.valor])),
        actividades: actividades.map(({ id, datos }): FilaActividad => ({ id, datos })),
        eventos: eventos.map(
          ({ dispositivo, seq, datos, hash, firma, llave }): FilaEvento => ({ dispositivo, seq, datos, hash, firma, llave }),
        ),
      };
    },

    async guardar(perfil, lote) {
      const tx = db.transaction(["meta", "actividades", "eventos"], "readwrite");
      const fin = terminar(tx);
      for (const [clave, valor] of Object.entries(lote.meta)) tx.objectStore("meta").put({ perfil, clave, valor });
      for (const a of lote.actividades) tx.objectStore("actividades").put({ perfil, ...a });
      // `add`: un evento nunca se reescribe; si ya existe, la transacción completa se cancela.
      for (const e of lote.eventos) tx.objectStore("eventos").add({ perfil, ...e });
      await fin;
    },

    async leerApp(clave) {
      const fila = await promesa(db.transaction("app").objectStore("app").get(clave));
      return fila?.valor ?? null;
    },

    async escribirApp(clave, valor) {
      const tx = db.transaction("app", "readwrite");
      const fin = terminar(tx);
      tx.objectStore("app").put({ clave, valor });
      await fin;
    },
  };
}
