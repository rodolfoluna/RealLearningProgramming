// Versión web (PWA) de la App Alumno: la misma interfaz que la app nativa (@rlp/alumno-ui), con el
// núcleo en WebAssembly y los perfiles en el navegador.
import { iniciarApp } from "@rlp/alumno-ui";
import { prepararPwa } from "./pwa";

// En la primera visita la página se recarga una vez a través del service worker (ver pwa.ts).
if (await prepararPwa()) {
  iniciarApp({
    backend: () => import("./backend-web").then((m) => m.crearBackendWeb()),
  });
}
