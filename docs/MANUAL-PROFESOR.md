# Manual del profesor: llaves, claves de firma y publicación

LP usa dos tipos de llaves, y conviene no confundirlas:

| | Para qué sirve | Quién la tiene | Dónde se configura |
|---|---|---|---|
| **Tus llaves de profesor** | Abrir las entregas de tus alumnos y firmar tus grupos, retroalimentaciones y archivos de acceso | Solo tú (y tus coprofesores, con las suyas) | En LP Profesor ([sección 1](#1-tus-llaves-de-profesor)) |
| **Llaves de firma de LP Alumno** | Que LP Profesor reconozca que una entrega salió de una versión oficial de la app (el semáforo) | El repositorio, como secretos de GitHub | En GitHub ([sección 2](#2-llaves-de-firma-de-lp-alumno)) |

Para instalar las apps en un laboratorio, ver [`INSTALACION.md`](INSTALACION.md).

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

Hay **dos** llaves de producción, cada una en un **secreto de GitHub**:

| Secreto | Para | Su llave pública en `crates/rlp-core/llaves_app.txt` |
|---|---|---|
| `RLP_CLAVE_APP` | Windows y Android | Línea sin marca (`v0.2.0 (producción, 2026-09)`) |
| `RLP_CLAVE_APP_WEB` | Versión web | Línea con la marca `[web]` |

Las llaves de v0.2.0 ya están generadas y sus públicas ya están en `llaves_app.txt`: solo falta
guardar cada **semilla** (la parte secreta, 44 caracteres terminados en `=`) en su secreto.

### 2.1 Guardar un secreto en GitHub
1. En el repositorio: **Settings → Secrets and variables → Actions**.
2. Pestaña **Secrets** → **New repository secret**.
3. **Name**: `RLP_CLAVE_APP` (o `RLP_CLAVE_APP_WEB`). **Secret**: la semilla, sin espacios.
4. **Add secret**. GitHub no la vuelve a mostrar: guárdala también en tu gestor de contraseñas.

### 2.2 Comprobar que la llave es la correcta (opcional)
En una computadora con el repositorio y Rust, en PowerShell:

```powershell
$env:RLP_CLAVE_APP = "<semilla nativa>"
cargo run -p rlp-core --example verificar_llave_app          # app nativa
$env:RLP_CLAVE_APP_WEB = "<semilla web>"
cargo run -p rlp-core --example verificar_llave_app -- web   # versión web
```

Debe decir **"Llave de firma de producción reconocida"**. Lo mismo corre en GitHub antes de
publicar; si falla, la publicación se detiene (ver la [sección 5](#5-problemas-comunes)). Nunca
muestra la semilla.

### 2.3 Crear o cambiar una llave
Solo si se perdió una semilla o se filtró la nativa:

1. `cargo run -p rlp-core --example generar_llave_app` muestra una **semilla** y una **llave
   pública**.
2. Agrega la pública como línea nueva en `crates/rlp-core/llaves_app.txt`:
   `<pública> v0.3.0 (producción, 2027-01)`. Para la web, agrega la marca:
   `<pública> v0.3.0 [web] (producción, 2027-01)`.
3. **No borres las líneas anteriores**: LP Profesor las necesita para seguir verificando las
   entregas viejas.
4. Reemplaza el secreto en GitHub con la semilla nueva y publica una versión nueva.

> **Nunca subas una semilla al repositorio** ni la mandes por chat o correo. La llave web no
> debe usarse como `RLP_CLAVE_APP` ni al revés: `verificar_llave_app` lo impide.

### 2.4 Compilar en tu computadora con la llave
Para que un APK compilado en tu PC firme como versión oficial, define la semilla nativa antes:

```powershell
$env:RLP_CLAVE_APP = "<semilla nativa>"
.\scripts\compilar-android.ps1
```

Sin la variable se usa la llave de desarrollo y las entregas salen en amarillo.

---

## 3. Android

### Por ahora: APK de depuración
No hacen falta secretos de Android. El Release incluye `LP-Alumno-<versión>-android-depuracion.apk`,
y en tu PC lo genera `.\scripts\compilar-android.ps1` (SDK en `E:\Android`). Limitación: su firma
cambia en cada compilación. Para instalar una versión nueva **hay que desinstalar la anterior, y eso
borra los datos**. Pide a los alumnos que **exporten su entrega antes** y después usen **Tengo mis
avances en un archivo**.

> Si cambiaste el nombre de la app y el celular sigue mostrando el anterior, regenera el proyecto
> de Android una vez: `.\scripts\compilar-android.ps1 -Regenerar`.

### Más adelante: APK firmado (se actualiza sin perder datos)
1. Crea el keystore **una sola vez** y guárdalo junto con su contraseña. Si se pierde, los alumnos
   tendrán que desinstalar para actualizar.
   ```powershell
   & "C:\Program Files\Android\Android Studio\jbr\bin\keytool.exe" -genkeypair -v `
     -keystore C:\llaves\lp-alumno.jks -alias lp -keyalg RSA -keysize 4096 -validity 10000
   ```
2. **En tu PC**:
   `.\scripts\compilar-android.ps1 -Release -Keystore C:\llaves\lp-alumno.jks -Alias lp`.
3. **En GitHub** (para que el Release traiga el APK firmado), crea tres secretos:
   - `ANDROID_KEYSTORE`: el keystore en base64. Cópialo al portapapeles con
     `[Convert]::ToBase64String([IO.File]::ReadAllBytes("C:\llaves\lp-alumno.jks")) | Set-Clipboard`;
   - `ANDROID_KEYSTORE_PASSWORD`: su contraseña;
   - `ANDROID_KEY_ALIAS`: `lp`.
4. Pasar del APK de depuración al firmado requiere **desinstalar una vez**. Avisa a los alumnos que
   exporten antes.

---

## 4. Publicar una versión

### GitHub Pages para la versión web (opcional, una vez)
1. **Settings → Pages → Build and deployment → Source: GitHub Actions**.
2. **Settings → Secrets and variables → Actions → Variables → New repository variable**:
   `RLP_PAGES` = `1`.

La dirección queda como `https://<usuario>.github.io/<repositorio>/`; compártela con los alumnos.

### Crear la versión
1. Anota los cambios en `CHANGELOG.md` y sube el número de versión: `Cargo.toml`, los
   `package.json` y los `tauri.conf.json` de las apps.
2. Crea la etiqueta sobre `main`:
   - con git: `git tag v0.3.0` y `git push origin v0.3.0`;
   - o en GitHub: **Releases → Draft a new release → Choose a tag** → escribe `v0.3.0` → *Create
     new tag on publish* → Target `main` → **Publish release**.
3. Los workflows llenan el Release:

| Workflow | Agrega | Necesita |
|---|---|---|
| Build Windows | `LP-Alumno-…-windows.zip`, `LP-Profesor-…-windows.zip` y las notas del CHANGELOG | `RLP_CLAVE_APP` |
| Build Web | `LP-Alumno-…-web.zip` (y GitHub Pages si `RLP_PAGES=1`) | `RLP_CLAVE_APP_WEB` |
| Build Android | `LP-Alumno-…-android-depuracion.apk` (o el firmado, si hay keystore) | — |

---

## 5. Problemas comunes

- **La publicación se detiene en "Llave de firma de producción"**: falta el secreto o no coincide
  con `llaves_app.txt`. Revisa las secciones 2.1 y 2.2 y vuelve a correr el workflow (Actions → el
  workflow → *Re-run jobs*).
- **Las entregas de Windows o Android salen en amarillo en "Firma de la app"**: esa versión se
  compiló sin `RLP_CLAVE_APP`, con la llave de desarrollo. Pasa con compilaciones de prueba o
  locales sin la variable (sección 2.4).
- **Todas las entregas web salen en amarillo**: es lo esperado (sección 2).
- **"La entrega no está dirigida a este profesor"** (Descifrado en rojo): la entrega es de un
  grupo creado con otras llaves, o el alumno no estaba en tu grupo cuando exportó. Restaura tu
  `.rlpk` correcto, pide a ese profesor que te agregue como coprofesor o pide al alumno que se una
  a tu grupo (**Unirme a un grupo**) y vuelva a exportar.
- **Perdí la contraseña de mis llaves**: no se puede recuperar, y el `.rlpk` usa la misma
  contraseña. Crea llaves nuevas y grupos nuevos, y los alumnos se unen a ellos (sus perfiles y su
  avance se conservan). Las entregas que hagan desde entonces sí las podrás abrir.
