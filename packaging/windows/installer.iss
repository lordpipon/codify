#define MyAppName "Codify"
#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif
#define MyAppPublisher "lordpipon"
#define MyAppExeName "codify.exe"

; Codify installer (.exe) — no logos, just the name.
[Setup]
AppId={{A1B2C3D4-0000-4F2E-9AC1-CODIFY0001}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\codify.ico
SetupIconFile=packaging\windows\codify.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
OutputDir=dist
OutputBaseFilename=Codify-Setup-{#MyAppVersion}
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "..\..\target\release\codify.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\packaging\windows\codify.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\packaging\windows\codify.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{cm:ProgramOnTheWeb,{#MyAppName}}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\codify.ico"
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\codify.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\codify.ico"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent