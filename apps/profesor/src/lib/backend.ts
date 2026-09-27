import type {
  DetalleEntrega,
  EstadoApp,
  FilaTablero,
  GrupoInfo,
  LineaDeTiempo,
  NuevoGrupo,
  ResultadoImportacion,
} from "./tipos";

export interface Backend {
  estadoApp(): Promise<EstadoApp>;
  elegirArchivos(titulo: string, extension: string, nombreFiltro: string, varios: boolean): Promise<string[]>;
  elegirCarpeta(titulo: string): Promise<string | null>;
  crearIdentidad(nombre: string, contrasena: string): Promise<void>;
  desbloquear(contrasena: string): Promise<void>;
  restaurarIdentidad(ruta: string, contrasena: string): Promise<void>;
  respaldarIdentidad(carpeta: string): Promise<string>;
  bloquear(): Promise<void>;
  crearGrupo(datos: NuevoGrupo): Promise<GrupoInfo>;
  grupos(): Promise<GrupoInfo[]>;
  exportarGrupo(grupoId: string, carpeta: string): Promise<string>;
  instalarGrupo(grupoId: string, carpetaApp: string): Promise<string>;
  importarEntregas(rutas: string[], codigosIniciales: Record<string, string>): Promise<ResultadoImportacion[]>;
  importarCarpeta(carpeta: string, codigosIniciales: Record<string, string>): Promise<ResultadoImportacion[]>;
  tablero(grupoId: string | null): Promise<FilaTablero[]>;
  detalle(entregaId: number): Promise<DetalleEntrega>;
  reproduccion(entregaId: number, actividadId: string): Promise<LineaDeTiempo>;
  calificar(perfilId: string, actividadId: string, calificacion: number | null, comentario: string): Promise<void>;
  exportarCsv(grupoId: string | null, actividades: [string, string][], carpeta: string): Promise<string>;
  exportarXlsx(grupoId: string | null, actividades: [string, string][], carpeta: string): Promise<string>;
}

let instancia: Promise<Backend> | null = null;

export function backend(): Promise<Backend> {
  if (!instancia) {
    instancia =
      typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
        ? import("./backend-tauri").then((m) => m.crearBackendTauri())
        : import("./backend-simulado").then((m) => m.crearBackendSimulado());
  }
  return instancia;
}
