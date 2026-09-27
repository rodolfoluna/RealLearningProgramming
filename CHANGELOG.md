# Cambios

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/). Las versiones
siguen [SemVer](https://semver.org/lang/es/). El curso tiene su propia versión (`curso/curso.yaml`).

## [0.2.0] — sin publicar

### Curso 1.1

- Unidades nuevas: **4 Funciones**, **5 Cadenas**, **6 Listas** y **7 Proyectos integradores**
  (control de calificaciones, inventario, ahorcado, gato, agenda, piedra-papel-tijera, conversor
  decimal/binario y punto de venta). 118 actividades en total.
- Las pruebas de función pueden revisar lo que la función **muestra** (`salida`), darle datos para
  sus `input()` (`entrada`) y usar argumentos con nombre (`kwargs`).
- Las funciones de un programa con menú se prueban por separado: el programa principal se detiene
  en su primer `input()`.
- Mensaje específico cuando se usa `print` en lugar de `return`.

### App Profesor

- El mapa de actividades mantiene visible el nombre del alumno al desplazarse y separa las
  unidades.

### Distribución

- Guía de instalación para laboratorios (`docs/INSTALACION.md`).
- Las etiquetas `v*` publican un Release de GitHub con las dos apps portables.

## [0.1.0] — 2026-09-27

Primera versión (Windows).

- **App Alumno**: registro con número de control, nombre y contraseña; código de recuperación;
  datos cifrados en la carpeta de la app; curso con unidades 0–3 (53 actividades); editor con
  pegado bloqueado y estadísticas de copias, intentos de pegar y salidas; consola con `input()`;
  pruebas automáticas; errores explicados en español; pistas; crear `.exe`; exportar e importar
  avances.
- **App Profesor**: llaves con respaldo; grupos con políticas; importación de entregas;
  verificación de integridad (firmas, cadena de eventos, reconstrucción del código tecla a
  tecla); tablero, detalle, calificaciones y CSV.
- CI en Linux y Windows; autoprueba con las apps reales.
