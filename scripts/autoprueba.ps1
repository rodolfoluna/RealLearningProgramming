# Versión Windows de scripts/autoprueba.sh (WebView2): flujo profesor → alumno → profesor con las
# apps reales compiladas en modo desarrollo con el frontend incluido.
$ErrorActionPreference = "Stop"
$bin = if ($env:BIN) { $env:BIN } else { "target\debug" }
$dir = Join-Path $env:TEMP ("rlp-autoprueba-" + [guid]::NewGuid().ToString("N"))
$compartida = Join-Path $dir "compartida"
New-Item -ItemType Directory -Force -Path (Join-Path $dir "alumno"), (Join-Path $dir "profesor"), $compartida | Out-Null

function Correr($app, $carpeta, $fase) {
  Write-Host "== $app ($fase)"
  $env:RLP_CARPETA = $carpeta
  $env:RLP_AUTOPRUEBA = "${fase}:$compartida"
  $log = Join-Path $dir "$app-$fase.log"
  $p = Start-Process -FilePath (Join-Path $bin "$app.exe") -RedirectStandardOutput $log -RedirectStandardError "$log.err" -PassThru -NoNewWindow
  if (-not $p.WaitForExit(240000)) { $p.Kill(); Get-Content "$log.err" -Tail 40; throw "Tiempo agotado: $app $fase" }
  $linea = Get-Content $log | Where-Object { $_ -like "AUTOPRUEBA *" } | Select-Object -First 1
  if (-not $linea) { Get-Content "$log.err" -Tail 40; throw "Sin resultado: $app $fase" }
  $r = $linea.Substring(11) | ConvertFrom-Json
  foreach ($paso in $r.pasos) {
    if ($paso.ok) { Write-Host "  OK  $($paso.paso) $($paso.detalle | ConvertTo-Json -Compress -Depth 2)" }
    else { Write-Host "  MAL $($paso.paso): $($paso.detalle)" }
  }
  if (-not $r.ok) { throw "Falló: $app $fase" }
}

Correr "rlp-profesor" (Join-Path $dir "profesor") "profesor-grupo"
if ($env:RLP_AUTOPRUEBA_EXE -eq "1") {
  Correr "rlp-alumno" (Join-Path $dir "alumno") "alumno+exe"
  $exe = Join-Path $dir "alumno\mis_ejecutables\hola_prueba.exe"
  $salida = "" | & $exe
  Write-Host "== ejecutable generado: $salida"
  if (-not ($salida -match "Hola desde el ejecutable")) { throw "El ejecutable generado no funcionó" }
} else {
  Correr "rlp-alumno" (Join-Path $dir "alumno") "alumno"
}
Correr "rlp-profesor" (Join-Path $dir "profesor") "profesor-importar"
Write-Host "Autoprueba completa: OK"
