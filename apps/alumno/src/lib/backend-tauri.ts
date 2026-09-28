import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { Backend } from "./backend";
import { conDialogo } from "./dialogos";

export function crearBackendTauri(): Backend {
  return {
    estadoApp: () => invoke("estado_app"),
    async elegirArchivo(titulo, extension, nombreFiltro) {
      const r = await conDialogo(() =>
        open({ title: titulo, multiple: false, directory: false, filters: [{ name: nombreFiltro, extensions: [extension] }] }),
      );
      return typeof r === "string" ? r : null;
    },
    async elegirCarpeta(titulo) {
      const r = await conDialogo(() => open({ title: titulo, multiple: false, directory: true }));
      return typeof r === "string" ? r : null;
    },
    abrirCarpeta: (ruta) => invoke("abrir_carpeta", { ruta }),
    importarGrupo: (ruta) => invoke("importar_grupo", { ruta }),
    registrar: (numeroControl, nombre, contrasena) => invoke("registrar", { numeroControl, nombre, contrasena }),
    iniciarSesion: (carpeta, secreto) => invoke("iniciar_sesion", { carpeta, secreto }),
    restaurar: (ruta, secreto) => invoke("restaurar", { ruta, secreto }),
    cerrarSesion: () => invoke("cerrar_sesion"),
    estado: () => invoke("estado"),
    estadisticas: () => invoke("estadisticas"),
    abrirActividad: (id, codigoInicial) => invoke("abrir_actividad", { id, codigoInicial }),
    guardarEdicion: (id, lote, texto) => invoke("guardar_edicion", { id, lote, texto }),
    reiniciarActividad: (id, codigoInicial) => invoke("reiniciar_actividad", { id, codigoInicial }),
    registrarPruebas: (id, pasadas, total, puntos) => invoke("registrar_pruebas", { id, pasadas, total, puntos }),
    registrarRespuesta: (id, respuesta, correcta, puntos) =>
      invoke("registrar_respuesta", { id, respuesta, correcta, puntos }),
    registrarPista: (id, numero) => invoke("registrar_pista", { id, numero }),
    registrarEvento: (tipo, actividad, datos) => invoke("registrar_evento", { tipo, actividad, datos }),
    exportar: (carpeta) => invoke("exportar", { carpeta }),
    importarAvances: (ruta) => invoke("importar_avances", { ruta }),
    importarRetroalimentacion: (ruta) => invoke("importar_retroalimentacion", { ruta }),
    leerAcceso: (ruta) => invoke("leer_acceso", { ruta }),
    entrarConAcceso: (rutaAcceso, temporal, nueva, rutaEntrega) =>
      invoke("entrar_con_acceso", { rutaAcceso, temporal, nueva, rutaEntrega }),
    cambiarContrasena: (actual, nueva) => invoke("cambiar_contrasena", { actual, nueva }),
    unirseGrupo: (ruta) => invoke("unirse_grupo", { ruta }),
    async generarEjecutable(nombre, codigo, alProgreso) {
      const dejar = await listen<string>("ejecutable-progreso", (e) => alProgreso(e.payload));
      try {
        return await invoke<string>("generar_ejecutable", { nombre, codigo });
      } finally {
        dejar();
      }
    },
  };
}
