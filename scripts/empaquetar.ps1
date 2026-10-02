# Arma las carpetas portables y sus .zip a partir de los ejecutables compilados:
#   dist-portable\LP-Alumno\   (LP Alumno.exe, runtime\, config\, LEEME.txt, MANUAL-ALUMNO.md)
#   dist-portable\LP-Profesor\ (LP Profesor.exe, LEEME.txt, INSTALACION.md, MANUAL-PROFESOR.md,
#                               MANUAL-ALUMNO.md para compartirlo con el grupo)
param([string]$Version = "0.2.0")
$ErrorActionPreference = "Stop"
$salida = "dist-portable"
if (Test-Path $salida) { Remove-Item -Recurse -Force $salida }
$alumno = Join-Path $salida "LP-Alumno"
$profesor = Join-Path $salida "LP-Profesor"
New-Item -ItemType Directory -Force -Path (Join-Path $alumno "config"), $profesor | Out-Null
Copy-Item "target\release\rlp-alumno.exe" (Join-Path $alumno "LP Alumno.exe")
Copy-Item -Recurse "runtime" (Join-Path $alumno "runtime")
Copy-Item "docs\LEEME-alumno.txt" (Join-Path $alumno "LEEME.txt")
Copy-Item "docs\MANUAL-ALUMNO.md" (Join-Path $alumno "MANUAL-ALUMNO.md")
Copy-Item "target\release\rlp-profesor.exe" (Join-Path $profesor "LP Profesor.exe")
Copy-Item "docs\LEEME-profesor.txt" (Join-Path $profesor "LEEME.txt")
Copy-Item "docs\INSTALACION.md" (Join-Path $profesor "INSTALACION.md")
Copy-Item "docs\MANUAL-PROFESOR.md" (Join-Path $profesor "MANUAL-PROFESOR.md")
Copy-Item "docs\MANUAL-ALUMNO.md" (Join-Path $profesor "MANUAL-ALUMNO.md")
Compress-Archive -Path $alumno -DestinationPath (Join-Path $salida "LP-Alumno-$Version-windows.zip")
Compress-Archive -Path $profesor -DestinationPath (Join-Path $salida "LP-Profesor-$Version-windows.zip")
Get-ChildItem $salida
