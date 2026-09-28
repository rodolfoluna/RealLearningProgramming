# Arma las carpetas portables y sus .zip a partir de los ejecutables compilados:
#   dist-portable\RLP-Alumno\   (RLP Alumno.exe, runtime\, config\, LEEME.txt)
#   dist-portable\RLP-Profesor\ (RLP Profesor.exe, LEEME.txt, INSTALACION.md)
param([string]$Version = "0.2.0")
$ErrorActionPreference = "Stop"
$salida = "dist-portable"
if (Test-Path $salida) { Remove-Item -Recurse -Force $salida }
$alumno = Join-Path $salida "RLP-Alumno"
$profesor = Join-Path $salida "RLP-Profesor"
New-Item -ItemType Directory -Force -Path (Join-Path $alumno "config"), $profesor | Out-Null
Copy-Item "target\release\rlp-alumno.exe" (Join-Path $alumno "RLP Alumno.exe")
Copy-Item -Recurse "runtime" (Join-Path $alumno "runtime")
Copy-Item "docs\LEEME-alumno.txt" (Join-Path $alumno "LEEME.txt")
Copy-Item "target\release\rlp-profesor.exe" (Join-Path $profesor "RLP Profesor.exe")
Copy-Item "docs\LEEME-profesor.txt" (Join-Path $profesor "LEEME.txt")
Copy-Item "docs\INSTALACION.md" (Join-Path $profesor "INSTALACION.md")
Compress-Archive -Path $alumno -DestinationPath (Join-Path $salida "RLP-Alumno-$Version-windows.zip")
Compress-Archive -Path $profesor -DestinationPath (Join-Path $salida "RLP-Profesor-$Version-windows.zip")
Get-ChildItem $salida
