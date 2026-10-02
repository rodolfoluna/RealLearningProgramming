# RealLearningProgramming

Apps **sin conexión** para aprender y enseñar programación en Python, en español:

- **LP Alumno**: curso con 42 lecciones y 118 actividades (fundamentos, condiciones, ciclos,
  funciones, cadenas, listas y 8 proyectos integradores), editor con el pegado bloqueado, consola con `input()`, pruebas automáticas,
  errores explicados en español, pistas, generación de `.exe` y entregas cifradas.
- **LP Profesor**: importa entregas, verifica que no se hayan modificado fuera de la app
  (reconstruye el código tecla a tecla), muestra avance y estadísticas (copias, intentos de pegar,
  salidas de ventana), **reproduce cómo se escribió** cada código, vuelve a correr las pruebas,
  exporta a Excel y envía **retroalimentación firmada** a los alumnos.

Windows (ambas apps), Android (App Alumno, APK) y **web** (App Alumno instalable desde el
navegador, también sin conexión). Las tres versiones de la App Alumno comparten la interfaz y el
núcleo, y sus entregas son las mismas. El diseño completo está en [`docs/DISENO.md`](docs/DISENO.md);
la versión web, en [`docs/PWA.md`](docs/PWA.md).

## Stack

Tauri 2 · Rust (`rlp-core`, también compilado a WebAssembly) · Svelte 5 + TypeScript ·
CodeMirror 6 · Pyodide (CPython 3.14 en WebAssembly) · PyInstaller · SQLite · IndexedDB y
service worker (versión web).

## Desarrollo

Requisitos: Node 22 + pnpm 10, Rust estable, Python 3 (para validar el curso). En Linux, además,
`libwebkit2gtk-4.1-dev` para compilar las apps.

```bash
pnpm install
pnpm preparar                 # compila el curso y copia Pyodide
pnpm dev:alumno               # interfaz en el navegador con núcleo simulado (http://localhost:1420)
pnpm dev:alumno-web           # versión web con el núcleo real en WebAssembly (http://localhost:1422)
pnpm dev:profesor             # http://localhost:1421 (contraseña de demo: profesor1234)
pnpm --filter @rlp/alumno tauri dev   # app real
```

La versión web compila el núcleo a WebAssembly con `pnpm wasm`. Requiere, una sola vez,
`rustup target add wasm32-unknown-unknown` y `cargo install wasm-bindgen-cli --version 0.2.129
--locked` (la versión de `wasm-bindgen` en `Cargo.lock`).

## Pruebas

```bash
cargo test -p rlp-core                 # núcleo: cifrado, historial, entregas, manipulaciones
cargo test -p rlp-web                  # núcleo de la versión web (perfiles en memoria + diario)
pnpm vitest run                        # lógica del editor
python3 scripts/validar_curso.py       # soluciones del curso en CPython
node scripts/validar-curso-pyodide.mjs # ... y en Pyodide
pnpm exec playwright test              # interfaz y Pyodide en Chromium (versión web: requiere
                                       # pnpm wasm y, para la prueba sin conexión, su build)
./scripts/autoprueba.sh                # apps reales: profesor → alumno → profesor (Xvfb)
```

## Curso

El contenido está en `curso/` (Markdown + YAML + Python). Para agregar una actividad, edita el
`actividades.yaml` de la lección, ejecuta `pnpm curso` y valida con `python3 scripts/validar_curso.py`.

## Compilar y publicar

Las apps de Windows y el APK de Android se compilan **en la computadora de quien publica**
(Windows). En GitHub solo corren **CI** (pruebas en cada push y PR) y **Build Web** (la versión
web); Build Windows y Build Android quedan como respaldo manual (Actions → Run workflow).

```powershell
.\scripts\compilar-windows.ps1                    # zips portables en dist-portable\ (-Autoprueba: flujo completo)
.\scripts\compilar-android.ps1                    # APK optimizado en dist-android\
.\scripts\publicar-version.ps1                    # compila ambos, crea la etiqueta y sube el Release
```

Para publicar una versión:

1. Anota los cambios en `CHANGELOG.md` (`## [X.Y.Z] — AAAA-MM-DD`) y sube la versión en
   `Cargo.toml`, los `package.json` (apps y `packages/alumno-ui`) y los `tauri.conf.json`. Fusiona
   en `main`.
2. En tu PC: `git checkout main`, `git pull` y `.\scripts\publicar-version.ps1`. Pide la semilla de
   la llave de firma (`RLP_CLAVE_APP`, no se muestra), la verifica, compila Windows y Android, crea
   la etiqueta `vX.Y.Z` y sube los zips, el APK y las notas del `CHANGELOG.md` al **Release**
   (requiere [GitHub CLI](https://cli.github.com/): `winget install GitHub.cli`, `gh auth login`).
3. Con la etiqueta, **Build Web** verifica la llave web, agrega `LP-Alumno-*-web.zip` al Release y
   publica el sitio en GitHub Pages.

Las llaves de firma (la nativa, que se queda en tu gestor de contraseñas, y la web, que va en el
secreto `RLP_CLAVE_APP_WEB`), cómo cambiarlas y la configuración de GitHub Pages están en el
[manual del profesor](docs/MANUAL-PROFESOR.md). Sus llaves públicas están en
`crates/rlp-core/llaves_app.txt`.

La guía para instalar en un laboratorio está en [`docs/INSTALACION.md`](docs/INSTALACION.md).
Manuales de uso: [alumno](docs/MANUAL-ALUMNO.md) (incluye cómo pasar los avances entre Windows,
Android y la web) y [profesor](docs/MANUAL-PROFESOR.md) (llaves del profesor, claves de firma,
Android, GitHub Pages y cómo publicar una versión).

### Versión web

El workflow **Build Web** genera el sitio (`LP-Alumno-*-web.zip`, archivos estáticos para
cualquier hosting) en cada push a `main`. Para publicarlo con cada etiqueta `v*`:

1. El secreto `RLP_CLAVE_APP_WEB` con la semilla de la llave web (su llave pública está en
   `llaves_app.txt` con la marca `[web]`).
2. Para GitHub Pages: Settings → Pages → Source: **GitHub Actions**; la variable del repositorio
   `RLP_PAGES` = `1`; y en Settings → Environments → **github-pages**, una regla que permita las
   etiquetas `v*` (por defecto solo deja publicar desde `main`).

### Windows y Android en tu computadora

Requisitos (Windows): Rust con las herramientas de C++ de Visual Studio, Node 22 y pnpm (`pnpm
install` hecho). Para Android, además, el SDK con **NDK**, **Build-Tools** y **Platform-Tools**
(desde el SDK Manager de Android Studio) y Java 17 (el de Android Studio sirve); el script toma el
SDK de `ANDROID_HOME` o, si no está definida, de `E:\Android`.

```powershell
.\scripts\compilar-android.ps1                    # APK optimizado (arm64 y armv7) en dist-android\
.\scripts\compilar-android.ps1 -Instalar          # ... y lo instala con adb en el celular conectado
.\scripts\compilar-android.ps1 -Depuracion -Targets x86_64   # sin optimizar, para un emulador
.\scripts\compilar-android.ps1 -Keystore C:\llaves\lp-alumno.jks -Alias lp   # con tu keystore
```

Sin `-Keystore`, el APK se firma con la llave de depuración de tu PC
(`%USERPROFILE%\.android\debug.keystore`), que no cambia entre compilaciones: las versiones nuevas
se instalan encima y conservan los datos mientras compiles en la misma PC. Respalda ese archivo;
con otra llave, para actualizar hay que desinstalar (se borran los datos de la app). Sin
`RLP_CLAVE_APP`, las entregas salen con la firma de desarrollo (amarillo en LP Profesor).
