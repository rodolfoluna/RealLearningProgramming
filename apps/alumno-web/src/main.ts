// Versión web (PWA) de la App Alumno: la misma interfaz que la app nativa (@rlp/alumno-ui), con el
// núcleo en WebAssembly y los perfiles en el navegador.
import { iniciarApp } from "@rlp/alumno-ui";

export default iniciarApp({
  backend: () => import("./backend-web").then((m) => m.crearBackendWeb()),
});
