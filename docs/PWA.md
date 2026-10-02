# Versión web (PWA) de LP Alumno

La App Alumno tiene, junto a la app nativa (Windows y Android), una versión web instalable
(`apps/alumno-web`). Las tres comparten la interfaz (`packages/alumno-ui`). El núcleo Rust (`rlp-core`) se compila a WebAssembly para que el cifrado, el
historial firmado y los archivos `.rlp/.rlpg/.rlpa/.rlpr` sean idénticos y la App Profesor los abra
igual. Fuera de alcance: crear `.exe`, PWA del profesor, app nativa de iOS y servidor propio.

| Fase | Estado |
|---|---|
| 0. Pruebas de riesgo | **Hecha en Chromium**; falta confirmar en iPhone/iPad y Chrome Android |
| 1. Núcleo portable | **Hecha** |
| 2. Crate `rlp-web` + paquete `nucleo-web` | **Hecha** |
| 3. Interfaz compartida y `backend-web.ts` | **Hecha** |
| 4. Cáscara PWA (manifest, service worker, instalación) | **Hecha** |
| 5. Llave web → Amarillo; pruebas cruzadas con la App Profesor | **Hecha** |
| 6. CI, Playwright (Chromium + WebKit) y documentación | **Hecha** |

## Fase 0 — resultados

**0.1 Pyodide e `input()` sin encabezados del servidor.** La app compilada se sirvió como sitio
estático **sin** COOP/COEP, con un service worker que agrega esos encabezados a cada respuesta y
una recarga inicial (patrón *coi-serviceworker*). En Chromium:

| | `crossOriginIsolated` | `input()` en la consola |
|---|---|---|
| Con el service worker | sí | funciona; Python listo ~2.9 s después de "Ejecutar" (primera vez) |
| Sin service worker | no | **no**: el programa se queda esperando (modo "limitado") |

Decisión: no hace falta un puente de `input()` propio en el service worker; se reutiliza el modo
"memoria" (SharedArrayBuffer) del escritorio y el sitio puede estar en cualquier hosting, incluido
GitHub Pages. Si en Safari falla, se usa el modo "puente" ya existente con una URL que atienda el
service worker. Peso de Pyodide: 13 MB (≈6 MB con gzip): `pyodide.asm.wasm` 9.2 MB,
`python_stdlib.zip` 2.5 MB, `pyodide.asm.mjs` 1.2 MB; se descarga al instalar.

**0.2 Núcleo en WebAssembly.** `rlp-core` sin SQLite compila a `wasm32-unknown-unknown`. Un
módulo de prueba con wasm-bindgen pesa 1.5 MB (0.52 MB con gzip). En un Worker de Chromium (y en
Node), en el servidor de pruebas:

| Operación | Tiempo |
|---|---|
| Argon2id estándar (32 MiB, t=3) | 100–140 ms |
| Argon2id 19 MiB, t=2 | ~40 ms |
| Registro (2 derivaciones) + 25 ediciones + exportar `.rlp` | 230–280 ms |

Un celular de gama media es de 3 a 6 veces más lento: ~0.5–0.8 s por derivación. Se conservan
los parámetros estándar (los mismos de la app nativa); falta medir en un iPhone real.

**0.3 Persistencia.** Se usará IndexedDB con `navigator.storage.persist()`. Safari puede borrar los
datos de un sitio **no instalado** tras 7 días sin uso; la PWA agregada a la pantalla de inicio
queda exenta. Por eso: guía de instalación, recordatorios de exportar la entrega (también es el
respaldo) y aviso de que borrar los datos del sitio borra el perfil. Falta probarlo en Safari.

## Fase 1 — núcleo portable (hecha)

- `crates/rlp-core/src/deposito.rs`: el trait `Deposito` guarda filas ya cifradas (meta,
  actividades, eventos) y escribe cada `Lote` completo o nada.
  - `DepositoSqlite` (función `sqlite`, por omisión): el `alumno.db` de siempre, mismo esquema.
  - `DepositoMemoria`: para la web. Lleva un **diario** de cambios (`tomar_diario()`) que la app
    guarda en IndexedDB después de cada operación, y se vuelve a cargar con una **instantánea**
    (`DepositoMemoria::desde`). Los lotes viajan como JSON con los bytes en base64.
- `almacen.rs` trabaja sobre cualquier `Deposito`; `alumno.rs` sobre el trait `Perfiles`:
  `CarpetaPerfiles` (carpetas con SQLite) o `PerfilesEnMemoria` (la app web le pasa los perfiles
  que tiene). Nuevas: `SesionAlumno::registrar_en`, `abrir_en`, `restaurar_en`, `ubicacion()` y
  `tomar_diario()`; `cerrar()` devuelve el último diario. Las funciones con rutas
  (`registrar`, `abrir`, `restaurar`, `listar_perfiles`, `carpeta()`) siguen igual para Tauri.
- Sin SQLite: `bd_profesor`, `excel` y `retroalimentacion::crear_desde_bd` quedan tras la función
  `sqlite` (la App Profesor la pide explícitamente).
- WebAssembly: reloj con `Date.now()` y azar con `crypto.getRandomValues` (getrandom `js`, uuid
  `js`, chrono `wasmbind`), como dependencias solo para `wasm32-unknown-unknown`: no hace falta una
  función `web`.
- Formatos sin cambios: las 16 pruebas de `tests/flujo.rs` pasan igual. `tests/web.rs` prueba el
  perfil web (diario, reapertura, diario perdido) y la continuación web ↔ escritorio verificada por
  el profesor.
- CI: pruebas del núcleo sin SQLite y clippy para `wasm32-unknown-unknown`.

## Fase 2 — núcleo web (hecha)

- **`crates/rlp-web`**: una sola función, `Nucleo.llamar(metodo, args_json) → json`, con las mismas
  operaciones que los comandos de la app nativa (`registrar`, `iniciar_sesion`, `restaurar`,
  `entrar_con_acceso`, `abrir_actividad`, `guardar_edicion`, `exportar`, `importar_avances`…),
  pero con archivos en base64 en lugar de rutas. `diario` devuelve los cambios de cada perfil
  (también los de una sesión recién cerrada). Se prueba en Rust nativo (`tests/nucleo.rs`).
- **Llave web**: al compilar a WebAssembly, `rlp-core/build.rs` lee solo `RLP_CLAVE_APP_WEB`;
  la semilla nativa nunca entra a la versión web. En `llaves_app.txt`, la marca `[web]` identifica
  su llave pública (`LlaveConocida.web`).
- **`pnpm wasm`** (`scripts/compilar-wasm.mjs`): compila `rlp-web` y genera
  `packages/nucleo-web/generado/` (1.7 MB; 0.6 MB con gzip). Comprueba el target de Rust y que
  `wasm-bindgen-cli` sea la versión del `Cargo.lock`. No forma parte de `pnpm preparar`: Windows y
  Android no lo necesitan.
- **`@rlp/nucleo-web`**: `crearNucleoWeb()` devuelve un cliente cuyos métodos corren en un Worker
  (`worker.ts`) con el núcleo y la base `rlp-alumno` de IndexedDB (`idb.ts`). `servicio.ts`:
  - guarda el diario **antes** de responder; si IndexedDB falla, deja de aceptar cambios y pide
    recargar (lo guardado sigue siendo consistente);
  - abre cada perfil en **una sola pestaña** (Web Locks): resuelve el riesgo 1;
  - atiende las llamadas en orden y guarda el grupo instalado del navegador.
- Pruebas: vitest con el wasm real y `fake-indexeddb` (registro, recarga, candado, contraseña
  incorrecta, continuar en otro navegador) y Playwright en Chromium (`banco.nucleo.spec.ts`:
  Worker e IndexedDB reales, recarga y segunda pestaña bloqueada).

## Fase 3 — una interfaz, dos cáscaras (hecha)

- **`packages/alumno-ui`**: toda la interfaz de la App Alumno (componentes, estado, simulación del
  núcleo). Se arranca con `iniciarApp({ backend, puenteEntrada?, alIniciar?, autoprueba? })`; ya
  no importa nada de Tauri.
- **`apps/alumno`** (Windows y Android) quedó como cáscara: `main.ts` conecta los comandos de Tauri,
  el puente `rlpentrada://` y el cierre de la ventana; `backend-tauri.ts` y `autoprueba.ts` siguen
  ahí. Sin cambios de comportamiento: Playwright, la autoprueba de escritorio y (en local) el APK.
- **`apps/alumno-web`**: `main.ts` + `backend-web.ts`, que implementa la misma interfaz `Backend`
  con `@rlp/nucleo-web`:
  - archivos con el selector del navegador (sin filtro de extensión en iPhone/iPad, que bloquea
    las extensiones propias);
  - exportar **descarga** el `.rlp` con el nombre que pone el núcleo;
  - QR del grupo con la cámara (`getUserMedia` + jsQR, porque Safari no tiene BarcodeDetector);
  - sin `.exe`; los textos dicen "este navegador" en lugar de "la carpeta de la app".
- Pyodide se carga con ruta relativa a la base del sitio (`base: "./"`), para servirlo en una
  subcarpeta. Si la página no está aislada, la consola avisa que `input()` no está disponible.
- Pruebas: proyecto `web` de Playwright (`web.flujo.spec.ts`) con el núcleo real: registro,
  `input()`, recarga conservando el perfil y descarga de la entrega.

## Fase 4 — app instalable y sin conexión (hecha)

- `public/manifest.webmanifest` (standalone, español, íconos 192/512 y *maskable* generados del
  ícono de la app) y etiquetas para iOS (`apple-touch-icon`, `apple-mobile-web-app-capable`).
- **Service worker** (`apps/alumno-web/sw.js`, plantilla que `vite.config.ts` completa al
  compilar con la lista de archivos y una versión por contenido):
  - guarda la app al instalarse y Pyodide en segundo plano, en una caché aparte que sobrevive a
    las actualizaciones de la app mientras Pyodide no cambie;
  - agrega **COOP/COEP** a cada respuesta, así que el sitio funciona en cualquier hosting estático
    (GitHub Pages incluido). En la primera visita la página se recarga una sola vez para quedar
    aislada;
  - una versión nueva espera: aparece "Hay una versión nueva de la app" con **Actualizar**, que
    primero guarda lo pendiente del editor.
- `src/pwa.ts`: `navigator.storage.persist()`, aviso "la app ya funciona sin conexión", botón
  **Instalar** (Chrome/Edge/Android) e instrucciones para iPhone/iPad ("Compartir → Agregar a
  pantalla de inicio"), que se pueden cerrar para siempre.
- Recordatorio de exportar la entrega si pasaron 7 días o más desde la última, y aviso en la
  pantalla de inicio de que borrar los datos del sitio borra los avances.
- Prueba `web.pwa.spec.ts`: el sitio compilado servido **sin encabezados y en una subcarpeta**
  (como GitHub Pages) queda aislado, ofrece el manifest y, **sin red**, registra un perfil y corre
  un programa con `input()`.

## Fase 5 — integridad y compatibilidad con la App Profesor (hecha)

- `verificacion.rs`: una entrega firmada con una llave `[web]` sale en **amarillo** en "Firma de
  la app" ("creado con la versión web; su firma no es secreta"). El "Historial" también sale en
  amarillo si incluye eventos escritos en la web, aunque luego se haya exportado desde la app
  nativa: así una llave web extraída no sirve para "lavar" un historial fabricado. El resto de las
  revisiones no cambia. Por ahora no es configurable por grupo.
- Llave web de producción generada: la pública está en `llaves_app.txt` con la marca `[web]` y la
  semilla va al secreto `RLP_CLAVE_APP_WEB`. `verificar_llave_app -- web` la revisa y rechaza
  cruzar las llaves nativa y web.
- Pruebas cruzadas:
  - en Rust (`crates/rlp-web/tests/nucleo.rs`): entrega web verificada por el profesor,
    `.rlpa` (con perfil en el navegador o desde el `.rlp`), `.rlpr` guardada y visible al volver
    a entrar, `.rlpg` por archivo y por QR;
  - en `crates/rlp-core/tests/web.rs`: escritorio → web → escritorio;
  - manual: el wasm compilado con la llave de producción generó una entrega que la App Profesor
    marcó en amarillo solo en firma e historial, con la reconstrucción tecla a tecla en verde.

## Fase 6 — CI, publicación y documentación (hecha)

- `ci.yml`:
  - núcleo sin SQLite y clippy para wasm de `rlp-core` y `rlp-web`; `cargo test -p rlp-web`;
  - `wasm-bindgen-cli` con la versión del `Cargo.lock` y `pnpm wasm`;
  - tipos de `alumno-web` y `nucleo-web`, build del sitio;
  - Playwright con el proyecto `web` en Chromium y también en **WebKit** (`web-webkit`, activado
    con `PLAYWRIGHT_WEBKIT=1`). WebKit prueba el sitio compilado (`vite preview`), que es lo que
    usan los alumnos: bloquea, bajo COEP, los módulos que el servidor de desarrollo de Vite sirve
    al worker de Python.
- `build-web.yml`:
  - en `main`, el sitio como artefacto;
  - en las etiquetas `v*`, verifica la llave web, agrega `LP-Alumno-<versión>-web.zip` al
    Release y lo publica en **GitHub Pages** si la variable del repositorio `RLP_PAGES` vale `1`.
- Documentación: README (desarrollo y publicación), `docs/INSTALACION.md` (versión web para
  alumnos y profesores), `docs/LEEME-alumno.txt`, `CHANGELOG.md` y `docs/DISENO.md`.
- `pnpm dev:alumno-web` arranca la versión web con el núcleo real.

## Riesgos encontrados

1. **Dos pestañas con el mismo perfil** agregarían eventos con los mismos números a la cadena del
   mismo dispositivo y romperían el historial guardado. **Resuelto** en la Fase 2: candado por
   perfil con Web Locks; la segunda pestaña avisa que ya está abierto en otra.
2. **Diario sin guardar** (se cerró la pestaña): la copia guardada queda en un estado anterior pero
   consistente (lo prueba `tests/web.rs`). **Resuelto** en la Fase 2: el Worker responde solo
   cuando IndexedDB confirmó.
3. **Sin service worker** (navegador que lo bloquea): `input()` no funciona. Desde la Fase 3 la
   consola lo dice claramente en lugar de quedarse esperando; con el service worker de la Fase 4
   no pasa en Chromium.

## Pendiente fuera del repositorio

- **Probar en dispositivos reales**: un iPhone/iPad (Safari, instalada en la pantalla de inicio)
  y un Android (Chrome). Ahí se confirman el aislamiento con el service worker, el tiempo de
  Argon2id, la cámara para el QR y que Safari conserve los datos de la app instalada.
- **Secretos y opciones del repositorio**: `RLP_CLAVE_APP_WEB` (semilla de la llave web) y, para
  GitHub Pages, Source "GitHub Actions", la variable `RLP_PAGES=1` y, en el entorno
  `github-pages`, una regla que permita las etiquetas `v*` (por defecto solo deja publicar desde
  `main`).
- Android no cambió de comportamiento, pero su interfaz ahora sale de `packages/alumno-ui`:
  recompilar el APK en local con `scripts/compilar-android.ps1` y probarlo una vez.
