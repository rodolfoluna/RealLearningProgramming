// App Alumno nativa (Windows y Android, con Tauri). La interfaz es @rlp/alumno-ui; aquí solo se
// conecta con el núcleo en Rust. En un navegador normal (desarrollo y pruebas de interfaz) usa la
// simulación en memoria.
import { iniciarApp, prepararCierre, type Backend } from "@rlp/alumno-ui";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";

const enTauri = "__TAURI_INTERNALS__" in window;

export default iniciarApp({
  backend: (): Promise<Backend> =>
    enTauri
      ? import("./backend-tauri").then((m) => m.crearBackendTauri())
      : import("@rlp/alumno-ui/simulado").then((m) => m.crearBackendSimulado()),
  // Entrada síncrona vía Rust para WebView sin SharedArrayBuffer (p. ej. WebKitGTK).
  puenteEntrada: enTauri
    ? {
        url: convertFileSrc("", "rlpentrada"),
        enviar: (linea) => invoke("entrada_enviar", { linea }),
        cancelar: () => invoke("entrada_cancelar"),
      }
    : undefined,
  // Al cerrar la ventana, la interfaz guarda lo pendiente y luego Rust cierra la sesión.
  alIniciar: enTauri
    ? async () => {
        const { listen } = await import("@tauri-apps/api/event");
        void listen("cerrando", async () => {
          await prepararCierre();
          await invoke("cerrar_app");
        });
      }
    : undefined,
  autoprueba: (fase) => import("./autoprueba").then((m) => m.autoprueba(fase)),
});
