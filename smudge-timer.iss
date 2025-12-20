; installer\smudge-timer.iss

#define MyAppName "Smudge Timer"
#define MyAppExeName "smudge-timer.exe"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Brad Eagle"
#define MyAppURL "https://example.com"  ; optional
#define MyDistDir "dist"

[Setup]
AppId={{D3D8F98A-3E5B-4E22-9F4D-0C8A4A4C3B2D}   ; generate a new GUID once and keep it stable
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=..\out
OutputBaseFilename=SmudgeTimer-Setup-{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

; If you have an icon, set it here
; SetupIconFile=..\assets\app.ico

UninstallDisplayIcon={app}\{#MyAppExeName}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop icon"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
; Copy EVERYTHING from dist into the install directory
Source: "{#MyDistDir}\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
; Launch after install
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent
