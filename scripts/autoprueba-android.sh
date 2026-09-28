#!/usr/bin/env bash
# Autoprueba de la App Alumno en un emulador Android (CI): instala el APK de depuración, activa la
# autoprueba con un archivo en la carpeta privada de la app (adb run-as), abre la app y espera el
# resultado que la app deja en autoprueba_resultado.json.
#   Uso: scripts/autoprueba-android.sh ruta/al/app-debug.apk
set -euo pipefail
APK=${1:?ruta del APK de depuración}
PKG=mx.reallearningprogramming.alumno
DATOS=/data/user/0/$PKG

adb wait-for-device
adb shell 'while [ "$(getprop sys.boot_completed)" != "1" ]; do sleep 2; done'
adb shell getprop ro.build.version.release | sed 's/^/Android /'
adb shell dumpsys package com.google.android.webview 2>/dev/null | grep -m1 versionName | sed 's/^ */WebView /' || true

adb install -r "$APK"
adb shell "run-as $PKG sh -c 'rm -f autoprueba_resultado.json; echo alumno-android:$DATOS/compartida > autoprueba.txt'"
adb logcat -c || true
adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null

resultado=""
for _ in $(seq 1 150); do
  if resultado=$(adb shell "run-as $PKG cat autoprueba_resultado.json" 2>/dev/null) && [ -n "$resultado" ]; then
    break
  fi
  resultado=""
  sleep 4
done

registro() {
  # Salida de Rust (println!/eprintln!, incluida la consola del WebView en depuración) y errores.
  adb logcat -d | grep -E "RustStdoutStderr|RustPanic|Tauri|chromium|AndroidRuntime|DEBUG   :" | tail -250
}

if [ -z "$resultado" ]; then
  echo "La app no terminó la autoprueba. Registro del sistema:"
  registro
  exit 1
fi

echo "$resultado" | python3 -c '
import json, sys
r = json.load(sys.stdin)
for p in r["pasos"]:
    print(("  ✔ " if p["ok"] else "  ✖ ") + p["paso"] + ("" if p["ok"] else ": " + str(p.get("detalle"))))
sys.exit(0 if r["ok"] else 1)' || {
  registro
  exit 1
}
echo "Autoprueba en Android: OK"
