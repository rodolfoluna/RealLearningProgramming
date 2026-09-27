#!/usr/bin/env bash
# Prueba de extremo a extremo con las apps REALES (Tauri + WebView + núcleo Rust), sin interfaz
# gráfica visible (Xvfb en Linux):
#   1. Profesor: crea llaves y un grupo, exporta grupo_Autoprueba.rlpg
#   2. Alumno: importa el grupo, se registra, "teclea" una actividad, corre pruebas en Pyodide,
#      registra copias/pegados y exporta su entrega .rlp
#   3. Profesor: importa la entrega, la verifica (historial, replay, firma) y re-ejecuta pruebas
# Requiere compilar con el frontend incluido:
#   pnpm preparar && pnpm --filter @rlp/alumno build && pnpm --filter @rlp/profesor build
#   cargo build -p rlp-alumno -p rlp-profesor --features tauri/custom-protocol
set -euo pipefail
cd "$(dirname "$0")/.."
BIN=${BIN:-target/debug}
DIR=$(mktemp -d)
mkdir -p "$DIR/alumno" "$DIR/profesor" "$DIR/compartida"
trap 'rm -rf "$DIR"' EXIT

correr() {
  local app=$1 carpeta=$2 fase=$3
  echo "== $app ($fase)"
  local salida
  if ! salida=$(RLP_CARPETA="$carpeta" RLP_AUTOPRUEBA="$fase:$DIR/compartida" timeout 240 \
      ${XVFB:-xvfb-run -a} "$BIN/$app" 2>&1); then
    echo "$salida" | tail -40
    echo "FALLÓ: $app $fase"
    exit 1
  fi
  echo "$salida" | grep '^AUTOPRUEBA' | sed 's/^AUTOPRUEBA //' | python3 -c '
import json, sys
r = json.load(sys.stdin)
for p in r["pasos"]:
    print(("  ✔ " if p["ok"] else "  ✖ ") + p["paso"] + ("" if p["ok"] else ": " + str(p.get("detalle"))))
sys.exit(0 if r["ok"] else 1)'
}

correr rlp-profesor "$DIR/profesor" profesor-grupo
if [ "${RLP_AUTOPRUEBA_EXE:-0}" = "1" ]; then
  # Requiere PyInstaller en RLP_PYTHON (o python3). Luego se ejecuta el programa generado.
  correr rlp-alumno "$DIR/alumno" alumno+exe
  echo "== ejecutable generado"
  salida=$("$DIR/alumno/mis_ejecutables/hola_prueba" < /dev/null)
  echo "  $salida" | head -2
  echo "$salida" | grep -q "Hola desde el ejecutable" || { echo "FALLÓ: el ejecutable no funcionó"; exit 1; }
else
  correr rlp-alumno "$DIR/alumno" alumno
fi
correr rlp-profesor "$DIR/profesor" profesor-importar
echo "Autoprueba completa: OK"
