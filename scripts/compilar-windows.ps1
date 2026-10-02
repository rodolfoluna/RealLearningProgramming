# Compila LP Alumno y LP Profesor para Windows en esta computadora y arma las carpetas portables y
# sus .zip en dist-portable\ (lo mismo que el workflow Build Windows, que ya solo corre a mano).
#
#   .\scripts\compilar-windows.ps1                 # dist-portable\LP-*-<versión>-windows.zip
#   .\scripts\compilar-windows.ps1 -Autoprueba     # además, el flujo profesor → alumno → profesor
#
# Requisitos: Rust (rustup, con las herramientas de C++ de Visual Studio), Node 22, pnpm e
# internet la primera vez (descarga Python y PyInstaller para runtime\). Para que las entregas
# salgan en verde, define antes $env:RLP_CLAVE_APP con la semilla de la llave de la app
# (publicar-version.ps1 la pide).
param(
  # Por defecto, la de apps\alumno\package.json.
  [string]$Version = "",
  # Corre la autoprueba con las apps reales (incluye crear un .exe).
  [switch]$Autoprueba,
  # Vuelve a descargar Python y PyInstaller para runtime\ (si no, se reutiliza el que haya).
  [switch]$NuevoRuntime
)
$ErrorActionPreference = "Stop"
$raiz = Split-Path -Parent $PSScriptRoot
Set-Location $raiz
if (-not $Version) { $Version = (Get-Content "apps\alumno\package.json" -Raw | ConvertFrom-Json).version }

function Ejecutar([string]$programa, [string[]]$argumentos) {
  & $programa @argumentos
  if ($LASTEXITCODE -ne 0) { throw "Falló: $programa $($argumentos -join ' ')" }
}

foreach ($herramienta in "cargo", "node", "pnpm") {
  if (-not (Get-Command $herramienta -ErrorAction SilentlyContinue)) { throw "Falta '$herramienta' (ver el README, sección Desarrollo)." }
}

# --- Llave de firma de las entregas ---
if ($env:RLP_CLAVE_APP) {
  Ejecutar "cargo" @("run", "-q", "-p", "rlp-core", "--example", "verificar_llave_app")
} else {
  Write-Warning "Sin RLP_CLAVE_APP: las entregas de estas apps saldrán con la firma de desarrollo (amarillo en LP Profesor)."
}

# --- Dependencias, curso y Python portátil ---
Ejecutar "pnpm" @("install", "--frozen-lockfile")
Ejecutar "pnpm" @("preparar")
if ($NuevoRuntime -or -not (Test-Path "runtime\python.exe")) { & "$PSScriptRoot\preparar-runtime.ps1" }

Write-Host "Probando que runtime\ genere un .exe…"
$python = (Resolve-Path "runtime\python.exe").Path
$prueba = Join-Path $env:TEMP "lp-prueba-exe"
if (Test-Path $prueba) { Remove-Item -Recurse -Force $prueba }
New-Item -ItemType Directory -Force -Path $prueba | Out-Null
Set-Content (Join-Path $prueba "hola.py") 'n = input("Nombre: ")', 'print("Hola,", n)' -Encoding utf8
Ejecutar $python @("-m", "PyInstaller", "--onefile", "--console", "--noconfirm", "--log-level", "WARN",
  "--distpath", (Join-Path $prueba "dist"), "--workpath", (Join-Path $prueba "build"), "--specpath", $prueba,
  (Join-Path $prueba "hola.py"))
$salida = "Ana" | & (Join-Path $prueba "dist\hola.exe")
if (-not ($salida -match "Hola, Ana")) { throw "El ejecutable de prueba no funcionó: $salida" }
Remove-Item -Recurse -Force $prueba

# --- Compilar ---
# Cada app se compila por separado: así LP Profesor NO incluye la llave de firma.
Ejecutar "pnpm" @("--filter", "@rlp/alumno", "tauri", "build", "--no-bundle")
Ejecutar "pnpm" @("--filter", "@rlp/profesor", "tauri", "build", "--no-bundle")
& "$PSScriptRoot\empaquetar.ps1" -Version $Version

if ($Autoprueba) {
  Ejecutar "cargo" @("build", "-p", "rlp-alumno", "-p", "rlp-profesor", "--features", "tauri/custom-protocol")
  $env:RLP_PYTHON = $python
  $env:RLP_AUTOPRUEBA_EXE = "1"
  & "$PSScriptRoot\autoprueba.ps1"
}

Write-Host ""
Write-Host "Listo: dist-portable\LP-Alumno-$Version-windows.zip y dist-portable\LP-Profesor-$Version-windows.zip" -ForegroundColor Green
