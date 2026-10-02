# Publica una versión desde esta computadora (Windows): compila LP Alumno y LP Profesor para
# Windows y el APK de Android, crea la etiqueta vX.Y.Z y sube todo al Release de GitHub. Al ver la
# etiqueta, GitHub (workflow Build Web) publica la versión web y agrega su .zip al mismo Release.
#
#   .\scripts\publicar-version.ps1                 # la versión de apps\alumno\package.json
#   .\scripts\publicar-version.ps1 -SinAndroid     # solo Windows
#   .\scripts\publicar-version.ps1 -SinWindows     # solo el APK (por ejemplo, para reemplazarlo)
#
# Antes: anota los cambios en CHANGELOG.md con la fecha (## [X.Y.Z] — AAAA-MM-DD), sube la versión
# (Cargo.toml, package.json y tauri.conf.json), fusiona todo en main y aquí: git checkout main;
# git pull. Requisitos: los de compilar-windows.ps1 y compilar-android.ps1, y GitHub CLI con la
# sesión iniciada (winget install GitHub.cli; gh auth login). Sin gh, los archivos quedan listos
# para subirlos a mano.
param(
  [switch]$SinWindows,
  [switch]$SinAndroid
)
$ErrorActionPreference = "Stop"
$raiz = Split-Path -Parent $PSScriptRoot
Set-Location $raiz

function Ejecutar([string]$programa, [string[]]$argumentos) {
  & $programa @argumentos
  if ($LASTEXITCODE -ne 0) { throw "Falló: $programa $($argumentos -join ' ')" }
}

# Corre un programa sin mostrar su salida ni detenerse si falla; dice si terminó bien.
function Probar([string]$programa, [string[]]$argumentos) {
  $anterior = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try { & $programa @argumentos *> $null } finally { $ErrorActionPreference = $anterior }
  return ($LASTEXITCODE -eq 0)
}

$version = (Get-Content "apps\alumno\package.json" -Raw | ConvertFrom-Json).version
$etiqueta = "v$version"
$repo = ""
if ((git remote get-url origin) -match 'github\.com[:/](.+?)(\.git)?/?$') { $repo = "https://github.com/" + $Matches[1] }
Write-Host "Publicando LP $etiqueta" -ForegroundColor Cyan

# --- Comprobaciones (antes de compilar, que tarda) ---
Ejecutar "git" @("fetch", "--quiet", "origin", "main")
$rama = (git rev-parse --abbrev-ref HEAD).Trim()
if ($rama -ne "main") { throw "Publica desde main (estás en '$rama'): git checkout main; git pull" }
if (git status --porcelain) { throw "Hay cambios sin guardar (git status): guárdalos o descártalos antes de publicar." }
if ((git rev-parse HEAD) -ne (git rev-parse origin/main)) { throw "Tu main no es igual al de GitHub: git pull" }

$changelog = Get-Content "CHANGELOG.md" -Encoding UTF8
$encabezado = $changelog | Where-Object { $_.StartsWith("## [$version]") } | Select-Object -First 1
if (-not $encabezado) { throw "CHANGELOG.md no tiene la sección '## [$version]'." }
if ($encabezado -notmatch '\d{4}-\d{2}-\d{2}') { throw "Pon la fecha de la versión en CHANGELOG.md: '$encabezado'" }

$etiquetaRemota = [bool](git ls-remote --tags origin "refs/tags/$etiqueta")
if ($etiquetaRemota) {
  # Ya publicada: solo se vuelven a compilar los archivos si el código de la app no cambió desde ella.
  Ejecutar "git" @("fetch", "--quiet", "origin", "+refs/tags/${etiqueta}:refs/tags/$etiqueta")
  if (-not (Probar "git" @("diff", "--quiet", $etiqueta, "HEAD", "--", "apps", "crates", "packages", "curso", "Cargo.lock", "pnpm-lock.yaml"))) {
    throw "La app cambió desde ${etiqueta}: sube la versión y publica una nueva."
  }
  Write-Host "La etiqueta $etiqueta ya existe: se actualizan sus archivos en el Release."
}

$conGh = [bool](Get-Command gh -ErrorAction SilentlyContinue)
if ($conGh) {
  if (-not (Probar "gh" @("auth", "status"))) { throw "Inicia sesión en GitHub CLI: gh auth login" }
} else {
  Write-Warning "No encontré GitHub CLI (gh): al final tendrás que subir los archivos al Release a mano."
}

# --- Llave de firma de las entregas ---
$semillaPedida = $false
if (-not $env:RLP_CLAVE_APP) {
  $segura = Read-Host "Semilla de la llave de firma de la app (RLP_CLAVE_APP; no se muestra)" -AsSecureString
  $env:RLP_CLAVE_APP = [System.Net.NetworkCredential]::new("", $segura).Password
  $semillaPedida = $true
}
try {
  Ejecutar "cargo" @("run", "-q", "-p", "rlp-core", "--example", "verificar_llave_app")

  # --- Compilar ---
  $archivos = @()
  if (-not $SinWindows) {
    & "$PSScriptRoot\compilar-windows.ps1" -Version $version
    $archivos += "dist-portable\LP-Alumno-$version-windows.zip", "dist-portable\LP-Profesor-$version-windows.zip"
  }
  if (-not $SinAndroid) {
    & "$PSScriptRoot\compilar-android.ps1"
    $archivos += "dist-android\LP-Alumno-$version-android.apk"
  }
  foreach ($archivo in $archivos) { if (-not (Test-Path $archivo)) { throw "No se generó $archivo." } }
} finally {
  if ($semillaPedida) { Remove-Item Env:RLP_CLAVE_APP -ErrorAction SilentlyContinue }
}

# --- Etiqueta (con ella GitHub publica la versión web) ---
if (-not $etiquetaRemota) {
  if (git tag --list $etiqueta) {
    if ((git rev-parse "$etiqueta^{commit}") -ne (git rev-parse HEAD)) { throw "La etiqueta local $etiqueta no apunta a main: git tag -d $etiqueta" }
  } else {
    Ejecutar "git" @("tag", "-a", $etiqueta, "-m", "LP $etiqueta")
  }
  Ejecutar "git" @("push", "origin", $etiqueta)
}

# --- Notas del Release: la sección del CHANGELOG y cómo descargar ---
$notas = New-Object System.Collections.Generic.List[string]
$dentro = $false
foreach ($linea in $changelog) {
  if ($linea.StartsWith("## [$version]")) { $dentro = $true; continue }
  if ($linea.StartsWith("## [")) { $dentro = $false }
  if ($dentro) { $notas.Add($linea) }
}
$notas.Add("")
$notas.Add("---")
$notas.Add("**Windows**: descarga los dos ``-windows.zip`` (LP Alumno y LP Profesor) y descomprímelos; no")
$notas.Add("requieren instalación ni internet. **Android**: ``-android.apk`` (LP Alumno). **Web**: ``-web.zip``")
$notas.Add("es el sitio de LP Alumno para cualquier hosting.")
$notas.Add("")
$notas.Add("Guías: [instalación]($repo/blob/main/docs/INSTALACION.md),")
$notas.Add("[manual del alumno]($repo/blob/main/docs/MANUAL-ALUMNO.md) y")
$notas.Add("[manual del profesor]($repo/blob/main/docs/MANUAL-PROFESOR.md).")
$archivoNotas = Join-Path $env:TEMP "lp-notas-$version.md"
[IO.File]::WriteAllLines($archivoNotas, $notas, (New-Object System.Text.UTF8Encoding $false))

# --- Release ---
if (-not $conGh) {
  Write-Host ""
  Write-Host "Sube estos archivos al Release $etiqueta ($repo/releases/tag/$etiqueta):" -ForegroundColor Yellow
  $archivos | ForEach-Object { Write-Host "  $_" }
  Write-Host "Título: LP $etiqueta. Notas: $archivoNotas"
  return
}
$previa = @()
if ($version.Contains("-")) { $previa = @("--prerelease") }
if (Probar "gh" @("release", "view", $etiqueta)) {
  Ejecutar "gh" (@("release", "edit", $etiqueta, "--title", "LP $etiqueta", "--notes-file", $archivoNotas) + $previa)
  if (-not $SinAndroid) {
    # Un solo APK por versión: quita los de otro nombre (por ejemplo, uno de depuración).
    $apk = Split-Path -Leaf $archivos[-1]
    $viejos = gh release view $etiqueta --json assets --jq ".assets[].name" | Where-Object { $_ -like "*.apk" -and $_ -ne $apk }
    foreach ($viejo in $viejos) { Ejecutar "gh" @("release", "delete-asset", $etiqueta, $viejo, "--yes") }
  }
  Ejecutar "gh" (@("release", "upload", $etiqueta, "--clobber") + $archivos)
} else {
  Ejecutar "gh" (@("release", "create", $etiqueta, "--verify-tag", "--title", "LP $etiqueta", "--notes-file", $archivoNotas) + $previa + $archivos)
}
Write-Host ""
Write-Host "Release listo: $repo/releases/tag/$etiqueta" -ForegroundColor Green
Write-Host "La versión web la publica GitHub en unos minutos (Actions → Build Web): $repo/actions"
