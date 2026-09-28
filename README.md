# RealLearningProgramming

Apps **sin conexión** para aprender y enseñar programación en Python, en español:

- **RLP Alumno**: curso con 42 lecciones y 118 actividades (fundamentos, condiciones, ciclos,
  funciones, cadenas, listas y 8 proyectos integradores), editor con el pegado bloqueado, consola con `input()`, pruebas automáticas,
  errores explicados en español, pistas, generación de `.exe` y entregas cifradas.
- **RLP Profesor**: importa entregas, verifica que no se hayan modificado fuera de la app
  (reconstruye el código tecla a tecla), muestra avance y estadísticas (copias, intentos de pegar,
  salidas de ventana), **reproduce cómo se escribió** cada código, vuelve a correr las pruebas,
  exporta a Excel y envía **retroalimentación firmada** a los alumnos.

Windows (ambas apps) y Android (App Alumno, APK). El diseño completo está en
[`docs/DISENO.md`](docs/DISENO.md).

## Stack

Tauri 2 · Rust (`rlp-core`) · Svelte 5 + TypeScript · CodeMirror 6 · Pyodide (CPython 3.14 en
WebAssembly) · PyInstaller · SQLite.

## Desarrollo

Requisitos: Node 22 + pnpm 10, Rust estable, Python 3 (para validar el curso). En Linux, además,
`libwebkit2gtk-4.1-dev` para compilar las apps.

```bash
pnpm install
pnpm preparar                 # compila el curso y copia Pyodide
pnpm dev:alumno               # interfaz en el navegador con núcleo simulado (http://localhost:1420)
pnpm dev:profesor             # http://localhost:1421 (contraseña de demo: profesor1234)
pnpm --filter @rlp/alumno tauri dev   # app real
```

## Pruebas

```bash
cargo test -p rlp-core                 # núcleo: cifrado, historial, entregas, manipulaciones
pnpm vitest run                        # lógica del editor
python3 scripts/validar_curso.py       # soluciones del curso en CPython
node scripts/validar-curso-pyodide.mjs # ... y en Pyodide
pnpm exec playwright test              # interfaz y Pyodide en Chromium
./scripts/autoprueba.sh                # apps reales: profesor → alumno → profesor (Xvfb)
```

## Curso

El contenido está en `curso/` (Markdown + YAML + Python). Para agregar una actividad, edita el
`actividades.yaml` de la lección, ejecuta `pnpm curso` y valida con `python3 scripts/validar_curso.py`.

## Compilar y publicar

El workflow **Build Windows** genera `RLP-Alumno-*.zip` y `RLP-Profesor-*.zip` (carpetas
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
