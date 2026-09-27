# Prepara runtime\ : CPython portátil oficial (paquete NuGet "python") + PyInstaller, para que la
# App Alumno genere ejecutables .exe sin internet. Se ejecuta en la máquina que empaqueta (CI).
param(
  [string]$Version = "3.13.7",
  [string]$Destino = "runtime"
)
$ErrorActionPreference = "Stop"
$tmp = Join-Path $env:TEMP "rlp-python-$Version"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
$paquete = Join-Path $tmp "python.zip"
Write-Host "Descargando CPython $Version (NuGet)…"
Invoke-WebRequest -Uri "https://www.nuget.org/api/v2/package/python/$Version" -OutFile $paquete
Expand-Archive -Path $paquete -DestinationPath (Join-Path $tmp "pkg") -Force
if (Test-Path $Destino) { Remove-Item -Recurse -Force $Destino }
Copy-Item -Recurse (Join-Path $tmp "pkg\tools") $Destino
$py = Join-Path $Destino "python.exe"
& $py -m ensurepip --upgrade
& $py -m pip install --no-warn-script-location --disable-pip-version-check "pyinstaller>=6.10"
& $py -c "import PyInstaller, sys; print('PyInstaller', PyInstaller.__version__, 'en Python', sys.version)"
# Limpieza para reducir tamaño.
Get-ChildItem -Recurse -Directory -Filter "__pycache__" $Destino | Remove-Item -Recurse -Force
Write-Host "runtime listo en $Destino"
