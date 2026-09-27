# RealLearningProgramming

Apps **sin conexión** para aprender y enseñar programación en Python, en español:

- **RLP Alumno**: curso con lecciones y 53 actividades (fundamentos, condiciones, ciclos y
  programas con menú), editor con el pegado bloqueado, consola con `input()`, pruebas automáticas,
  errores explicados en español, pistas, generación de `.exe` y entregas cifradas.
- **RLP Profesor**: importa entregas, verifica que no se hayan modificado fuera de la app
  (reconstruye el código tecla a tecla), muestra avance y estadísticas (copias, intentos de pegar,
  salidas de ventana), vuelve a correr las pruebas y exporta calificaciones a CSV.

Windows ahora; Android (app del alumno) en la fase 2. El diseño completo está en
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

## Compilar para Windows

El workflow **Build Windows** genera `RLP-Alumno-*.zip` y `RLP-Profesor-*.zip` (carpetas
portables). Para producción, genera la llave de firma de la App Alumno con
`cargo run -p rlp-core --example generar_llave_app`, guarda la semilla en el secreto
`RLP_CLAVE_APP` y la llave pública en `crates/rlp-core/llaves_app.txt`.
