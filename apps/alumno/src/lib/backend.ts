// Acceso al núcleo. En la app de escritorio/Android usa los comandos de Tauri; en un navegador
// normal (desarrollo y pruebas de interfaz) usa una simulación en memoria.

import type { LoteOperaciones } from "@rlp/editor";
import type {
  EstadoActividad,
  EstadoAlumno,
  EstadoApp,
  Estadisticas,
  GrupoInfo,
  InfoAcceso,
  ResultadoGuardado,
  Retroalimentacion,
  ResumenImportacion,
  Secreto,
} from "./tipos";

export interface Backend {
  estadoApp(): Promise<EstadoApp>;
  /** Abre un diálogo para elegir un archivo; devuelve la ruta o null. */
  elegirArchivo(titulo: string, extension: string, nombreFiltro: string): Promise<string | null>;
  elegirCarpeta(titulo: string): Promise<string | null>;
  /** Diálogo "Guardar como" (en Android devuelve una URI content://). */
  elegirDestino(titulo: string, nombre: string): Promise<string | null>;
  /** Escanea un código QR con la cámara (solo celular). */
  escanearQr(): Promise<string | null>;
  abrirCarpeta(ruta: string): Promise<void>;
  importarGrupo(ruta: string): Promise<GrupoInfo>;
  importarGrupoQr(contenido: string): Promise<GrupoInfo>;
  unirseGrupoQr(contenido: string): Promise<GrupoInfo>;
  registrar(numeroControl: string, nombre: string, contrasena: string): Promise<{ estado: EstadoAlumno; codigo: string }>;
  iniciarSesion(carpeta: string, secreto: Secreto): Promise<EstadoAlumno>;
  restaurar(ruta: string, secreto: Secreto): Promise<EstadoAlumno>;
  cerrarSesion(): Promise<void>;
  estado(): Promise<EstadoAlumno>;
  estadisticas(): Promise<Estadisticas>;
  abrirActividad(id: string, codigoInicial: string): Promise<EstadoActividad>;
  guardarEdicion(id: string, lote: LoteOperaciones, texto: string): Promise<ResultadoGuardado>;
  reiniciarActividad(id: string, codigoInicial: string): Promise<EstadoActividad>;
  registrarPruebas(id: string, pasadas: number, total: number, puntos: number): Promise<EstadoActividad>;
  registrarRespuesta(id: string, respuesta: string, correcta: boolean, puntos: number): Promise<EstadoActividad>;
  registrarPista(id: string, numero: number): Promise<EstadoActividad>;
  registrarEvento(tipo: string, actividad: string | null, datos: Record<string, unknown>): Promise<void>;
  exportar(carpeta: string): Promise<string>;
  /** Exporta al archivo elegido con `elegirDestino`; devuelve el nombre de la entrega. */
  exportarA(destino: string): Promise<string>;
  importarAvances(ruta: string): Promise<ResumenImportacion>;
  importarRetroalimentacion(ruta: string): Promise<Retroalimentacion>;
  /** Lee un archivo de acceso del profesor (comprueba su firma). */
  leerAcceso(ruta: string): Promise<InfoAcceso>;
  /** Entra con el archivo de acceso; `rutaEntrega` solo si el perfil no está en esta carpeta. */
  entrarConAcceso(rutaAcceso: string, temporal: string, nueva: string, rutaEntrega: string | null): Promise<EstadoAlumno>;
  cambiarContrasena(actual: string, nueva: string): Promise<void>;
  unirseGrupo(ruta: string): Promise<GrupoInfo>;
  generarEjecutable(nombre: string, codigo: string, alProgreso: (linea: string) => void): Promise<string>;
}

export function enTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

let instancia: Promise<Backend> | null = null;

export function backend(): Promise<Backend> {
  if (!instancia) {
    instancia = enTauri()
      ? import("./backend-tauri").then((m) => m.crearBackendTauri())
      : import("./backend-simulado").then((m) => m.crearBackendSimulado());
  }
  return instancia;
}
