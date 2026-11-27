# Captura la ventana del editor abierta por `cargo run -p rod-app --example captura-editor`.
# Uso: powershell -NoProfile -ExecutionPolicy Bypass -File capturar.ps1 -Nombre antes-editor-1440
param(
    [string]$Nombre = "editor",
    [int]$Ancho = 1440,
    [int]$Alto = 900
)

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class CapWin {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int ht, bool repaint);
}
"@

$proc = Get-Process | Where-Object { $_.MainWindowTitle -like "*OpenDialogue (capturas)*" } |
    Select-Object -First 1
if (-not $proc) { throw "No hay ventana del editor de capturas abierta." }

[CapWin]::MoveWindow($proc.MainWindowHandle, 0, 0, $Ancho, $Alto, $true) | Out-Null
[CapWin]::SetForegroundWindow($proc.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 900

$rect = New-Object CapWin+RECT
[CapWin]::GetWindowRect($proc.MainWindowHandle, [ref]$rect) | Out-Null
$anchoReal = $rect.Right - $rect.Left
$altoReal = $rect.Bottom - $rect.Top
$bmp = New-Object System.Drawing.Bitmap $anchoReal, $altoReal
$gfx = [System.Drawing.Graphics]::FromImage($bmp)
$gfx.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bmp.Size)

$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ruta = Join-Path $dir "$Nombre.png"
$bmp.Save($ruta, [System.Drawing.Imaging.ImageFormat]::Png)
$gfx.Dispose()
$bmp.Dispose()
Write-Output "$ruta ($anchoReal x $altoReal)"
