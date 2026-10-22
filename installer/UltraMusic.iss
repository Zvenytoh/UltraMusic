#ifndef AppVersion
  #define AppVersion "0.2.0"
#endif

[Setup]
AppId={{E7C85E87-E68A-46B7-9FF5-5353B638756A}
AppName=UltraMusic
AppVersion={#AppVersion}
AppPublisher=Zvenytoh
AppPublisherURL=https://github.com/Zvenytoh/UltraMusic
AppSupportURL=https://github.com/Zvenytoh/UltraMusic/issues
AppUpdatesURL=https://github.com/Zvenytoh/UltraMusic/releases
DefaultDirName={localappdata}\Programs\UltraMusic
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.22000
OutputDir=..\target\distribution
OutputBaseFilename=UltraMusic-Setup
SetupIconFile=..\assets\ultramusic.ico
UninstallDisplayIcon={app}\UltraMusic.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
AppMutex=Local\UltraMusic.Native.1
CloseApplications=yes
RestartApplications=no

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked

[Files]
Source: "..\target\release\ultramusic.exe"; DestDir: "{app}"; DestName: "UltraMusic.exe"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\dependencies\blocker\*"; DestDir: "{app}\blocker"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\target\dependencies\MicrosoftEdgeWebview2Setup.exe"; Flags: dontcopy

[Icons]
Name: "{autoprograms}\UltraMusic"; Filename: "{app}\UltraMusic.exe"
Name: "{autodesktop}\UltraMusic"; Filename: "{app}\UltraMusic.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\UltraMusic.exe"; Description: "Launch UltraMusic"; Flags: nowait postinstall skipifsilent

[Code]
procedure InitializeWizard;
var
  Directory, Drive: String;
begin
  { Updaters before 0.2.4 pass Rust's canonical \\?\ drive path to /DIR.
    Repair that legacy input before Setup validates the hidden directory page.
    Only local drive prefixes are converted; other invalid paths stay invalid. }
  Directory := WizardForm.DirEdit.Text;
  Drive := Uppercase(Copy(Directory, 5, 1));
  if (Copy(Directory, 1, 4) = '\\?\') and
    (Length(Drive) = 1) and (Pos(Drive, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ') > 0) and
    (Copy(Directory, 6, 2) = ':\') then
  begin
    Delete(Directory, 1, 4);
    WizardForm.DirEdit.Text := Directory;
    Log('Normalized legacy updater directory to an ordinary local drive path.');
  end;
end;

function HasRuntimeAt(Root: Integer; Key: String): Boolean;
var
  Version: String;
begin
  Result := RegQueryStringValue(Root, Key, 'pv', Version) and
    (Version <> '') and (Version <> '0.0.0.0');
end;

function HasWebView2: Boolean;
var
  Key: String;
begin
  Key := 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';
  Result := HasRuntimeAt(HKCU32, Key) or HasRuntimeAt(HKCU64, Key) or
    HasRuntimeAt(HKLM32, Key) or HasRuntimeAt(HKLM64, Key);
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ExitCode: Integer;
begin
  Result := '';
  if not HasWebView2 then
  begin
    ExtractTemporaryFile('MicrosoftEdgeWebview2Setup.exe');
    if not Exec(ExpandConstant('{tmp}\MicrosoftEdgeWebview2Setup.exe'),
      '/silent /install', '', SW_HIDE, ewWaitUntilTerminated, ExitCode) then
      Result := 'Microsoft Edge WebView2 could not be installed. Check your Internet connection and try again.'
    else if not HasWebView2 then
      Result := 'Microsoft Edge WebView2 is required. Install the Evergreen runtime and run UltraMusic Setup again.';
  end;
end;
