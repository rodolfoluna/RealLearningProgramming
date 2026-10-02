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

El workflow **Build Windows** genera `LP-Alumno-*.zip` y `LP-Profesor-*.zip` (carpetas
portables) en cada push. Para publicar una versión:

1. Configura una sola vez el secreto `RLP_CLAVE_APP` (Settings → Secrets and variables →
   Actions) con la semilla de la llave de firma de la App Alumno; su llave pública ya está en
   `crates/rlp-core/llaves_app.txt`. Para cambiarla: `cargo run -p rlp-core --example
   generar_llave_app` y agrega la nueva llave pública (conserva las anteriores para que las
   entregas viejas se sigan verificando).
2. Anota los cambios en `CHANGELOG.md` y sube la versión en `Cargo.toml`, los `package.json` y
   los `tauri.conf.json` de las apps.
3. Crea la etiqueta sobre `main` (`git tag v0.2.0 && git push origin v0.2.0`). El workflow
   verifica la llave de producción, compila, corre la autoprueba y publica el **Release** con los
   dos zips y las notas del `CHANGELOG.md`.

La guía para instalar en un laboratorio está en [`docs/INSTALACION.md`](docs/INSTALACION.md).
Manuales de uso: [alumno](docs/MANUAL-ALUMNO.md) (incluye cómo pasar los avances entre Windows,
Android y la web) y [profesor](docs/MANUAL-PROFESOR.md) (llaves del profesor, claves de firma,
Android, GitHub Pages y cómo publicar una versión).

### Versión web

El workflow **Build Web** genera el sitio (`LP-Alumno-*-web.zip`, archivos estáticos para
cualquier hosting). Para publicar:

1. Configura una sola vez el secreto `RLP_CLAVE_APP_WEB` con la semilla de la llave web (su
   llave pública ya está en `llaves_app.txt` con la marca `[web]`).
2. Para que se publique en GitHub Pages: Settings → Pages → Source: **GitHub Actions**, y la
   variable del repositorio `RLP_PAGES` = `1`.
3. Con la etiqueta `v*`, el workflow verifica la llave, agrega el `.zip` al Release y publica el
   sitio.

### APK de Android en tu computadora

Requisitos (Windows): el SDK de Android con **NDK**, **Build-Tools** y **Platform-Tools** (desde
el SDK Manager de Android Studio), Java 17 (el de Android Studio sirve) y `pnpm install` hecho.
El script toma el SDK de `ANDROID_HOME` o, si no está definida, de `E:\Android`.

```powershell
.\scripts\compilar-android.ps1                    # APK de depuración (arm64 y armv7) en dist-android\
.\scripts\compilar-android.ps1 -Instalar          # ... y lo instala con adb en el celular conectado
.\scripts\compilar-android.ps1 -Targets x86_64    # para un emulador
$env:RLP_CLAVE_APP = "<semilla>"                  # entregas en verde (la misma del secreto de CI)
.\scripts\compilar-android.ps1 -Release -Keystore C:\llaves\lp-alumno.jks -Alias lp
```

El APK de depuración (el que se usa por ahora) cambia de firma: para pasar a uno firmado con `-Release` hay que
desinstalar la app (se borran sus datos). Usa siempre el mismo keystore para que las
actualizaciones conserven los datos. El workflow **Build Android** ya no corre en cada push: se
lanza a mano (Actions → Build Android → Run workflow) o al crear una etiqueta `v*`, que agrega el
APK al Release (de depuración mientras no haya keystore).
