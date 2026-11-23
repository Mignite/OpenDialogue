; Instalador de Rusty Open Dialogue para Windows x64.
;
; Se compila sobre el staging que deja empaquetar.ps1(-ConservarStaging):
;
;   makensis /DSTAGING=<staging> /DOUTFILE=<setup.exe> packaging\windows\instalar.nsi
;
; Instalación por usuario (sin pedir administrador), igual que el NSIS del
; oráculo (`installMode: currentUser`) y con la licencia LGPL visible.

!ifndef STAGING
  !error "Falta /DSTAGING=<carpeta con rod-app.exe y las DLLs>"
!endif
!ifndef OUTFILE
  !define OUTFILE "RustyOpenDialogue-0.1.0-win-x64-setup.exe"
!endif

Name "Rusty Open Dialogue 0.1.0"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\RustyOpenDialogue"
RequestExecutionLevel user
SetCompressor /SOLID lzma
ShowInstDetails show
LicenseData "${STAGING}\LICENSE-libmpv-LGPL-2.1.txt"

Section "Rusty Open Dialogue"
  SetOutPath "$INSTDIR"
  File "${STAGING}\rod-app.exe"
  File "${STAGING}\*.dll"
  File "${STAGING}\*.txt"
  File "${STAGING}\*.md"
  SetOutPath "$INSTDIR\testdata"
  File "${STAGING}\testdata\clip-15s-2audio.mp4"
  SetOutPath "$INSTDIR"

  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\Rusty Open Dialogue"
  CreateShortcut "$SMPROGRAMS\Rusty Open Dialogue\Rusty Open Dialogue.lnk" "$INSTDIR\rod-app.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\rod-app.exe"
  Delete "$INSTDIR\*.dll"
  Delete "$INSTDIR\*.txt"
  Delete "$INSTDIR\*.md"
  Delete "$INSTDIR\Uninstall.exe"
  Delete "$INSTDIR\testdata\clip-15s-2audio.mp4"
  RMDir "$INSTDIR\testdata"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\Rusty Open Dialogue\Rusty Open Dialogue.lnk"
  RMDir "$SMPROGRAMS\Rusty Open Dialogue"
SectionEnd
