// Lo que corre dentro del Worker: el núcleo en WebAssembly y la persistencia en IndexedDB.
//
// - Después de cada operación se guarda el diario de cambios y solo entonces se responde: lo que
//   la interfaz da por guardado ya está en IndexedDB.
// - Un perfil solo se abre en una pestaña a la vez (Web Locks): dos pestañas escribiendo la misma
//   cadena de eventos romperían el historial.
// - Las operaciones llegan en orden (el Worker las encola), así que los diarios no se mezclan.

import type { Almacen } from "./idb";
import type { Diario, GrupoInstalado, NucleoWasm } from "./tipos";

export const OTRA_PESTANA =
  "Tu perfil ya está abierto en otra pestaña o ventana de este navegador. Ciérrala para continuar aquí.";
const GUARDADO_FALLIDO =
  "No se pudo guardar en este navegador. Recarga la página: tu trabajo se conserva hasta lo último que se guardó.";
const CLAVE_GRUPO = "grupo";

/** Candados entre pestañas: devuelven la función para soltarlo, o null si otra lo tiene. */
export interface Candados {
  tomar(nombre: string): Promise<(() => void) | null>;
}

/** Web Locks del navegador (o, si no existen, sin candado). */
export function candadosDelNavegador(): Candados {
  const locks = (globalThis.navigator as Navigator | undefined)?.locks;
  return {
    tomar: (nombre) =>
      locks
        ? new Promise((resolver) => {
            void locks.request(nombre, { ifAvailable: true }, (candado) => {
              if (!candado) {
                resolver(null);
                return;
              }
              return new Promise<void>((soltar) => resolver(soltar));
            });
          })
        : Promise.resolve(() => {}),
  };
}

export function aBase64(bytes: Uint8Array): string {
  let texto = "";
  for (let i = 0; i < bytes.length; i += 0x8000) {
    texto += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(texto);
}

export function deBase64(texto: string): Uint8Array {
  const crudo = atob(texto);
  const bytes = new Uint8Array(crudo.length);
  for (let i = 0; i < crudo.length; i++) bytes[i] = crudo.charCodeAt(i);
  return bytes;
}

function mensaje(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

interface ConPerfil {
  perfil: { perfil_id: string };
}

export class ServicioNucleo {
  private perfilAbierto: string | null = null;
  private soltar: (() => void) | null = null;
  private roto = false;

  constructor(
    private readonly nucleo: NucleoWasm,
    private readonly almacen: Almacen,
    private readonly candados: Candados = candadosDelNavegador(),
  ) {}

  /** Llama al núcleo y guarda el diario antes de responder (también si la operación falló). */
  private async llamar<T>(metodo: string, args: object = {}): Promise<T> {
    if (this.roto) throw new Error(GUARDADO_FALLIDO);
    let resultado: string | undefined;
    let error: unknown;
    try {
      resultado = this.nucleo.llamar(metodo, JSON.stringify(args));
    } catch (e) {
      error = e;
    }
    try {
      const diarios = JSON.parse(this.nucleo.llamar("diario", "{}")) as Diario[];
      for (const d of diarios) await this.almacen.guardar(d.perfil, d.lote);
    } catch (e) {
      // Lo que está en memoria ya no coincide con lo guardado: no se sigue escribiendo.
      this.roto = true;
      console.error("No se pudo guardar el diario", e);
      throw new Error(GUARDADO_FALLIDO);
    }
    if (error !== undefined) throw new Error(mensaje(error));
    return JSON.parse(resultado!) as T;
  }

  private async tomar(perfil: string) {
    const soltar = await this.candados.tomar(`rlp-perfil-${perfil}`);
    if (!soltar) throw new Error(OTRA_PESTANA);
    this.soltar = soltar;
    this.perfilAbierto = perfil;
  }

  private liberar() {
    this.soltar?.();
    this.soltar = null;
    this.perfilAbierto = null;
  }

  private async grupoInstalado(): Promise<GrupoInstalado | null> {
    return (await this.almacen.leerApp(CLAVE_GRUPO)) as GrupoInstalado | null;
  }

  // ------------------------------------------------------------------ sin sesión

  /** Versión, perfiles guardados en este navegador y grupo instalado. */
  async inicio() {
    const v = await this.llamar<{ version: string; dev: boolean }>("version");
    const perfiles = await this.llamar<unknown[]>("perfiles", { metas: await this.almacen.metas() });
    return { ...v, perfiles, grupo: (await this.grupoInstalado())?.info ?? null };
  }

  /** Grupo del profesor (archivo `.rlpg`) para los perfiles nuevos de este navegador. */
  async instalarGrupo(archivo: Uint8Array) {
    const g = await this.llamar<GrupoInstalado>("leer_grupo", { archivo: aBase64(archivo) });
    await this.almacen.escribirApp(CLAVE_GRUPO, g);
    return g.info;
  }

  /** Igual, desde el texto del código QR que muestra la App Profesor. */
  async instalarGrupoQr(contenido: string) {
    const g = await this.llamar<GrupoInstalado>("grupo_qr", { contenido });
    await this.almacen.escribirApp(CLAVE_GRUPO, g);
    return g.info;
  }

  async registrar(numero_control: string, nombre: string, contrasena: string) {
    await this.cerrarSesion();
    const r = await this.llamar<{ estado: ConPerfil; codigo: string }>("registrar", {
      metas: await this.almacen.metas(),
      grupo: (await this.grupoInstalado())?.grupo ?? null,
      numero_control,
      nombre,
      contrasena,
    });
    await this.tomar(r.estado.perfil.perfil_id);
    return r;
  }

  async iniciarSesion(perfil: string, secreto: unknown) {
    await this.cerrarSesion();
    await this.tomar(perfil);
    try {
      return await this.llamar<ConPerfil>("iniciar_sesion", {
        instantanea: await this.almacen.instantanea(perfil),
        secreto,
      });
    } catch (e) {
      this.liberar();
      throw e;
    }
  }

  /** Crea en este navegador un perfil a partir de una entrega `.rlp` (continuar aquí). */
  async restaurar(archivo: Uint8Array, secreto: unknown) {
    await this.cerrarSesion();
    const estado = await this.llamar<ConPerfil>("restaurar", {
      metas: await this.almacen.metas(),
      archivo: aBase64(archivo),
      secreto,
    });
    await this.tomar(estado.perfil.perfil_id);
    return estado;
  }

  async leerAcceso(contenido: string) {
    return this.llamar<{ perfil_id: string; perfil_local: boolean }>("leer_acceso", {
      metas: await this.almacen.metas(),
      archivo: contenido,
    });
  }

  /** Entra con el archivo de acceso del profesor; `entrega` solo si el perfil no está aquí. */
  async entrarConAcceso(acceso: string, temporal: string, nueva: string, entrega: Uint8Array | null) {
    await this.cerrarSesion();
    const info = await this.leerAcceso(acceso);
    if (info.perfil_local) {
      await this.tomar(info.perfil_id);
      try {
        return await this.llamar<ConPerfil>("entrar_con_acceso", {
          acceso,
          temporal,
          nueva,
          instantanea: await this.almacen.instantanea(info.perfil_id),
        });
      } catch (e) {
        this.liberar();
        throw e;
      }
    }
    const estado = await this.llamar<ConPerfil>("entrar_con_acceso", {
      acceso,
      temporal,
      nueva,
      entrega: entrega ? aBase64(entrega) : null,
      metas: await this.almacen.metas(),
    });
    await this.tomar(estado.perfil.perfil_id);
    return estado;
  }

  async cerrarSesion() {
    if (this.perfilAbierto === null) return;
    try {
      await this.llamar("cerrar_sesion");
    } finally {
      this.liberar();
    }
  }

  // ------------------------------------------------------------------ con sesión

  /**
   * Operaciones de la sesión con los mismos nombres y argumentos que los comandos de la app
   * nativa: estado, estadisticas, abrir_actividad, guardar_edicion, reiniciar_actividad,
   * registrar_pruebas, registrar_respuesta, registrar_pista, registrar_evento y
   * cambiar_contrasena.
   */
  async sesion<T = unknown>(operacion: string, args: object = {}) {
    if (!OPERACIONES_SESION.has(operacion)) throw new Error(`Operación no permitida: ${operacion}`);
    return this.llamar<T>(operacion, args);
  }

  async exportar() {
    const r = await this.llamar<{ nombre: string; archivo: string }>("exportar");
    return { nombre: r.nombre, archivo: deBase64(r.archivo) };
  }

  async importarAvances(archivo: Uint8Array) {
    return this.llamar("importar_avances", { archivo: aBase64(archivo) });
  }

  async importarRetroalimentacion(archivo: Uint8Array) {
    return this.llamar("importar_retroalimentacion", { archivo: aBase64(archivo) });
  }

  /** Une el perfil a un grupo; si este navegador no tenía grupo, queda como el suyo. */
  async unirseGrupo(archivo: Uint8Array) {
    return this.unido(await this.llamar<GrupoInstalado>("unirse_grupo", { archivo: aBase64(archivo) }));
  }

  async unirseGrupoQr(contenido: string) {
    return this.unido(await this.llamar<GrupoInstalado>("unirse_grupo_qr", { contenido }));
  }

  private async unido(g: GrupoInstalado) {
    if (!(await this.grupoInstalado())) await this.almacen.escribirApp(CLAVE_GRUPO, g);
    return g.info;
  }
}

/** Lo que la interfaz puede pedirle al Worker. */
export const METODOS_PUBLICOS = new Set<string>([
  "inicio",
  "instalarGrupo",
  "instalarGrupoQr",
  "registrar",
  "iniciarSesion",
  "restaurar",
  "leerAcceso",
  "entrarConAcceso",
  "cerrarSesion",
  "sesion",
  "exportar",
  "importarAvances",
  "importarRetroalimentacion",
  "unirseGrupo",
  "unirseGrupoQr",
]);

const OPERACIONES_SESION = new Set([
  "estado",
  "estadisticas",
  "abrir_actividad",
  "guardar_edicion",
  "reiniciar_actividad",
  "registrar_pruebas",
  "registrar_respuesta",
  "registrar_pista",
  "registrar_evento",
  "cambiar_contrasena",
]);
