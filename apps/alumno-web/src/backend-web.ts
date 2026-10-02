// Núcleo de la versión web: rlp-core en WebAssembly (Worker de @rlp/nucleo-web) con los perfiles
// en IndexedDB. Los archivos se eligen con el selector del navegador y las entregas se descargan.

import type {
  Backend,
  EstadoActividad,
  EstadoAlumno,
  EstadoApp,
  Estadisticas,
  GrupoInfo,
  InfoAcceso,
  ResultadoGuardado,
  ResumenImportacion,
  Retroalimentacion,
} from "@rlp/alumno-ui";
import { avisar, conDialogo } from "@rlp/alumno-ui";
import { crearNucleoWeb } from "@rlp/nucleo-web";

/** Archivos elegidos por el alumno, por nombre (lo que la interfaz llama "ruta"). */
const archivos = new Map<string, File>();

/** En iPhone/iPad, `accept` con extensiones propias deja los archivos sin poder elegirse. */
const esIos = /iPad|iPhone|iPod/.test(navigator.userAgent) || (navigator.platform === "MacIntel" && navigator.maxTouchPoints > 1);

function elegirArchivo(extension: string): Promise<string | null> {
  return conDialogo(
    () =>
      new Promise<string | null>((resolver) => {
        let listo = false;
        const terminar = (nombre: string | null) => {
          if (!listo) resolver(nombre);
          listo = true;
        };
        const campo = document.createElement("input");
        campo.type = "file";
        if (!esIos) campo.accept = `.${extension}`;
        campo.addEventListener("change", () => {
          const archivo = campo.files?.[0];
          if (!archivo) return terminar(null);
          archivos.set(archivo.name, archivo);
          terminar(archivo.name);
        });
        campo.addEventListener("cancel", () => terminar(null));
        // Navegadores sin el evento `cancel`: al volver a la página sin archivo, se canceló.
        addEventListener("focus", () => setTimeout(() => campo.files?.length || terminar(null), 1500), { once: true });
        campo.click();
      }),
  );
}

async function leer(nombre: string): Promise<Uint8Array> {
  const archivo = archivos.get(nombre);
  if (!archivo) throw new Error("Vuelve a elegir el archivo.");
  return new Uint8Array(await archivo.arrayBuffer());
}

async function leerTexto(nombre: string): Promise<string> {
  return new TextDecoder().decode(await leer(nombre));
}

function descargar(nombre: string, datos: Uint8Array) {
  const url = URL.createObjectURL(new Blob([datos as BlobPart], { type: "application/octet-stream" }));
  const enlace = document.createElement("a");
  enlace.href = url;
  enlace.download = nombre;
  document.body.append(enlace);
  enlace.click();
  enlace.remove();
  setTimeout(() => URL.revokeObjectURL(url), 60_000);
}

/** El Worker devuelve el mismo JSON que los comandos de la app nativa. */
const como = <T>(valor: unknown) => valor as T;

/** Cada cuántos días se recuerda exportar la entrega (en el navegador también es el respaldo). */
const DIAS_RESPALDO = 7;

function clave(perfil: string) {
  return `rlp-exportado-${perfil}`;
}

function recordarRespaldo(estado: EstadoAlumno) {
  let ultima = estado.perfil.creado;
  try {
    ultima = Number(localStorage.getItem(clave(estado.perfil.perfil_id))) || ultima;
  } catch {
    /* sin localStorage: se cuenta desde el registro */
  }
  const dias = Math.floor((Date.now() - ultima) / 86_400_000);
  if (dias >= DIAS_RESPALDO) {
    setTimeout(() => avisar(`Hace ${dias} días que no exportas tu entrega. Expórtala: también es tu respaldo.`, 9000), 2500);
  }
}

export function crearBackendWeb(): Backend {
  const nucleo = crearNucleoWeb();
  const sesion = async <T>(operacion: string, args: object = {}) => como<T>(await nucleo.sesion(operacion, args));
  let perfil: string | null = null;

  /** Recuerda qué perfil está abierto (para el recordatorio de respaldo). */
  function abierto(estado: EstadoAlumno): EstadoAlumno {
    perfil = estado.perfil.perfil_id;
    recordarRespaldo(estado);
    return estado;
  }

  async function exportar(): Promise<string> {
    const { nombre, archivo } = await nucleo.exportar();
    descargar(nombre, archivo);
    try {
      if (perfil) localStorage.setItem(clave(perfil), String(Date.now()));
    } catch {
      /* sin localStorage */
    }
    return nombre;
  }

  return {
    async estadoApp(): Promise<EstadoApp> {
      const i = await nucleo.inicio();
      return {
        version: i.version,
        plataforma: "web",
        carpeta_datos: "este navegador",
        escribible: true,
        grupo: como<GrupoInfo | null>(i.grupo),
        perfiles: como<EstadoApp["perfiles"]>(i.perfiles),
        puede_generar_exe: false,
        dev: i.dev,
        autoprueba: null,
      };
    },
    elegirArchivo: (_titulo, extension) => elegirArchivo(extension),
    elegirCarpeta: async () => null,
    // La entrega se descarga con el nombre que le da el núcleo.
    elegirDestino: async (_titulo, nombre) => nombre,
    escanearQr: () => import("./qr").then((m) => conDialogo(() => m.escanearQr())),
    abrirCarpeta: async () => {},
    importarGrupo: async (ruta) => como<GrupoInfo>(await nucleo.instalarGrupo(await leer(ruta))),
    importarGrupoQr: async (contenido) => como<GrupoInfo>(await nucleo.instalarGrupoQr(contenido)),
    unirseGrupoQr: async (contenido) => como<GrupoInfo>(await nucleo.unirseGrupoQr(contenido)),
    registrar: async (numeroControl, nombre, contrasena) => {
      const r = como<{ estado: EstadoAlumno; codigo: string }>(await nucleo.registrar(numeroControl, nombre, contrasena));
      perfil = r.estado.perfil.perfil_id;
      return r;
    },
    // En la web, la "carpeta" de un perfil es su identificador.
    iniciarSesion: async (id, secreto) => abierto(como<EstadoAlumno>(await nucleo.iniciarSesion(id, secreto))),
    restaurar: async (ruta, secreto) => abierto(como<EstadoAlumno>(await nucleo.restaurar(await leer(ruta), secreto))),
    cerrarSesion: async () => {
      perfil = null;
      await nucleo.cerrarSesion();
    },
    estado: () => sesion<EstadoAlumno>("estado"),
    estadisticas: () => sesion<Estadisticas>("estadisticas"),
    abrirActividad: (id, codigoInicial) => sesion<EstadoActividad>("abrir_actividad", { id, codigo_inicial: codigoInicial }),
    guardarEdicion: (id, lote, texto) => sesion<ResultadoGuardado>("guardar_edicion", { id, lote, texto }),
    reiniciarActividad: (id, codigoInicial) =>
      sesion<EstadoActividad>("reiniciar_actividad", { id, codigo_inicial: codigoInicial }),
    registrarPruebas: (id, pasadas, total, puntos) =>
      sesion<EstadoActividad>("registrar_pruebas", { id, pasadas, total, puntos }),
    registrarRespuesta: (id, respuesta, correcta, puntos) =>
      sesion<EstadoActividad>("registrar_respuesta", { id, respuesta, correcta, puntos }),
    registrarPista: (id, numero) => sesion<EstadoActividad>("registrar_pista", { id, numero }),
    registrarEvento: async (tipo, actividad, datos) => {
      await sesion("registrar_evento", { tipo, actividad, datos });
    },
    exportar: () => exportar(),
    exportarA: () => exportar(),
    importarAvances: async (ruta) => como<ResumenImportacion>(await nucleo.importarAvances(await leer(ruta))),
    importarRetroalimentacion: async (ruta) =>
      como<Retroalimentacion>(await nucleo.importarRetroalimentacion(await leer(ruta))),
    leerAcceso: async (ruta) => como<InfoAcceso>(await nucleo.leerAcceso(await leerTexto(ruta))),
    entrarConAcceso: async (rutaAcceso, temporal, nueva, rutaEntrega) =>
      abierto(
        como<EstadoAlumno>(
          await nucleo.entrarConAcceso(await leerTexto(rutaAcceso), temporal, nueva, rutaEntrega ? await leer(rutaEntrega) : null),
        ),
      ),
    cambiarContrasena: async (actual, nueva) => {
      await sesion("cambiar_contrasena", { actual, nueva });
    },
    unirseGrupo: async (ruta) => como<GrupoInfo>(await nucleo.unirseGrupo(await leer(ruta))),
    generarEjecutable: async () => {
      throw new Error("Crear un .exe solo se puede en la app de Windows.");
    },
  };
}
