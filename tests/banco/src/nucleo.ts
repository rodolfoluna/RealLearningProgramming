// Expone el núcleo web (Worker + WebAssembly + IndexedDB) a las pruebas de Playwright.
import { crearNucleoWeb } from "@rlp/nucleo-web";

Object.assign(window, { nucleo: crearNucleoWeb() });
