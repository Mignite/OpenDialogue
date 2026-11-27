# Empaqueta el release de Windows con el libmpv **LGPL**, no con el DLL GPL de
# desarrollo. Verifica el SHA256 del DLL antes de meterlo en el paquete: si el
# ZIP oficial cambia, el script falla en vez de distribuir un binario distinto
# del que revisamos.
#
#   cargo build --release -p rod-app
#   pwsh packaging/windows/empaquetar.ps1
#
# Opciones: -MpvZip <zip local> para no descargar, -Out <zip de salida>.

param(
    [string]$ReleaseDir,
    [string]$Out,
    [string]$MpvZip,
    [switch]$ConservarStaging
)

$ErrorActionPreference = 'Stop'

$raiz = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not $ReleaseDir) {
    $ReleaseDir = Join-Path $raiz 'target\release'
}
if (-not $Out) {
    $Out = Join-Path $ReleaseDir 'RustyOpenDialogue-0.1.0-win-x64.zip'
}

$linea = (Get-Content (Join-Path $PSScriptRoot 'libmpv-lgpl.sha256') -TotalCount 1).Trim()
$partes = $linea -split '\s+'
$esperado = $partes[0].ToLowerInvariant()
$url = 'https://github.com/mpv-player/mpv/releases/download/v0.42.0/libmpv-v0.42.0-x86_64-w64-mingw32-lgpl.zip'
$destino = Join-Path ([System.IO.Path]::GetTempPath()) 'rod-paquete'
New-Item -ItemType Directory -Path $destino -Force | Out-Null

if (-not $MpvZip) {
    $MpvZip = Join-Path $destino 'libmpv-v0.42.0-lgpl.zip'
    if (-not (Test-Path $MpvZip)) {
        New-Item -ItemType Directory -Path $destino -Force | Out-Null
        Write-Host "Descargando $url"
        Invoke-WebRequest -Uri $url -OutFile $MpvZip
    }
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
$archivo = [System.IO.Compression.ZipFile]::OpenRead($MpvZip)
$dll = Join-Path $destino 'libmpv-2.dll'
$licencia = Join-Path $destino 'LICENSE-libmpv-LGPL-2.1.txt'
try {
    foreach ($entrada in $archivo.Entries) {
        if ($entrada.Name -eq 'libmpv-2.dll') {
            [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entrada, $dll, $true)
        }
        if ($entrada.Name -eq 'LICENSE.LGPL') {
            [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entrada, $licencia, $true)
        }
    }
} finally {
    $archivo.Dispose()
}
if (-not (Test-Path $dll) -or -not (Test-Path $licencia)) {
    throw 'El ZIP oficial no trae libmpv-2.dll y LICENSE.LGPL'
}
$obtenido = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLowerInvariant()
if ($obtenido -ne $esperado) {
    throw "SHA256 de libmpv-2.dll: $obtenido (esperado $esperado)"
}
Write-Host "libmpv LGPL verificado: $obtenido"

$exe = Join-Path $ReleaseDir 'rod-app.exe'
if (-not (Test-Path $exe)) {
    throw "Falta $exe. Compila primero: cargo build --release -p rod-app"
}

$staging = Join-Path $destino 'staging'
if (Test-Path $staging) {
    Remove-Item $staging -Recurse -Force
}
New-Item -ItemType Directory -Path $staging -Force | Out-Null
Copy-Item $exe $staging
# La DLL GPL que puso build.rs se descarta: va la LGPL verificada.
Get-ChildItem $ReleaseDir -Filter '*.dll' |
    Where-Object { $_.Name -ne 'libmpv-2.dll' } |
    Copy-Item -Destination $staging
Copy-Item $dll (Join-Path $staging 'libmpv-2.dll')
Copy-Item $licencia (Join-Path $staging 'LICENSE-libmpv-LGPL-2.1.txt')
Copy-Item (Join-Path $PSScriptRoot 'THIRD_PARTY_NOTICES.md') $staging

New-Item -ItemType Directory -Path (Join-Path $staging 'testdata') -Force | Out-Null
Copy-Item (Join-Path $raiz 'testdata\clip-15s-2audio.mp4') (Join-Path $staging 'testdata')

@'
Rusty Open Dialogue 0.1.0 (Windows x64)

Doble clic en rod-app.exe. No necesita instalar mpv ni tener vcpkg en el PATH:
todas las DLLs van en esta carpeta. Al abrir, el proyecto de ejemplo apunta a
testdata\clip-15s-2audio.mp4, incluido aquí para probar video, subtítulos y
exportación sin descargar nada.

Este paquete usa libmpv LGPL (mpv v0.42.0). Lee THIRD_PARTY_NOTICES.md y
LICENSE-libmpv-LGPL-2.1.txt antes de redistribuirlo.
'@ | Set-Content -Encoding UTF8 (Join-Path $staging 'LEEME.txt')

if (Test-Path $Out) {
    Remove-Item $Out -Force
}
Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $Out -CompressionLevel Optimal
Write-Host "staging: $staging"
if (-not $ConservarStaging) {
    Remove-Item $staging -Recurse -Force
}

$info = Get-Item $Out
Write-Host "paquete: $($info.FullName)"
Write-Host "tamano: $([math]::Round($info.Length / 1MB, 1)) MB"
