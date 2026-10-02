# Manual del profesor: llaves, claves de firma y publicación

LP usa dos tipos de llaves, y conviene no confundirlas:

| | Para qué sirve | Quién la tiene | Dónde se configura |
|---|---|---|---|
| **Tus llaves de profesor** | Abrir las entregas de tus alumnos y firmar tus grupos, retroalimentaciones y archivos de acceso | Solo tú (y tus coprofesores, con las suyas) | En LP Profesor ([sección 1](#1-tus-llaves-de-profesor)) |
| **Llaves de firma de LP Alumno** | Que LP Profesor reconozca que una entrega salió de una versión oficial de la app (el semáforo) | Quien publica las versiones: la nativa en su gestor de contraseñas, la web como secreto de GitHub | En tu PC y en GitHub ([sección 2](#2-llaves-de-firma-de-lp-alumno)) |

Para instalar las apps en un laboratorio, ver [`INSTALACION.md`](INSTALACION.md). Para publicar una
versión, la [sección 4](#4-publicar-una-versión).

---

## 1. Tus llaves de profesor

### Crearlas (una sola vez)
1. Abre **LP Profesor** → **Crear mis llaves de profesor**.
2. Escribe **tu nombre** (lo verán tus alumnos) y una **contraseña de al menos 10 caracteres**.
3. Al terminar aparece **Guarda un respaldo de tus llaves**: toca **💾 Guardar respaldo…** y
   guarda el archivo `.rlpk` en una memoria USB **y** en tu nube. Está protegido con la misma
   contraseña.

> **Si pierdes tus llaves** (se daña la computadora, formateas o borras la carpeta) **no podrás
> abrir las entregas** de tus alumnos. Nadie puede recuperarlas por ti: el respaldo `.rlpk` es la
> única copia.

Las llaves quedan cifradas en la carpeta `datos_profesor`, junto a `LP Profesor.exe`. Copiar esa
carpeta también sirve de respaldo (con tu base de entregas).

### Usarlas en otra computadora
Abre LP Profesor ahí → **Ya tengo un respaldo** (o **Restaurar desde un respaldo**) →
**Elegir respaldo…** → tu `.rlpk` → la contraseña con la que lo guardaste → **Restaurar**.

### Cada día
LP Profesor pide **Contraseña de tus llaves** → **Desbloquear**. Al terminar, **🔒 Bloquear**.

### Coprofesores
En **Grupos** aparece **tu llave pública**: compártela con otro profesor para que te agregue a sus
grupos. Al crear un grupo, en **Llaves de coprofesores**, pega las llaves públicas de quienes deben
poder abrir esas entregas. La llave pública no es secreta; tu contraseña y tu `.rlpk`, sí.

### Lo que firmas con tus llaves
- **Archivo de grupo** (`.rlpg`, o el **📱 QR para celulares**): los alumnos lo importan y la app
  comprueba que es tuyo.
- **Retroalimentación** (`.rlpr`): cada alumno solo puede leer la suya.
- **Archivo de acceso** (`.rlpa`, en el detalle del alumno): para quien perdió su contraseña y su
  código de recuperación; la app le da una contraseña temporal para entregarle.

---

## 2. Llaves de firma de LP Alumno

Cada versión de LP Alumno firma las entregas con una llave incluida en la app. LP Profesor revisa
esa firma en **Firma de la app** (el semáforo):

| Resultado | Qué significa |
|---|---|
| 🟢 Verde | Firmada con la llave **de producción** de la app nativa (Windows o Android). |
| 🟡 Amarillo | Firmada con la llave **de desarrollo** (una compilación de prueba) o con la llave **web**. El código de una página web se puede descargar, así que su firma no prueba nada por sí sola: revisa el historial y la reproducción de la escritura. |
| 🔴 Rojo | Llave desconocida o archivo modificado fuera de la app. |

Hay **dos** llaves de producción. La parte secreta de cada una es su **semilla** (44 caracteres
terminados en `=`); la parte pública va en `crates/rlp-core/llaves_app.txt`:

| Llave | Para | Dónde vive la semilla | Su línea en `llaves_app.txt` |
|---|---|---|---|
| `RLP_CLAVE_APP` | Windows y Android, que se compilan en tu PC | En tu gestor de contraseñas; `publicar-version.ps1` te la pide al publicar | Sin marca: `v0.2.1 (producción, 2026-10)` |
| `RLP_CLAVE_APP_WEB` | La versión web, que compila GitHub | En el secreto de GitHub `RLP_CLAVE_APP_WEB` | Con la marca `[web]` |

> **Nunca subas una semilla al repositorio** ni la mandes por chat o correo. Si se pierde no se
> puede recuperar: hay que crear otra llave (sección 2.3). La llave pública sí se puede
> compartir.

### 2.1 Guardar un secreto en GitHub
La versión web necesita `RLP_CLAVE_APP_WEB`. `RLP_CLAVE_APP` solo hace falta en GitHub si usas los
workflows manuales de respaldo (sección 4).

1. En el repositorio: **Settings → Secrets and variables → Actions**.
2. Pestaña **Secrets** → **New repository secret** (o el lápiz de uno que ya existe, para
   cambiarlo).
3. **Name**: `RLP_CLAVE_APP_WEB` (o `RLP_CLAVE_APP`). **Secret**: la semilla, sin espacios.
4. **Add secret**. GitHub no la vuelve a mostrar: guárdala también en tu gestor de contraseñas.

### 2.2 Comprobar que una semilla es la correcta
En tu PC, en PowerShell, dentro de la carpeta del repositorio:

```powershell
$env:RLP_CLAVE_APP = "<semilla nativa>"
cargo run -p rlp-core --example verificar_llave_app          # app nativa
$env:RLP_CLAVE_APP_WEB = "<semilla web>"
cargo run -p rlp-core --example verificar_llave_app -- web   # versión web
```

Debe decir **"Llave de firma de producción reconocida"**. Si dice que la llave pública **no está
en `llaves_app.txt`**, esa semilla no es la de esta versión (sección 2.3). `publicar-version.ps1` y
Build Web hacen la misma comprobación antes de publicar. Nunca muestra la semilla.

### 2.3 Crear o cambiar una llave
Si se perdió una semilla o se filtró la nativa:

1. En tu PC: `cargo run -p rlp-core --example generar_llave_app`. Muestra una **semilla** y una
   **llave pública**.
2. Guarda la semilla en tu gestor de contraseñas (y en el secreto de GitHub que corresponda,
   sección 2.1).
3. Agrega la pública como línea nueva en `crates/rlp-core/llaves_app.txt`:
   `<pública> v0.3.0 (producción, 2027-01)`. Para la web, agrega la marca:
   `<pública> v0.3.0 [web] (producción, 2027-01)`.
4. **No borres las líneas de llaves con las que ya se publicó**: LP Profesor las necesita para
   seguir verificando las entregas viejas.
5. Publica una versión nueva (sección 4): solo las apps compiladas con la lista nueva reconocen la
   llave nueva.

### 2.4 Compilar en tu computadora con la llave
`publicar-version.ps1` pide la semilla sin mostrarla y la olvida al terminar. Para compilar sin
publicar, defínela antes en esa ventana de PowerShell (así no queda en el historial):

```powershell
$env:RLP_CLAVE_APP = [Net.NetworkCredential]::new("", (Read-Host "Semilla" -AsSecureString)).Password
.\scripts\compilar-windows.ps1      # o .\scripts\compilar-android.ps1
```

Sin la variable se usa la llave de desarrollo y las entregas salen en amarillo.

---

## 3. Android

### La llave del APK
`.\scripts\compilar-android.ps1` compila el APK **optimizado** y lo firma con la **llave de
depuración de tu PC**: `%USERPROFILE%\.android\debug.keystore` (la crea Android Studio; si no
existe, el script la crea). No hace falta crear un keystore.

- Esa llave no cambia entre compilaciones: los alumnos **instalan cada versión nueva encima, sin
  perder datos**, siempre que compiles en **la misma PC**.
- **Respalda `debug.keystore`** (memoria USB o nube). Si se pierde, o compilas en otra PC, el APK
  sale con otra firma y Android no lo deja instalar encima ("conflicto con un paquete
  existente"): hay que desinstalar, y eso borra los datos. Pide a los alumnos que **exporten su
  entrega antes** y después usen **Tengo mis avances en un archivo**.
- El APK de v0.2.0 que compiló GitHub tiene otra firma: quien lo instaló debe exportar,
  desinstalarlo e instalar el nuevo.
- `-Depuracion` compila sin optimizar (unas 10 veces más grande): solo para probar en un
  emulador, con `-Targets x86_64`.
- Si cambiaste el nombre de la app y el celular sigue mostrando el anterior, regenera el proyecto
  de Android una vez: `.\scripts\compilar-android.ps1 -Regenerar`.

### Más adelante: tu propio keystore
Para firmar con un keystore propio (por ejemplo, si algún día se publica en una tienda):

1. Créalo **una sola vez** y guárdalo junto con su contraseña:
   ```powershell
   & "C:\Program Files\Android\Android Studio\jbr\bin\keytool.exe" -genkeypair -v `
     -keystore C:\llaves\lp-alumno.jks -alias lp -keyalg RSA -keysize 4096 -validity 10000
   ```
2. Compila con él: `.\scripts\compilar-android.ps1 -Keystore C:\llaves\lp-alumno.jks -Alias lp`.
   Para que `publicar-version.ps1` lo use, define antes `$env:ANDROID_KEYSTORE_FILE` y
   `$env:ANDROID_KEY_ALIAS`.
3. Cambiar de la llave de depuración a tu keystore obliga a **desinstalar una vez**: avisa a los
   alumnos que exporten antes.

---

## 4. Publicar una versión

Las apps de Windows y el APK se compilan **en tu PC**. GitHub solo hace dos cosas:

- **CI**: corre las pruebas en cada push y en cada PR.
- **Build Web**: con cada etiqueta `v*`, publica la versión web en GitHub Pages y agrega su `.zip`
  al Release.

### Preparar (una sola vez)
1. **En tu PC**: lo del README (Rust, Node 22, pnpm y el SDK de Android) y **GitHub CLI**:
   `winget install GitHub.cli` y después `gh auth login`.
2. **Secreto `RLP_CLAVE_APP_WEB`** en GitHub (sección 2.1).
3. **GitHub Pages**, para la versión web:
   1. **Settings → Pages → Build and deployment → Source: GitHub Actions**.
   2. **Settings → Secrets and variables → Actions → Variables → New repository variable**:
      `RLP_PAGES` = `1`.
   3. **Settings → Environments → github-pages → Deployment branches and tags → Add deployment
      branch or tag rule**: *Ref type* **Tag**, *Name pattern* `v*` → **Add rule**. Sin esta regla,
      GitHub solo deja publicar desde `main` y rechaza las etiquetas.

   La dirección queda como `https://<usuario>.github.io/<repositorio>/`; compártela con los
   alumnos.

### Cada versión
1. Anota los cambios en `CHANGELOG.md`, en una sección `## [X.Y.Z] — AAAA-MM-DD` con la fecha del
   día, y sube el número de versión en `Cargo.toml`, en los `package.json` de las apps y de
   `packages/alumno-ui`, y en los `tauri.conf.json`. Fusiona todo en `main`.
2. En tu PC, en PowerShell, dentro de la carpeta del repositorio:
   ```powershell
   git checkout main
   git pull
   .\scripts\publicar-version.ps1
   ```
   Te pide la semilla `RLP_CLAVE_APP` (no se muestra) y la comprueba. Luego compila Windows y
   Android, crea la etiqueta `vX.Y.Z` y sube todo al Release con sus notas.
3. En unos minutos, GitHub publica la versión web (**Actions → Build Web**).

| En el Release | Lo genera |
|---|---|
| `LP-Alumno-…-windows.zip`, `LP-Profesor-…-windows.zip` y las notas del CHANGELOG | Tu PC (`publicar-version.ps1`) |
| `LP-Alumno-…-android.apk` | Tu PC (`publicar-version.ps1`) |
| `LP-Alumno-…-web.zip` y GitHub Pages | GitHub (Build Web) |

Opciones: `-SinAndroid` o `-SinWindows` omiten una parte. Sobre una versión ya publicada, vuelven a
compilar y reemplazan sus archivos, siempre que el código de la app no haya cambiado desde su
etiqueta.

> **Respaldo**: si no puedes compilar en tu PC, en GitHub **Actions → Build Windows** (o **Build
> Android**) **→ Run workflow**, eligiendo la etiqueta en *Use workflow from*. Necesitan el secreto
> `RLP_CLAVE_APP`; el APK que hace GitHub es de depuración (mucho más grande) y con otra firma.

---

## 5. Problemas comunes

- **"La llave pública … no está en `llaves_app.txt`"**: esa semilla no es la de esta versión.
  Comprueba con la sección 2.2; si la perdiste, crea otra (2.3).
- **"Falta el secreto RLP_CLAVE_APP_WEB"** en Build Web: guárdalo (2.1) y vuelve a correr el
  workflow (**Actions → la corrida → Re-run failed jobs**).
- **"Tag … is not allowed to deploy to github-pages due to environment protection rules"** en
  Build Web: falta la regla de etiquetas del entorno `github-pages` (sección 4, *Preparar*, paso
  3.3). Agrégala y vuelve a correr los jobs fallidos.
- **`publicar-version.ps1` se detiene antes de compilar**: el mensaje dice qué falta (estar en
  `main` y al día, no tener cambios sin guardar, la fecha en el CHANGELOG o iniciar sesión con
  `gh auth login`).
- **Las entregas de Windows o Android salen en amarillo en "Firma de la app"**: esa versión se
  compiló sin `RLP_CLAVE_APP`, con la llave de desarrollo (sección 2.4).
- **Todas las entregas web salen en amarillo**: es lo esperado (sección 2).
- **Android no deja instalar la versión nueva ("conflicto con un paquete existente")**: el APK se
  firmó con otra llave (sección 3). Exportar, desinstalar e instalar el nuevo.
- **"La entrega no está dirigida a este profesor"** (Descifrado en rojo): la entrega es de un
  grupo creado con otras llaves, o el alumno no estaba en tu grupo cuando exportó. Restaura tu
  `.rlpk` correcto, pide a ese profesor que te agregue como coprofesor o pide al alumno que se una
  a tu grupo (**Unirme a un grupo**) y vuelva a exportar.
- **Perdí la contraseña de mis llaves**: no se puede recuperar, y el `.rlpk` usa la misma
  contraseña. Crea llaves nuevas y grupos nuevos, y los alumnos se unen a ellos (sus perfiles y su
  avance se conservan). Las entregas que hagan desde entonces sí las podrás abrir.
