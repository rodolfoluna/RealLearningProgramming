# RealLearningProgramming — Documento de diseño

Dos aplicaciones para enseñar y aprender programación en Python **sin conexión a internet**:

- **LP Alumno**: curso integrado, editor con el pegado bloqueado, consola, pruebas automáticas,
  generación de ejecutables y exportación de avances protegida. Windows, Android y web (PWA).
- **LP Profesor**: importa las entregas, detecta si se modificaron fuera de la app, muestra el
  avance y las estadísticas de cada alumno y permite calificar.

**Nombre corto: LP.** Es el que ve la gente (apps, ventanas, archivos que se descargan). Los
identificadores internos conservan `rlp` por compatibilidad: extensiones `.rlp/.rlpg/.rlpa/.rlpr/
.rlpk`, paquetes `@rlp/*` y crates `rlp-*`, variables `RLP_*`, el identificador de Android y los
contextos de firma. Cambiarlos dejaría sin abrir las entregas y los perfiles que ya existen.

Este documento explica las decisiones, la arquitectura, la seguridad, los formatos y el plan por
fases. El código está organizado igual que aquí se describe. Los manuales de uso están en
[`MANUAL-ALUMNO.md`](MANUAL-ALUMNO.md) y [`MANUAL-PROFESOR.md`](MANUAL-PROFESOR.md).

---

## 1. Requisitos y cambios propuestos

### Requisitos originales

1. App del profesor para revisar entregas y avances.
2. App del alumno que en la primera ejecución pide número de control y nombre, y guarda los datos
   en su propia carpeta (portable).
3. Los avances se exportan para entregarlos al profesor.
4. Otros alumnos no pueden abrir los archivos; el mismo alumno sí, aunque instale la app en otro
   dispositivo.
5. El profesor detecta si un archivo se modificó fuera de la app.
6. Todo funciona sin internet.
7. Curso incluido: bases, condiciones, ciclos, funciones, listas y programas completos, en consola.
8. Generar el ejecutable de cada programa.
9. Pegar texto bloqueado donde se escribe código; estadísticas de copias e intentos de pegar
   visibles para alumno y profesor.
10. Proponer más funciones para el aprendizaje y el mejor stack.
11. (Agregado después) Versión Android de la app del alumno para quien no tiene PC en casa.

### Cambios propuestos (y ya implementados)

| # | Cambio | Por qué |
|---|--------|---------|
| 1 | **Contraseña obligatoria** en el registro, con **código de recuperación** | El nombre y el número de control son públicos entre compañeros: no sirven para proteger archivos. |
| 2 | **Varios perfiles por carpeta** | En los laboratorios varias personas usan la misma PC y la misma carpeta de la app. |
| 3 | **Integridad honesta**: firma + historial + reproducción del código tecla a tecla | Sin internet, cualquier secreto dentro de la app puede extraerlo un experto; la reproducción del historial es la evidencia más difícil de falsificar. |
| 4 | **Grupo firmado por el profesor** (`.rlpg`) | Así las entregas se cifran para el profesor (y coprofesores) y se fijan las políticas del grupo. |
| 5 | **Salidas de ventana** registradas y visibles también para el alumno | Complementa el bloqueo de pegado (copiar de otra ventana y reescribir). |
| 6 | **Ejecutables solo en Windows** | PyInstaller no genera .exe desde Android; en el celular se indica "genéralo en una PC". |
| 7 | Temario con **cadenas** y **try/except** básico | Son necesarios para hacer "programas más completos". |
| 8 | **Stack web + Rust** en lugar de PySide6 | Un solo código para Windows y Android (ver §3). |
| 9 | **Pegar lo propio** como opción del profesor | Mover el propio código dentro del editor es legítimo; por defecto todo pegado se bloquea. |

---

## 2. Arquitectura

```
┌───────── App Alumno: Windows y Android (Tauri 2) · Web (PWA) ─────────┐
│  Interfaz Svelte 5 + TypeScript (@rlp/alumno-ui, igual en las tres)   │
│   ├─ Editor CodeMirror 6 ── bloqueo de pegado, conteo de copias,      │
│   │                         captura de operaciones de edición         │
│   ├─ Worker de Python (Pyodide = CPython en WebAssembly)              │
│   │     input() con SharedArrayBuffer  ó  puente rlpentrada:// (Rust) │
│   └─ Lecciones Markdown con ejemplos ejecutables                      │
│  Rust                                                                 │
│   ├─ rlp-core: cifrado, historial firmado, SQLite, entregas .rlp      │
│   ├─ rlp-puente: entrada síncrona sin SharedArrayBuffer               │
│   └─ PyInstaller desde runtime\ (CPython portátil) → .exe             │
│  Web: rlp-core en WebAssembly (rlp-web) en un Worker + IndexedDB      │
└───────────────────────────────────────────────────────────────────────┘
          │ .rlp (cifrado y firmado)            ▲ .rlpg (grupo firmado)
          ▼                                     │
┌──────────────────────── App Profesor (Tauri 2) ──────────────────────┐
│  Svelte: tablero, detalle, grupos, importación                        │
│  Rust: rlp-core (sin la llave de firma) + SQLite del profesor          │
│  Pyodide: vuelve a correr las pruebas del alumno en un sandbox WASM   │
└───────────────────────────────────────────────────────────────────────┘
```

### Estructura del repositorio

```
crates/rlp-core/        Núcleo en Rust (compartido; se prueba con cargo test)
  src/crypto.rs         AES-256-GCM, Argon2id, X25519+HKDF (envolturas), Ed25519, códigos
  src/llave_app.rs      Llave de firma de la App Alumno (inyectada al compilar) y llaves de confianza
  src/almacen.rs        Perfil del alumno: meta, actividades cifradas, eventos encadenados
  src/deposito.rs       Dónde se guarda: SQLite (nativa) o memoria + diario (web)
  src/eventos.rs        Formato de eventos, cadena de hashes y firma
  src/replay.rs         Reproducción del historial (UTF-16), segmentos, ritmo de escritura
  src/estadisticas.rs   Contadores calculados desde el historial
  src/alumno.rs         Operaciones de la App Alumno (registro, actividades, exportar/importar)
  src/entrega.rs        Formato .rlp
  src/grupo.rs          Formato .rlpg
  src/profesor.rs       Identidad del profesor, creación de grupos
  src/verificacion.rs   Revisión de entregas (semáforo verde/amarillo/rojo)
  src/bd_profesor.rs    SQLite del profesor: grupos, entregas, tablero, calificaciones, CSV
  tests/flujo.rs        Flujo completo y manipulaciones
  tests/web.rs          Perfiles en memoria (versión web) y continuación web ↔ escritorio
crates/rlp-puente/      Protocolo rlpentrada:// (input y sleep sin SharedArrayBuffer)
crates/rlp-web/         Núcleo de la versión web (WebAssembly): llamar(método, json)
apps/alumno/            App Alumno nativa (Windows y Android): cáscara Tauri + backend (src/, src-tauri/)
apps/alumno-web/        App Alumno web (PWA): cáscara web + backend con el núcleo WebAssembly
apps/profesor/          Interfaz (src/) y app Tauri (src-tauri/)
packages/alumno-ui/     Interfaz de la App Alumno, compartida por la nativa y la web
packages/editor/        Extensiones de CodeMirror (pegado, copias, operaciones, menú)
packages/python-worker/ Worker de Pyodide, harness.py (ejecución, pruebas) y errores_es.py
packages/ui-comun/      Tema claro/oscuro, Markdown con resaltado, modal, semáforo
packages/curso/         Tipos del curso y JSON compilado (generado)
packages/nucleo-web/    Worker con el núcleo WebAssembly y los perfiles en IndexedDB
curso/                  Contenido del curso (YAML + Markdown + Python)
scripts/                Compilar/validar curso, copiar Pyodide, runtime, empaquetar, autoprueba
tests/                  Banco de pruebas y pruebas de interfaz (Playwright)
```

---

## 3. Stack y por qué

| Área | Elección | Motivo |
|------|----------|--------|
| Contenedor | **Tauri 2** | Un solo código para Windows y Android; binario pequeño; usa el WebView del sistema. |
| Núcleo | **Rust** (`aes-gcm`, `argon2`, `x25519-dalek`, `ed25519-dalek`, `rusqlite`) | Mismo código en PC y Android; la llave de firma queda en código nativo. |
| Interfaz | **Svelte 5 + TypeScript + Vite** | Sencillo de mantener, reactivo, diseño adaptable (PC y celular). |
| Editor | **CodeMirror 6** | Control total de las vías de pegado; buen soporte en móviles. |
| Python | **Pyodide** (CPython 3.14 en WebAssembly) en un Web Worker | Sin internet, Python real, `input()` funcional, se puede detener cualquier ciclo infinito. |
| Ejecutables | **PyInstaller** en un CPython portátil (`runtime\`) | Genera `.exe` de un solo archivo sin internet. |
| Curso | YAML + Markdown + Python | El profesor lo edita como texto; se valida automáticamente. |
| Pruebas | cargo test, Vitest, Playwright, validación del curso en CPython y Pyodide, autoprueba con las apps reales | Ver §10. |

Alternativas descartadas: **PySide6** (en Android es experimental y no se adapta a celulares);
**Flet** (editor y control de pegado limitados, difícil detener ciclos infinitos en Android);
**app Android nativa aparte** (dos interfaces que mantener).

---

## 4. Seguridad e integridad

### 4.1 Llaves del alumno

Al registrarse se genera una **llave de datos (DEK)** aleatoria de 256 bits. Todo lo que la app
guarda se cifra con ella (AES-256-GCM). La DEK se guarda **envuelta** tres veces:

1. Con la **contraseña** del alumno (Argon2id, 32 MiB, 3 pasadas).
2. Con el **código de recuperación** de 100 bits (`XXXX-XXXX-XXXX-XXXX-XXXX`), que se muestra una
   sola vez.
3. Con la llave pública **X25519** de cada profesor del grupo (ECIES: X25519 + HKDF-SHA256 + AES-GCM).

Consecuencias:

- Otro alumno que copie la carpeta o el archivo `.rlp` **no puede abrir nada** sin la contraseña.
- El mismo alumno abre sus trabajos en otra PC o en su celular con su contraseña (o su código).
- El profesor **siempre** puede abrir las entregas de su grupo.
- Si el alumno olvida la contraseña, la recupera con su código (fase 2: el profesor también podrá
  restablecerla, porque puede abrir la DEK).

### 4.2 Historial firmado

Cada acción relevante es un **evento** (registro, sesión, apertura de actividad, lotes de
operaciones de edición, ejecución, pruebas, copia, intento de pegado, inserción sospechosa, salida
de ventana, pista, respuesta, exportación, importación, ejecutable). Cada instalación
(`dispositivo_id`) lleva su propia cadena:

```
hash_i  = SHA-256(hash_{i-1} ‖ json_i)          (hash_0 = 32 ceros)
firma_i = Ed25519(llave_app, "rlp-evento-v1" ‖ perfil_id ‖ 0 ‖ hash_i)
```

El JSON se guarda y exporta con sus bytes exactos. Las estadísticas **se calculan desde el
historial**, que es la única fuente de verdad.

### 4.3 Historial de edición reproducible

El editor registra cada cambio como `[dt, desde, hasta, insertado, origen]` (posiciones UTF-16,
igual que JavaScript). El núcleo en Rust **aplica** esas operaciones para obtener el código: el
código guardado nunca viene "suelto" de la interfaz, sale del historial. Orígenes: `t` tecleo,
`i` automático (sangría/nueva línea), `d` borrado, `u`/`r` deshacer/rehacer, `p` pegado permitido,
`o` otro.

Cada actividad tiene **segmentos**: empiezan con un evento `base` (`inicio` o `reinicio` con el
código inicial, o `continuacion` con el hash del texto del que se partió al venir de otro
dispositivo) y siguen con las operaciones de ese dispositivo.

### 4.4 Verificación en la App Profesor

| Revisión | Rojo cuando… | Amarillo cuando… |
|----------|--------------|------------------|
| Firma de la app | la firma no corresponde o la llave no es de una versión oficial | se usó la llave de desarrollo |
| Integridad del archivo | el contenido no coincide con el hash del manifiesto | — |
| Descifrado | no está dirigido a este profesor o se alteró el cifrado | — |
| Historial | faltan eventos, un hash no coincide, una firma es inválida | algún evento lo firmó una llave no reconocida |
| Identidad | el nombre/número no coincide con el evento de registro | no hay evento de registro |
| Fechas y horas | — | el reloj retrocedió o hay fechas futuras |
| Continuidad | el historial cambió o se acortó respecto a entregas anteriores | — |
| Historial de escritura | el código entregado **no** se reconstruye tecla a tecla | código inicial distinto al del curso; ráfagas de escritura > 12 caracteres/s |

Además el tablero marca si un mismo perfil aparece con otro número de control (o al revés).

### 4.5 Límites (dicho con honestidad)

- Una app sin conexión se ejecuta en la PC del alumno: alguien con conocimientos de ingeniería
  inversa podría extraer la llave de firma del binario. Por eso la señal principal no es la firma,
  sino la **reproducción del historial de edición** y el **ritmo de escritura**: fabricar un
  historial de tecleo creíble para un programa es mucho más difícil que editar un archivo.
- Nada impide que un alumno **reescriba a mano** código de otra fuente. El historial, las salidas de
  ventana y una breve **defensa oral** del código son los mejores complementos.
- En Android (fase 2) el bloqueo de pegado es menos hermético (teclados de terceros): se bloquea el
  menú y las inserciones grandes o de varias líneas, y todo se registra.
- En la **versión web** cualquiera puede descargar el código, así que su llave de firma no es
  secreta: la App Profesor marca en **amarillo** la firma de esas entregas y el historial escrito
  en la web ("confía en el historial y en la reproducción de la escritura"). El resto de las
  revisiones (descifrado, cadena, reproducción tecla a tecla, fechas, identidad) son las mismas.

### 4.6 Del profesor al alumno: retroalimentación y acceso

- **Retroalimentación** (`.rlpr`): un archivo por grupo. La parte de cada alumno (calificación y
  comentario por actividad) va cifrada con **su** llave de datos, que el profesor abre desde la
  envoltura dirigida a él en la última entrega; el archivo completo va firmado con la llave Ed25519
  del profesor. La App Alumno solo lo acepta si la firma es la del profesor que creó su grupo, no
  acepta una retroalimentación más vieja que la que ya tiene y la guarda cifrada en su perfil.
- **Archivo de acceso** (`.rlpa`): para quien olvidó su contraseña **y** su código de
  recuperación. El profesor envuelve la llave de datos del alumno con una contraseña temporal
  (Argon2id, `XXXX-XXXX-XXXX`) y firma el archivo. El alumno entra con ambos, fija una contraseña
  nueva y recibe un **código de recuperación nuevo**. La app comprueba la firma, que el archivo
  sea de ese alumno, que lo haya firmado el profesor de su grupo y que la llave abra realmente su
  historial; queda registrado como evento `acceso_profesor`. Si el perfil no está en esa
  computadora, se restaura desde su último `.rlp` con el mismo archivo.

### 4.7 Llave de firma de la App Alumno

- Desarrollo: llave pública fija (la App Profesor la acepta en **amarillo**).
- Producción: `cargo run -p rlp-core --example generar_llave_app` genera un par. La semilla se
  queda con quien publica (`scripts/publicar-version.ps1` la pide al compilar en su PC; el secreto
  `RLP_CLAVE_APP` solo lo usan los workflows manuales); la pública va a
  `crates/rlp-core/llaves_app.txt` (o a la variable `RLP_CLAVE_APP_PUBLICA`). La semilla se inyecta
  ofuscada por `build.rs`. La lista conserva las llaves de versiones anteriores para seguir
  verificando sus entregas.
- Al publicar, `cargo run -p rlp-core --example verificar_llave_app` detiene la compilación si
  falta la semilla o si su llave pública no está en la lista: una versión publicada nunca firma
  con la llave de desarrollo.
- La App Profesor se compila **sin** la función `firmar`: no contiene ninguna llave privada. Cada
  app se compila por separado para que Cargo no unifique esa función.
- **Versión web**: llave propia, secreto `RLP_CLAVE_APP_WEB`. Al compilar a WebAssembly,
  `build.rs` lee solo esa variable, así que la semilla nativa nunca entra a un bundle web. Su
  llave pública lleva la marca `[web]` en `llaves_app.txt`. `verificar_llave_app -- web` la
  revisa antes de publicar y también impide usar la llave web en la app nativa, o al revés.

---

## 5. Formatos de archivo

### `alumno.db` (SQLite, una por perfil en `datos/perfiles/<perfil_id>/`)

El núcleo guarda estas filas a través del trait `Deposito` (`deposito.rs`): SQLite en escritorio y
Android; en memoria con un diario de cambios que la versión web guarda en IndexedDB (ver
[`PWA.md`](PWA.md)). El contenido cifrado es el mismo en ambos.

```sql
meta(clave, valor)                         -- JSON en claro: perfil público, dispositivo, envolturas, grupo
actividades(id, datos)                     -- AES-GCM(DEK, AAD="act|perfil|id", EstadoActividad JSON)
eventos(dispositivo, seq, datos, hash, firma, llave)
                                           -- datos = AES-GCM(DEK, AAD="ev|perfil|disp|seq", JSON exacto)
```

### Entrega `.rlp` (zip; nombre `<numcontrol>_<AAAAMMDD-HHMM>.rlp`)

| Entrada | Contenido |
|---------|-----------|
| `manifiesto.json` | formato, versión, `perfil_id`, fecha, app (versión, llave pública, dev), `grupo_id`, envolturas de la DEK, cabezas de cadena por dispositivo, SHA-256 del payload |
| `payload.bin` | AES-GCM(DEK, AAD="rlp-payload-v1\|perfil", deflate(JSON: perfil, grupo, actividades, llaves de la app, eventos)) |
| `firma.sig` | Ed25519 de la app sobre los bytes exactos del manifiesto |

El mismo archivo sirve para **entregar** y para **continuar en otro dispositivo**: al importarlo se
insertan las cadenas de los otros dispositivos (verificadas) y, por actividad, gana la versión más
reciente.

### Grupo `.rlpg`

`{"contenido": "<JSON exacto>", "firma": "<Ed25519 del profesor>"}` con: nombre, materia, periodo,
profesor, llaves X25519 de los profesores, llave Ed25519 de firma, políticas
(`pegado: bloquear|propio`, `registrar_salidas`) y la expresión regular del número de control.

### Respaldo de llaves del profesor `.rlpk`

JSON con las llaves públicas y las privadas cifradas (AES-GCM con una llave envuelta con
Argon2id(contraseña del profesor)).

### Retroalimentación `.rlpr` y acceso `.rlpa`

Ambos: `{"contenido": "<JSON exacto>", "firma": "<Ed25519 del profesor>"}`.
- `.rlpr`: `formato`, `grupo_id`, `profesor`, `llave_firma`, `creado` y `alumnos`
  (`perfil_id` → JSON `{profesor, creado, actividades: {id: {calificacion, comentario}}}` cifrado con
  AES-GCM y la llave de datos del alumno, AAD `retro|<perfil_id>`).
- `.rlpa`: `formato`, `perfil_id`, `numero_control`, `nombre`, `profesor`, `llave_firma`, `creado` y
  la llave de datos envuelta con la contraseña temporal (AAD `dek|<perfil_id>`).

### Base del profesor `profesor.db`

`grupos`, `entregas` (manifiesto, reporte, estadísticas y contenido descifrado), `calificaciones`.

---

## 6. App Alumno

- **Portable**: la app, `runtime\`, `config\grupo.rlpg`, `datos\` y `mis_ejecutables\` viven en la
  misma carpeta (USB, Documentos). Si la carpeta no permite escribir, la app lo avisa.
- **Inicio**: perfiles existentes → contraseña (o código de recuperación + contraseña nueva);
  "Soy nuevo" → registro con código de recuperación; "Tengo mis avances en un archivo" → restaurar
  desde `.rlp`. Sin grupo: importar el `.rlpg` o practicar.
- **Temario** con avance por unidad; **lecciones** en Markdown con botón "Probar" en cada ejemplo.
- **Actividades**:
  - *Código*: editor (resaltado, sangría automática, pareo de paréntesis, revisión de sintaxis en
    vivo con mensaje en español, zoom), consola con `input()`, **Probar** con pruebas automáticas y
    diferencias explicadas, errores en español con "ir a la línea", pistas escalonadas, reiniciar y
    **crear .exe**.
  - *Predicción*: escribir la salida de un programa antes de ejecutarlo.
  - *Opción múltiple* con explicación por opción.
- **Bloqueo de pegado** en el editor (y en las respuestas de predicción): evento `paste`, `drop`,
  `beforeinput` y filtro de transacciones; menú contextual propio sin "Pegar"; inserciones de más de
  3 caracteres visibles en una sola pulsación se rechazan (autoescritores, portapapeles de teclados).
  Política "propio": permite pegar solo lo copiado del mismo editor (comparando hashes).
- **Contadores en vivo** (copias, intentos de pegar, salidas) y **Mis estadísticas** (tiempo,
  ejecuciones, errores, pruebas, pistas, teclas…), las mismas que ve el profesor.
- **Exportar entrega**, **importar avances** de otro equipo, **cambiar contraseña**, **unirse a un
  grupo**, tema claro/oscuro.

### Android (APK de la App Alumno)

- Mismo código que en Windows (Tauri 2). Los datos viven en la carpeta privada de la app
  (`/data/user/0/mx.reallearningprogramming.alumno`); **desinstalar la app los borra**, así que el
  alumno debe exportar su entrega con frecuencia (también le sirve de respaldo).
- **Archivos**: el selector de Android devuelve URIs `content://`; los comandos de Rust los abren
  con el plugin fs (`archivos.rs`). Exportar usa "Guardar como" (Descargas, Drive…), porque en
  Android no se puede elegir una carpeta.
- **Unirse al grupo por QR**: la App Profesor muestra el `.rlpg` firmado como QR
  (`RLPG1:` + deflate + base64 URL, ~1 KB); la App Alumno lo escanea con la cámara
  (plugin barcode-scanner) y verifica la firma igual que con el archivo.
- **Barra de teclas de código** en pantallas táctiles (Tab, `:`, paréntesis, corchetes, comillas,
  operadores, flechas, deshacer): inserta como tecleo normal, así cuenta en el historial.
- Cambiar de app (`visibilitychange`) cuenta como salida de la ventana.
- Sin `.exe` (no hay PyInstaller en el celular).
- **Compilación local** (`scripts/compilar-android.ps1`, Windows): toma el SDK de `ANDROID_HOME`
  o `E:\Android`, el NDK más nuevo y el Java de Android Studio; agrega los targets de Rust, genera
  el proyecto con `tauri android init` si falta, compila arm64/armv7 **optimizado** y lo firma con
  la llave de depuración de esa PC (`%USERPROFILE%\.android\debug.keystore`, la misma en cada
  compilación) o con tu keystore (`-Keystore`/`-Alias`). Deja el APK en `dist-android\`
  (`-Instalar` lo instala con adb; `-Depuracion` hace el build sin optimizar, para un emulador).
- **CI** (`build-android.yml`, solo a mano, como respaldo): genera el proyecto con
  `tauri android init`, compila un APK de depuración x86_64 que se instala en un **emulador** y
  corre la autoprueba (activada con `autoprueba.txt` en la carpeta privada vía `adb shell run-as`;
  el resultado queda en `autoprueba_resultado.json`), y el APK para celulares (arm64 y armv7),
  firmado con el keystore de los secretos `ANDROID_KEYSTORE`, `ANDROID_KEYSTORE_PASSWORD` y
  `ANDROID_KEY_ALIAS` (sin ellos, uno de depuración con firma distinta en cada corrida). La firma
  debe ser siempre la misma: Android solo actualiza una app (conservando sus datos) si coincide.

### Versión web (PWA)

- La misma interfaz (`packages/alumno-ui`) con otra cáscara (`apps/alumno-web`): el núcleo
  `rlp-core` compilado a WebAssembly corre en un Worker y los perfiles se guardan cifrados en
  IndexedDB, con las mismas filas que el `alumno.db`. Un perfil se abre en una sola pestaña a la
  vez (Web Locks).
- Instalable y sin conexión: el service worker guarda la app y Pyodide, y agrega COOP/COEP para
  que `input()` funcione en cualquier hosting estático.
- Archivos con el selector del navegador; la entrega se descarga; QR con la cámara; sin `.exe`.
  Los datos viven en el navegador: la app recuerda exportar la entrega como respaldo.
- Detalle, decisiones y resultados de las pruebas en [`PWA.md`](PWA.md).

### Ejecución de Python

- Pyodide corre en un Web Worker. Con `SharedArrayBuffer` (WebView2 en Windows), `input()` espera
  con `Atomics.wait` y **Detener** lanza `KeyboardInterrupt`; si no responde, se termina el worker.
- Sin `SharedArrayBuffer` (WebKitGTK, algunos Android), `input()` y `time.sleep()` hacen una
  petición síncrona al puente `rlpentrada://` servido por Rust, y Detener termina el worker.
- Las pruebas se corren **una por una** con límite de tiempo: un ciclo infinito no impide revisar
  las demás. Durante las pruebas `time.sleep` no espera.
- `os.system("cls")` limpia la consola; el entorno se restaura entre ejecuciones.

---

## 7. App Profesor

- **Llaves**: se crean con contraseña (≥ 10 caracteres) y se exige guardar un **respaldo** (`.rlpk`).
  Se pueden restaurar en otra PC.
- **Grupos**: nombre, materia, periodo, formato del número de control, política de pegado, registro
  de salidas y llaves de coprofesores. Se exporta el `.rlpg` o se instala directamente en una
  carpeta de la App Alumno (`config\grupo.rlpg`).
- **Importar**: archivos sueltos o una carpeta completa (p. ej. la USB con todas las entregas).
  Las repetidas se ignoran; se guarda el historial de entregas de cada alumno.
- **Tablero**: alumnos con semáforo de integridad, avance, puntos, tiempo, ejecuciones, copias,
  intentos de pegar, salidas, pistas; **mapa de actividades**; búsqueda; exportar a **Excel**
  (hojas Resumen, Actividades y Calificaciones con los comentarios como notas) o CSV; y
  **retroalimentación** `.rlpr` para todo el grupo.
- **Detalle**: reporte de verificación y ritmo de escritura; por actividad: estado, estadísticas,
  código (solo lectura), **volver a correr las pruebas** (en Pyodide: el código del alumno no toca
  el disco del profesor), enunciado y solución de referencia, calificación y comentario.
- **Reproductor de escritura**: "Ver cómo lo escribió" vuelve a escribir el código tecla a tecla
  desde el historial firmado (también cuando el alumno continuó en otro equipo), con velocidad
  hasta 100×, "saltar pausas" y una línea de tiempo con marcas de intentos de pegar, copias,
  salidas de la ventana, ejecuciones, pruebas, pistas y reinicios. Al final comprueba que el
  resultado coincida con el código entregado.
- **Archivo de acceso** `.rlpa` para el alumno que olvidó su contraseña y su código (ver 4.6).

---

## 8. Curso

El curso vive en `curso/`: cada unidad es una carpeta con `unidad.yaml`; cada lección, una carpeta
con `leccion.md` y `actividades.yaml`. `pnpm curso` lo compila (la App Alumno recibe una versión
**sin soluciones**). `scripts/validar_curso.py` y `scripts/validar-curso-pyodide.mjs` comprueban que
cada solución pasa sus pruebas, que el código inicial no, y que cada predicción es correcta.

Tipos de prueba (`pruebas:` de una actividad de código):

```yaml
- entrada: "3\n4\n"           # datos para input()
  salida: ["La suma es 7"]     # fragmentos en orden (modo "contiene", predeterminado)
  modo: contiene | exacta | normalizada | regex | termina
- funcion: es_par              # llama una función del alumno
  args: [4]
  kwargs: {base: 10}           # argumentos con nombre (opcional)
  esperado: true               # valor devuelto (se revisa si está, o si no hay "salida")
  entrada: "17\n"              # datos para los input() dentro de la función (opcional)
  salida: ["Hola, Ana"]        # lo que la función debe mostrar (opcional; usa "modo")
  oculta: true                 # no se muestran los datos si falla
```

En las pruebas de función, el programa principal se ejecuta hasta su primer `input()` y ahí se
detiene sin error: así se pueden probar por separado las funciones de un programa completo (con
menú) siempre que estén definidas antes del programa principal, que es la estructura que enseña
el curso.

### Incluido en esta versión (8 unidades, 42 lecciones, 118 actividades; curso 1.1)

| Unidad | Lecciones |
|--------|-----------|
| 0 Introducción | Tu primer programa (print, comentarios, errores) |
| 1 Fundamentos | Variables y tipos · Operadores · input() y conversión · Cadenas y f-strings · math y redondeo |
| 2 Condiciones | Comparaciones · if/else · elif · and/or/not y anidadas · try/except |
| 3 Ciclos | while · Contadores y acumuladores · Centinela · for y range · break/continue · Anidados y figuras · Programas con menú (calculadora, cajero) |
| 4 Funciones | def y llamada · Parámetros · return (print contra return) · Valores por defecto y argumentos con nombre · Alcance (UnboundLocalError) · Descomponer un programa |
| 5 Cadenas | Recorrer (vocales, invertir, palíndromos) · Métodos (split/join, title, count…) · Validaciones (número de control, contraseña segura, correo) · Formato de tablas y cifrado César |
| 6 Listas | Crear e indexar · Métodos · Recorrer y acumular (máximo sin max) · Buscar y filtrar · Tuplas, rebanadas y comprensiones · Matrices |
| 7 Proyectos | Control de calificaciones · Inventario · Ahorcado (2 partes) · Gato (2 partes) · Agenda · Piedra, papel o tijera · Conversor decimal/binario · Punto de venta |

Cada proyecto se califica por partes (cada función con sus pruebas) y como programa completo.
Los diccionarios quedan como posible unidad opcional en una versión posterior.

---

## 9. Plan por fases

- **Fase 1 (hecha, Windows)**: núcleo, App Alumno, App Profesor, curso U0–U3, CI y empaquetado
  portable.
- **Fase 2 (en curso)**: unidades 4–7 del curso (**hecho**: funciones, cadenas, listas y
  proyectos); versión publicable (llave de firma de producción, Releases); **reproductor visual
  del historial**, **retroalimentación firmada** profesor → alumno, **archivo de acceso** y
  exportar a **Excel** (**hechos**); **APK Android** (barra de teclas de código, unirse al grupo por
  **QR**, exportar con "Guardar como", pausa de la app como salida, autoprueba en emulador;
  **hecho**, pendiente afinar las heurísticas de IME con teclados reales). Después: problemas de
  Parsons, historial de versiones, consola interactiva.
- **Versión web (PWA) de la App Alumno** (**hecha**; falta probarla en iPhone y Android reales):
  misma interfaz y núcleo en WebAssembly, instalable y sin conexión. Detalle en [`PWA.md`](PWA.md).
- **Fase 3**: **detección de similitud** entre alumnos (huellas de tokens/AST); visualizador paso a
  paso (tipo Python Tutor); editor del curso y paquetes `.curso` firmados; insignias y rachas;
  tablero de dificultades por actividad.

---

## 10. Pruebas

| Nivel | Qué cubre | Cómo |
|-------|-----------|------|
| Núcleo (Rust) | cifrado, contraseñas, recuperación, envolturas, firmas, UTF-16 (propiedades), flujo completo, continuar en otro dispositivo, byte alterado, manifiesto editado, re-cifrado con otra llave, código cambiado sin historial, eventos borrados, otro profesor | `cargo test -p rlp-core` |
| Editor (TS) | filtro de pegado, inserciones sospechosas, operaciones reproducibles | `pnpm vitest run` |
| Navegador | Pyodide con `input()`, detener, `time.sleep`, errores en español, pruebas con ciclo infinito; pegado por teclado, menú, evento y arrastre; política "propio"; flujos de interfaz de ambas apps | `pnpm exec playwright test` |
| Curso | 118 actividades: soluciones, códigos iniciales y predicciones en CPython y Pyodide; pruebas del arnés | `python3 scripts/validar_curso.py`, `node scripts/validar-curso-pyodide.mjs`, `python3 -m unittest discover packages/python-worker/pruebas` |
| Apps reales | profesor → alumno → profesor con Tauri, WebView y núcleo reales | `scripts/autoprueba.sh` (Linux/Xvfb), `scripts/autoprueba.ps1` (Windows/WebView2) |
| Windows | compilación, runtime con PyInstaller que genera un `.exe` funcional, empaquetado | `scripts/compilar-windows.ps1 -Autoprueba` (o el workflow manual `build-windows.yml`) |

---

## 11. Distribución

- Las versiones se compilan y publican **desde la PC de quien administra**:
  `scripts/publicar-version.ps1` compila Windows (`compilar-windows.ps1`) y Android
  (`compilar-android.ps1`), crea la etiqueta `v*` y sube los zips, el APK y las notas de
  `CHANGELOG.md` al **Release** de GitHub (el zip del profesor incluye `INSTALACION.md` y los
  manuales). Con la etiqueta, el workflow **Build Web** publica la versión web y agrega su zip.
  En GitHub corren solo **CI** (pruebas) y **Build Web**; Build Windows y Build Android quedan
  como respaldo manual.
- Guía completa para un laboratorio (WebView2 sin internet, antivirus, equipos que se restauran al
  reiniciar, entregas, actualizar sin perder datos): [`docs/INSTALACION.md`](INSTALACION.md).
- Resumen: el profesor descomprime `LP-Profesor`, crea sus llaves, **guarda el respaldo** y crea
  el grupo; con "Instalar en carpeta de la App Alumno" coloca el grupo en la carpeta `LP-Alumno`
  y la copia a las PCs o memorias USB (o entrega el `.rlpg` para que cada alumno lo importe).
