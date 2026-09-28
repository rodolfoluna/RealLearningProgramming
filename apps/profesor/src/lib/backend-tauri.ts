import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { Backend } from "./backend";

export function crearBackendTauri(): Backend {
  return {
    estadoApp: () => invoke("estado_app"),
    async elegirArchivos(titulo, extension, nombreFiltro, varios) {
      const r = await open({ title: titulo, multiple: varios, directory: false, filters: [{ name: nombreFiltro, extensions: [extension] }] });
      if (!r) return [];
      return Array.isArray(r) ? r : [r];
    },
    async elegirCarpeta(titulo) {
      const r = await open({ title: titulo, multiple: false, directory: true });
      return typeof r === "string" ? r : null;
    },
    crearIdentidad: (nombre, contrasena) => invoke("crear_identidad", { nombre, contrasena }),
    desbloquear: (contrasena) => invoke("desbloquear", { contrasena }),
    restaurarIdentidad: (ruta, contrasena) => invoke("restaurar_identidad", { ruta, contrasena }),
    respaldarIdentidad: (carpeta) => invoke("respaldar_identidad", { carpeta }),
    bloquear: () => invoke("bloquear"),
    crearGrupo: (datos) => invoke("crear_grupo", { datos }),
    grupos: () => invoke("grupos"),
    exportarGrupo: (grupoId, carpeta) => invoke("exportar_grupo", { grupoId, carpeta }),
    instalarGrupo: (grupoId, carpetaApp) => invoke("instalar_grupo", { grupoId, carpetaApp }),
    importarEntregas: (rutas, codigosIniciales) => invoke("importar_entregas", { rutas, codigosIniciales }),
    importarCarpeta: (carpeta, codigosIniciales) => invoke("importar_carpeta", { carpeta, codigosIniciales }),
    tablero: (grupoId) => invoke("tablero", { grupoId }),
    detalle: (entregaId) => invoke("detalle", { entregaId }),
    reproduccion: (entregaId, actividadId) => invoke("reproduccion", { entregaId, actividadId }),
    calificar: (perfilId, actividadId, calificacion, comentario) =>
      invoke("calificar", { perfilId, actividadId, calificacion, comentario }),
    exportarCsv: (grupoId, actividades, carpeta) => invoke("exportar_csv", { grupoId, actividades, carpeta }),
    exportarXlsx: (grupoId, actividades, carpeta) => invoke("exportar_xlsx", { grupoId, actividades, carpeta }),
    exportarRetroalimentacion: (grupoId, carpeta) => invoke("exportar_retroalimentacion", { grupoId, carpeta }),
    crearAcceso: (entregaId, carpeta) => invoke("crear_acceso", { entregaId, carpeta }),
  };
}
