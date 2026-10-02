# Instalación en un laboratorio

Guía para poner a funcionar **LP Alumno** y **LP Profesor** en las computadoras de una escuela.
Para el uso diario están el [manual del alumno](MANUAL-ALUMNO.md) y el
[manual del profesor](MANUAL-PROFESOR.md) (llaves, claves de firma y publicación).
Ninguna de las dos apps necesita internet ni permisos de administrador.

## 1. Descargar

En la página de **Releases** del repositorio descarga los dos archivos de la versión más reciente:

- `LP-Alumno-<versión>-windows.zip` (≈ 60 MB; incluye Python para crear ejecutables)
- `LP-Profesor-<versión>-windows.zip`

Requisitos: Windows 10 u 11 de 64 bits con **Microsoft Edge WebView2**. Windows 11 ya lo trae y
Windows 10 lo recibe con Windows Update. Si una computadora no lo tiene y no hay internet,
descarga en otra computadora el instalador **"Evergreen Standalone Installer" (x64)** desde la
página de WebView2 de Microsoft y ejecútalo en cada equipo.

## 2. Preparar la App Profesor (una sola vez)

1. Descomprime `LP-Profesor` en tu computadora (por ejemplo, en `Documentos`).
2. Abre **LP Profesor.exe** y crea tus llaves con una contraseña de al menos 10 caracteres.
3. **Guarda el respaldo de tus llaves** (`.rlpk`) en una memoria USB o en tu nube.
   Sin tus llaves no se pueden abrir las entregas de tus alumnos.
4. En **Grupos**, crea un grupo por cada clase (nombre, formato del número de control, política
   de pegado).

Tus datos quedan en la carpeta `datos_profesor`, junto a la app. Respáldala de vez en cuando.

## 3. Instalar la App Alumno en el laboratorio

La carpeta es **portable**: se copia, no se instala.

1. Descomprime `LP-Alumno` una vez.
2. En la App Profesor, en tu grupo, elige **"Instalar en carpeta de la App Alumno"** y selecciona
   esa carpeta: así cada alumno queda unido al grupo desde el primer uso.
   (Otra opción: **"Guardar archivo de grupo"** y compartir el `.rlpg` para que cada alumno lo
   importe.)
3. Copia la carpeta a cada computadora, por ejemplo a `C:\LP-Alumno` o a `D:\`, en un lugar
   donde los alumnos **puedan escribir**. También pueden llevarla en su memoria USB.
4. Crea un acceso directo a `LP Alumno.exe` en el escritorio.

Cada alumno se registra con su número de control, su nombre y una contraseña, y **anota su
código de recuperación**. Varias personas pueden usar la misma carpeta: cada perfil está cifrado
con la contraseña de su dueño.

### Antivirus

La opción **Crear .exe** genera programas con PyInstaller. Algunos antivirus desconfían de los
ejecutables nuevos. Si se bloquean, agrega una excepción para las carpetas `runtime` y
`mis_ejecutables` dentro de la carpeta de la App Alumno.

### Computadoras que se restauran al reiniciar

Si el laboratorio borra los cambios al reiniciar (congeladores como Deep Freeze), pon la carpeta
de la app en una unidad que no se restaure o pide a los alumnos que usen su memoria USB. Al final
de cada clase deben **exportar su entrega**, que también les sirve de respaldo.

## 4. Recibir y revisar entregas

1. Cada alumno usa **Exportar entrega** y te da su archivo `.rlp` (USB, carpeta compartida,
   correo o plataforma de la escuela). El archivo solo lo pueden abrir el alumno y tú.
2. En la App Profesor, **Importar entregas** acepta archivos sueltos o una carpeta completa.
   Puedes importar entregas nuevas del mismo alumno cuantas veces quieras: se guarda el historial.
3. Revisa el **semáforo de integridad** (verde, amarillo, rojo), el avance, las estadísticas de
   copias e intentos de pegar y el código de cada actividad; con "Ver cómo lo escribió" puedes
   reproducir la escritura tecla a tecla. Exporta a Excel.
4. Para devolver calificaciones y comentarios, en el tablero usa **Retroalimentación**: comparte
   el `.rlpr` con todo el grupo (cada alumno solo puede abrir lo suyo) y cada quien lo importa
   desde el menú con su nombre.

## 5. Celulares Android

Para quien no tiene computadora en casa hay un **APK** de LP Alumno. Viene en el Release
(`LP-Alumno-<versión>-android.apk`) o se compila en una computadora con Windows y el SDK de
Android con `scripts\compilar-android.ps1` (ver el README).

1. En el celular, abre el APK y permite "instalar apps de origen desconocido" para el navegador o
   el administrador de archivos.
2. En la App Profesor, en **Grupos → QR para celulares**, proyecta el código; cada alumno toca
   **📷 Escanear QR del grupo** antes de registrarse (o después, desde el menú con su nombre).
3. Para entregar, **Exportar entrega** abre "Guardar como": se puede guardar en Descargas o en
   Drive y compartir desde ahí.

> **Importante**: en Android los datos viven dentro de la app. **Desinstalarla los borra**:
> exporta tu entrega antes. Una versión nueva se instala encima y conserva los datos siempre que el
> APK esté firmado con la misma llave (la de la computadora donde se compila: ver el
> [manual del profesor](MANUAL-PROFESOR.md#3-android)). Si Android no deja instalarla
> ("conflicto con un paquete existente"), la llave cambió: **exporta la entrega**, desinstala,
> instala la nueva y usa "Tengo mis avances en un archivo".

## 6. Versión web (sin instalar nada)

La App Alumno también funciona **en el navegador**: es la misma app, con el mismo curso y las
mismas entregas `.rlp`. Sirve para celulares (también iPhone), tabletas y computadoras donde no se
puede copiar la carpeta portable.

**Publicarla** (una vez, quien administra el repositorio): el workflow "Build Web" la publica en
GitHub Pages (ver el README) o genera un `.zip` con archivos estáticos que se pueden subir a
cualquier hosting, también una carpeta del servidor de la escuela. No necesita configuración
especial del servidor.

**Para el alumno:**

1. Abre la dirección de la app en Chrome, Edge o Safari.
2. **Instálala**: en Chrome/Edge/Android aparece el botón *Instalar*; en iPhone/iPad toca
   **Compartir → Agregar a pantalla de inicio**. Ya instalada, funciona **sin conexión** (avisa
   cuando termina de descargar Python).
3. Para unirse al grupo: escanear el **QR** que proyecta el profesor o importar el `.rlpg`.
4. **Exportar entrega** descarga el `.rlp` (queda en *Descargas*); se entrega igual que siempre.

> **Importante**: en la versión web los avances viven en el navegador de ese equipo. No uses una
> ventana privada, no borres los datos del sitio y **exporta tu entrega seguido**: la app lo
> recuerda cada 7 días. En iPhone, instálala en la pantalla de inicio para que Safari no la borre.

En la App Profesor, las entregas hechas en la versión web marcan en **amarillo** "Firma de la app"
(el código de una página web se puede descargar, así que su firma no prueba nada por sí sola). El
resto de las revisiones, incluida la reproducción tecla a tecla, funcionan igual.

## 7. Continuar en casa

El alumno exporta su `.rlp` y, en otra computadora con la App Alumno (o en la versión web), elige
**"Tengo mis avances en un archivo"** e ingresa su contraseña. Para volver, exporta allá e
importa acá con **"Importar avances de otro equipo"**.

## 8. Actualizar a una versión nueva

Los datos viven en la carpeta de cada app, así que al actualizar **no borres**:

- En la App Alumno: la carpeta `datos` (perfiles de los alumnos) y `config` (grupo instalado).
- En la App Profesor: la carpeta `datos_profesor`.

Pasos: descomprime la versión nueva y copia encima **solo** el `.exe`, `runtime` y `LEEME.txt`,
o copia las carpetas `datos`/`config` (o `datos_profesor`) de la instalación anterior a la nueva.
Las entregas de versiones anteriores se siguen verificando.

## 9. Olvidé mi contraseña

- **Alumno**: en la pantalla de inicio, "Olvidé mi contraseña" con su código de recuperación.
  Si también lo perdió: en la App Profesor, en el detalle del alumno, **Archivo de acceso** crea un
  `.rlpa` y muestra una contraseña temporal; el alumno elige "Tengo un archivo de acceso de mi
  profesor", pone una contraseña nueva y recibe un código de recuperación nuevo.
- **Profesor**: "Restaurar desde un respaldo" con el archivo `.rlpk` y la contraseña con la que lo
  guardó.
