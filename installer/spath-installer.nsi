; spath NSIS Installer Script
; Windows PATH Security Scanner and Fixer

!include "MUI2.nsh"
!include "FileFunc.nsh"

; General
Name "spath"
OutFile "spath-setup.exe"
InstallDir "$PROGRAMFILES\spath"
InstallDirRegKey HKLM "Software\spath" "InstallDir"
RequestExecutionLevel admin

; Version info
!define VERSION "0.1.0"
VIProductVersion "0.1.0.0"
VIAddVersionKey "ProductName" "spath"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "Windows PATH Security Scanner"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "MIT License"

; Interface Settings
!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; Languages
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Russian"
!insertmacro MUI_LANGUAGE "Japanese"

; Installer Section
Section "Install"
    SetOutPath "$INSTDIR"
    
    ; Copy files
    File "..\target\release\spath.exe"
    File "..\README.md"
    File "..\LICENSE"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"
    
    ; Add to PATH
    EnVar::AddValue "PATH" "$INSTDIR"
    
    ; Registry entries for Add/Remove Programs
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath" \
        "DisplayName" "spath - PATH Security Scanner"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath" \
        "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath" \
        "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath" \
        "DisplayIcon" "$INSTDIR\spath.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath" \
        "Publisher" "vremyavnikuda"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath" \
        "DisplayVersion" "${VERSION}"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath" \
        "NoModify" 1
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath" \
        "NoRepair" 1
    
    ; Get installed size
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath" \
        "EstimatedSize" "$0"
    
    ; Store install dir
    WriteRegStr HKLM "Software\spath" "InstallDir" "$INSTDIR"
SectionEnd

; Uninstaller Section
Section "Uninstall"
    ; Remove from PATH
    EnVar::DeleteValue "PATH" "$INSTDIR"
    
    ; Remove files
    Delete "$INSTDIR\spath.exe"
    Delete "$INSTDIR\README.md"
    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    
    ; Remove registry entries
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\spath"
    DeleteRegKey HKLM "Software\spath"
SectionEnd
