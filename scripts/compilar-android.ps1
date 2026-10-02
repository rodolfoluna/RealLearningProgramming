# Compila el APK de la App Alumno en esta computadora (Windows). El APK queda en dist-android\.
#
#   .\scripts\compilar-android.ps1                          # APK de depuración (de prueba)
#   .\scripts\compilar-android.ps1 -Instalar                # además lo instala con adb
#   .\scripts\compilar-android.ps1 -Targets x86_64          # para un emulador
#   .\scripts\compilar-android.ps1 -Release -Keystore C:\llaves\lp-alumno.jks -Alias lp
#
# Requisitos: pnpm install hecho, Rust (rustup), SDK de Android con NDK, build-tools y
# platform-tools, y Java 17 (el JBR de Android Studio sirve). Para que las entregas salgan en
# verde, define antes $env:RLP_CLAVE_APP con la semilla de la llave de la app.
param(
  # Carpeta del SDK de Android (por defecto ANDROID_HOME o E:\Android).
  [string]$Sdk = $(if ($env:ANDROID_HOME) { $env:ANDROID_HOME } else { "E:\Android" }),
  # aarch64 y armv7 para celulares; x86_64 o i686 para emuladores.
  [string[]]$Targets = @("aarch64", "armv7"),
  # APK firmado con tu keystore (actualizable sin perder datos). Sin esto: APK de prueba.
  [switch]$Release,
  [string]$Keystore = $env:ANDROID_KEYSTORE_FILE,
  [string]$Alias = $env:ANDROID_KEY_ALIAS,
  # Instala el APK con adb en el celular o emulador conectado.
  [switch]$Instalar,
  # Vuelve a generar el proyecto de Android (apps\alumno\src-tauri\gen\android).
  [switch]$Regenerar
)
$ErrorActionPreference = "Stop"
$raiz = Split-Path -Parent $PSScriptRoot
Set-Location $raiz

function Ejecutar([string]$programa, [string[]]$argumentos) {
  & $programa @argumentos
  if ($LASTEXITCODE -ne 0) { throw "Falló: $programa $($argumentos -join ' ')" }
}

# La versión más nueva de una carpeta como ndk\ o build-tools\ (ignora sufijos como "-rc1").
function MasNueva([string]$carpeta) {
  if (-not (Test-Path $carpeta)) { return $null }
  Get-ChildItem $carpeta -Directory |
    Sort-Object { try { [version]($_.Name -replace '-.*$', '') } catch { [version]"0.0" } } -Descending |
    Select-Object -First 1
}

# --- Herramientas ---
if (-not (Test-Path (Join-Path $Sdk "platforms"))) {
  throw "No encontré el SDK de Android en '$Sdk'. Indica la carpeta con -Sdk o la variable ANDROID_HOME."
}
$env:ANDROID_HOME = $Sdk
$env:ANDROID_SDK_ROOT = $Sdk

if (-not $env:NDK_HOME -or -not (Test-Path $env:NDK_HOME)) {
  $ndk = MasNueva (Join-Path $Sdk "ndk")
  if (-not $ndk) {
    throw "No hay NDK en '$Sdk\ndk'. Instálalo en Android Studio: SDK Manager > SDK Tools > NDK (Side by side)."
  }
  $env:NDK_HOME = $ndk.FullName
}

if (-not $env:JAVA_HOME -or -not (Test-Path (Join-Path $env:JAVA_HOME "bin\java.exe"))) {
  $candidatos = @(
    (Join-Path $env:ProgramFiles "Android\Android Studio\jbr"),
    (Join-Path (Split-Path -Parent $Sdk) "Android Studio\jbr"),
    (Join-Path $Sdk "Android Studio\jbr"),
    (Join-Path $env:LOCALAPPDATA "Programs\Android Studio\jbr")
  )
  $jbr = $candidatos | Where-Object { Test-Path (Join-Path $_ "bin\java.exe") } | Select-Object -First 1
  if ($jbr) {
    $env:JAVA_HOME = $jbr
  } elseif (-not (Get-Command java -ErrorAction SilentlyContinue)) {
    throw "No encontré Java 17. Define JAVA_HOME (por ejemplo, la carpeta jbr de Android Studio)."
  }
}
$env:Path = "$Sdk\platform-tools;$env:Path"
if ($env:JAVA_HOME) { $env:Path = "$env:JAVA_HOME\bin;$env:Path" }

Write-Host "SDK:  $Sdk"
Write-Host "NDK:  $env:NDK_HOME"
Write-Host "Java: $(if ($env:JAVA_HOME) { $env:JAVA_HOME } else { (Get-Command java).Source })"

# --- Rust ---
$triples = @{
  aarch64 = "aarch64-linux-android"; armv7 = "armv7-linux-androideabi"
  x86_64 = "x86_64-linux-android"; i686 = "i686-linux-android"
}
foreach ($t in $Targets) {
  if (-not $triples.ContainsKey($t)) { throw "Target desconocido '$t' (usa aarch64, armv7, x86_64 o i686)." }
}
$instalados = rustup target list --installed
$faltan = $Targets | ForEach-Object { $triples[$_] } | Where-Object { $instalados -notcontains $_ }
if ($faltan) { Ejecutar "rustup" (@("target", "add") + $faltan) }

if ($Release) {
  if (-not $Keystore -or -not (Test-Path $Keystore)) { throw "Con -Release indica tu keystore: -Keystore ruta\al\archivo.jks" }
  if (-not $Alias) { throw "Con -Release indica el alias de la llave: -Alias nombre" }
  if (-not $env:ANDROID_KEYSTORE_PASSWORD) {
    $segura = Read-Host "Contraseña del keystore" -AsSecureString
    $env:ANDROID_KEYSTORE_PASSWORD = [Runtime.InteropServices.Marshal]::PtrToStringAuto(
      [Runtime.InteropServices.Marshal]::SecureStringToBSTR($segura))
  }
  if (-not $env:RLP_CLAVE_APP) {
    Write-Warning "Sin RLP_CLAVE_APP: las entregas de este APK saldrán con la firma de desarrollo (amarillo en la App Profesor)."
  }
}

# --- Compilar ---
Push-Location "apps\alumno"
try {
  if ($Regenerar -and (Test-Path "src-tauri\gen\android")) { Remove-Item -Recurse -Force "src-tauri\gen\android" }
  if (-not (Test-Path "src-tauri\gen\android")) {
    Write-Host "Generando el proyecto de Android (tauri android init)…"
    Ejecutar "pnpm" @("tauri", "android", "init", "--ci")
    Ejecutar "pnpm" @("tauri", "icon", "src-tauri\icons\icon.png")
  }
  $argumentos = @("tauri", "android", "build", "--apk")
  if (-not $Release) { $argumentos += "--debug" }
  foreach ($t in $Targets) { $argumentos += @("--target", $t) }
  $inicio = Get-Date
  Ejecutar "pnpm" $argumentos
} finally {
  Pop-Location
}

# --- Firmar y nombrar ---
$version = (Get-Content "apps\alumno\package.json" -Raw | ConvertFrom-Json).version
$apks = "apps\alumno\src-tauri\gen\android\app\build\outputs\apk"
New-Item -ItemType Directory -Force -Path "dist-android" | Out-Null
$patron = if ($Release) { "*release-unsigned.apk" } else { "*debug.apk" }
$generado = Get-ChildItem $apks -Recurse -Filter $patron |
  Where-Object { $_.LastWriteTime -ge $inicio } | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $generado) { throw "No encontré el APK compilado ($patron) en $apks." }

if ($Release) {
  $herramientas = MasNueva (Join-Path $Sdk "build-tools")
  if (-not $herramientas) { throw "No hay build-tools en '$Sdk\build-tools' (SDK Manager > SDK Tools > Android SDK Build-Tools)." }
  $alineado = Join-Path $env:TEMP "rlp-alineado.apk"
  $destino = "dist-android\LP-Alumno-$version-android.apk"
  Ejecutar (Join-Path $herramientas.FullName "zipalign.exe") @("-f", "4", $generado.FullName, $alineado)
  Ejecutar (Join-Path $herramientas.FullName "apksigner.bat") @("sign", "--ks", $Keystore,
    "--ks-pass", "env:ANDROID_KEYSTORE_PASSWORD", "--ks-key-alias", $Alias, "--out", $destino, $alineado)
  Ejecutar (Join-Path $herramientas.FullName "apksigner.bat") @("verify", $destino)
  Remove-Item $alineado -ErrorAction SilentlyContinue
} else {
  $destino = "dist-android\LP-Alumno-$version-android-depuracion-$($Targets -join '-').apk"
  Copy-Item $generado.FullName $destino -Force
}
Write-Host ""
Write-Host "APK listo: $destino" -ForegroundColor Green

if ($Instalar) {
  Ejecutar "adb" @("install", "-r", $destino)
  Write-Host "Instalado. Si Android no lo deja actualizar (firma distinta), desinstala la app antes; se borran sus datos."
}
