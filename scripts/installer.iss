; Inno Setup Script for MacKeys
; Zero-elevation Windows installer for Apple Magic Keyboard with Numeric Keypad (A1843)
#define MyAppName "MacKeys"
#define MyAppVersion "0.2.1"
#define MyAppPublisher "MacKeys Contributors"
#define MyAppURL "https://github.com/DooperDrew/mackeys-win"
#define MyAppExeName "magicbridge-settings.exe"
#define MyAgentExeName "magicbridge-agent.exe"

[Setup]
AppId={{B523AE21-0D34-4C2E-8E92-31FE66A487B2}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={localappdata}\Programs\MacKeys
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
; Zero Administrator elevation required (per-user install into %LOCALAPPDATA%)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\dist
OutputBaseFilename=MacKeys-v{#MyAppVersion}-Setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
CloseApplicationsFilter={#MyAgentExeName},{#MyAppExeName}
UninstallDisplayIcon={app}\{#MyAgentExeName}
VersionInfoVersion={#MyAppVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription=MacKeys Setup (Apple Magic Keyboard for Windows)

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "autostart"; Description: "Start MacKeys automatically at Windows sign-in"; GroupDescription: "Startup options:"

[Files]
Source: "..\target\release\{#MyAgentExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\crates\settings\locales\*"; DestDir: "{app}\locales"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\docs\user-guide.md"; DestDir: "{app}"; Flags: ignoreversion; DestName: "user-guide.md"
Source: "uninstall.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "setup-wizard.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\MacKeys Settings"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Comment: "Configure Apple Magic Keyboard mappings and profiles"
Name: "{group}\MacKeys Agent"; Filename: "{app}\{#MyAgentExeName}"; WorkingDir: "{app}"; Comment: "Background keyboard remap hook and battery monitor"
Name: "{group}\Uninstall MacKeys"; Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\uninstall.ps1"""; Comment: "Uninstall MacKeys"
Name: "{autodesktop}\MacKeys Settings"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon; Comment: "MacKeys Settings"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "MacKeys"; ValueData: """{app}\{#MyAgentExeName}"""; Tasks: autostart; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#MyAgentExeName}"; Description: "Start MacKeys background agent"; Flags: nowait postinstall skipifsilent
Filename: "{app}\{#MyAppExeName}"; Description: "Launch Initial Setup Wizard / Settings"; Flags: nowait postinstall skipifsilent checked

[UninstallDelete]
Type: files; Name: "{app}\*.log"
Type: dirifempty; Name: "{app}\locales"
Type: dirifempty; Name: "{app}"

[Code]
// Automatically terminate any running MacKeys instances prior to installation
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
begin
  Exec('taskkill.exe', '/F /IM magicbridge-agent.exe /IM magicbridge-settings.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := '';
end;

// Stop running instances prior to uninstallation
function InitializeUninstall(): Boolean;
var
  ResultCode: Integer;
begin
  Exec('taskkill.exe', '/F /IM magicbridge-agent.exe /IM magicbridge-settings.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := True;
end;
