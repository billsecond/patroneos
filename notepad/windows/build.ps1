<#
Builds Patrone Notepad for Windows from this checkout, and packs it with
the GTK runtime it needs and PatroneOS's fonts into
out\windows\patrone-notepad-VERSION-windows-x86_64.zip, which installs on
any 64-bit Windows 10 or 11 PC without administrator rights (install.ps1
in the zip).

  powershell -ExecutionPolicy Bypass -File notepad\windows\build.ps1 [-Msys2 FOLDER] [-SkipPackages]

It needs Rust from rustup, and MSYS2 (https://www.msys2.org) in -Msys2
(by default %LOCALAPPDATA%\msys64). Unless -SkipPackages, it first installs
MSYS2's GTK 4, compiler and icon packages and Rust's
stable-x86_64-pc-windows-gnu toolchain. Nothing is installed for the
system: MSYS2's packages stay in its folder, Rust's in %USERPROFILE%\.rustup.
#>
param(
    [string]$Msys2 = "$env:LOCALAPPDATA\msys64",
    [switch]$SkipPackages
)
$ErrorActionPreference = 'Stop'

$root = (Resolve-Path "$PSScriptRoot\..\..").Path
$notepad = Join-Path $root 'notepad'
$version = (Select-String -Path "$notepad\Cargo.toml" -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$mingw = Join-Path $Msys2 'mingw64'
$bash = Join-Path $Msys2 'usr\bin\bash.exe'
if (-not (Test-Path $bash)) {
    throw "MSYS2 isn't in $Msys2 (notepad\README.md, `"Windows`", says how to set it up)"
}

# A command in MSYS2's MINGW64 environment, in this folder
function Invoke-Msys2([string]$Command) {
    $env:MSYSTEM = 'MINGW64'
    $env:CHERE_INVOKING = '1'
    & $bash -lc $Command
    if ($LASTEXITCODE -ne 0) { throw "MSYS2: '$Command' failed" }
}

if (-not $SkipPackages) {
    Invoke-Msys2 ('pacman -S --needed --noconfirm mingw-w64-x86_64-gtk4 mingw-w64-x86_64-gcc ' +
        'mingw-w64-x86_64-pkgconf mingw-w64-x86_64-librsvg mingw-w64-x86_64-adwaita-icon-theme ' +
        'mingw-w64-x86_64-ntldd')
    rustup toolchain install stable-x86_64-pc-windows-gnu --profile minimal
    if ($LASTEXITCODE -ne 0) { throw 'rustup failed' }
}

# --- Build ---------------------------------------------------------------------
# MSYS2's MINGW64 libraries are built against msvcrt, like Rust's
# x86_64-pc-windows-gnu target, which links them with MSYS2's own gcc
$env:PATH = "$mingw\bin;$env:PATH"
$env:PKG_CONFIG_PATH = "$mingw\lib\pkgconfig"
$env:CARGO_TARGET_DIR = Join-Path $notepad 'target\windows'
Push-Location $notepad
try {
    cargo +stable-x86_64-pc-windows-gnu build --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
} finally {
    Pop-Location
}
$exe = Join-Path $env:CARGO_TARGET_DIR 'release\patrone-notepad.exe'

# --- The app's folder ----------------------------------------------------------
# Laid out like MSYS2's own prefix (bin, lib, share), which is where GLib,
# GTK and gdk-pixbuf look for their schemas, icons and image loaders when
# they run from somewhere else
$out = Join-Path $root 'out\windows'
$stage = Join-Path $out 'Patrone Notepad'
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
foreach ($folder in 'bin', 'lib', 'share\glib-2.0\schemas', 'share\icons', 'share\fonts\patroneos', 'share\licenses') {
    New-Item -ItemType Directory -Force (Join-Path $stage $folder) | Out-Null
}
Copy-Item $exe "$stage\bin\"
# GLib's session bus helper: what keeps Notepad one process, so opening a
# file while it runs opens a tab in its window
Copy-Item "$mingw\bin\gdbus.exe" "$stage\bin\"

# The image loaders (SVG for the icons, and images in Markdown), and their
# list with paths relative to the folder
Copy-Item -Recurse "$mingw\lib\gdk-pixbuf-2.0" "$stage\lib\"
$cache = Get-ChildItem -Recurse "$stage\lib\gdk-pixbuf-2.0" -Filter loaders.cache | Select-Object -First 1
if ($cache) {
    $prefix = [regex]::Escape(($mingw -replace '\\', '/') + '/')
    $text = (Get-Content -Raw $cache.FullName) -replace "(?i)`"$prefix", '"'
    $text = $text -replace '(?i)"[a-z]:[\\/][^"]*?[\\/]mingw64[\\/]', '"'
    Set-Content -NoNewline -Encoding utf8 $cache.FullName $text
}

# Every DLL those need, from MSYS2
$binaries = @("$stage\bin\patrone-notepad.exe", "$stage\bin\gdbus.exe") +
    (Get-ChildItem -Recurse "$stage\lib\gdk-pixbuf-2.0" -Filter *.dll).FullName
$mingwBin = "$mingw\bin\".ToLower()
$copied = @{}
foreach ($binary in $binaries) {
    foreach ($line in & "$mingw\bin\ntldd.exe" -R $binary) {
        if ($line -match '=>\s+(\S+\.dll)') {
            $dll = $Matches[1] -replace '/', '\'
            if ($dll.ToLower().StartsWith($mingwBin) -and -not $copied.ContainsKey($dll)) {
                Copy-Item $dll "$stage\bin\" -Force
                $copied[$dll] = $true
            }
        }
    }
}

# GTK's settings schemas (the file chooser's, the color chooser's)
Copy-Item "$mingw\share\glib-2.0\schemas\gschemas.compiled" "$stage\share\glib-2.0\schemas\"

# Icons: GNOME's Adwaita, for the buttons and menus, and Notepad's own
Copy-Item -Recurse "$mingw\share\icons\Adwaita" "$stage\share\icons\"
Copy-Item -Recurse "$mingw\share\icons\hicolor" "$stage\share\icons\"
$appIcons = "$stage\share\icons\hicolor\scalable\apps"
New-Item -ItemType Directory -Force $appIcons | Out-Null
Copy-Item "$notepad\data\org.patrone.Notepad.svg" $appIcons

# The Start menu's icon: the SVG at each size, in one .ico (PNG entries)
$sizes = 16, 24, 32, 48, 64, 128, 256
$pngs = foreach ($size in $sizes) {
    $png = Join-Path $out "icon-$size.png"
    & "$mingw\bin\rsvg-convert.exe" -w $size -h $size -o $png "$notepad\data\org.patrone.Notepad.svg"
    if ($LASTEXITCODE -ne 0) { throw 'rsvg-convert failed' }
    [IO.File]::ReadAllBytes($png)
    Remove-Item $png
}
$ico = New-Object IO.MemoryStream
$writer = New-Object IO.BinaryWriter $ico
$writer.Write([uint16]0); $writer.Write([uint16]1); $writer.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $edge = if ($sizes[$i] -ge 256) { 0 } else { $sizes[$i] }
    $writer.Write([byte]$edge); $writer.Write([byte]$edge); $writer.Write([byte]0); $writer.Write([byte]0)
    $writer.Write([uint16]1); $writer.Write([uint16]32)
    $writer.Write([uint32]$pngs[$i].Length); $writer.Write([uint32]$offset)
    $offset += $pngs[$i].Length
}
foreach ($png in $pngs) { $writer.Write($png) }
[IO.File]::WriteAllBytes("$stage\patrone-notepad.ico", $ico.ToArray())

# PatroneOS's fonts: the release fonts-patroneos installs (fonts\release.txt),
# checked against its pinned checksum. Notepad loads them for itself; they
# aren't installed in Windows.
$release = @{}
foreach ($line in Get-Content "$root\fonts\release.txt") {
    if ($line -match '^(\w+)=(.+)$') { $release[$Matches[1]] = $Matches[2] }
}
$zip = Join-Path $out "PatroneOS-Fonts-$($release.version).zip"
if (-not (Test-Path $zip) -or (Get-FileHash $zip -Algorithm SHA256).Hash -ne $release.sha256.ToUpper()) {
    Invoke-WebRequest -Uri $release.url -OutFile $zip -UseBasicParsing
}
if ((Get-FileHash $zip -Algorithm SHA256).Hash -ne $release.sha256.ToUpper()) {
    throw "$($release.url) isn't the release fonts\release.txt pins (its SHA-256 differs)"
}
$fonts = Join-Path $out 'fonts'
if (Test-Path $fonts) { Remove-Item -Recurse -Force $fonts }
Expand-Archive $zip $fonts
Get-ChildItem -Recurse $fonts -Filter 'PatroneOS*.ttf' | Copy-Item -Destination "$stage\share\fonts\patroneos\"
Get-ChildItem -Recurse $fonts -Include OFL.txt, FONTLOG.txt | Copy-Item -Destination "$stage\share\fonts\patroneos\"
Remove-Item -Recurse -Force $fonts

# --- Licences ------------------------------------------------------------------
# Notepad's (GPL-3.0-or-later), its Rust crates', and each MSYS2 package
# whose files are in the folder, with where their source is
Copy-Item "$root\LICENSE" "$stage\share\licenses\patrone-notepad-LICENSE.txt"
Push-Location $notepad
try {
    cargo +stable-x86_64-pc-windows-gnu vendor --locked --quiet "$out\vendor" | Out-Null
} finally {
    Pop-Location
}
Invoke-Msys2 "bash '$(($root -replace '\\', '/'))/packaging/third-party-licenses.sh' '$(($out -replace '\\', '/'))/vendor' > '$(($stage -replace '\\', '/'))/share/licenses/rust-crates.txt'"
Remove-Item -Recurse -Force "$out\vendor"
$packages = @{}
foreach ($dll in @($copied.Keys) + @("$mingw\bin\gdbus.exe")) {
    $owner = & $bash -lc "pacman -Qqo '$($dll -replace '\\', '/')' 2>/dev/null"
    if ($owner) { $packages[$owner.Trim()] = $true }
}
foreach ($package in 'mingw-w64-x86_64-gtk4', 'mingw-w64-x86_64-gdk-pixbuf2', 'mingw-w64-x86_64-librsvg', 'mingw-w64-x86_64-adwaita-icon-theme', 'mingw-w64-x86_64-glib2') {
    $packages[$package] = $true
}
$sources = @('The GTK runtime in this folder is from MSYS2 (https://www.msys2.org), unchanged.',
    'Each package''s source is at https://packages.msys2.org/base, and its licence is in this folder.', '')
foreach ($package in ($packages.Keys | Sort-Object)) {
    $info = & $bash -lc "pacman -Qi $package 2>/dev/null"
    $versionLine = ($info | Select-String '^Version\s*:\s*(.+)$').Matches
    $licenceLine = ($info | Select-String '^Licenses\s*:\s*(.+)$').Matches
    $sources += "$package $(if ($versionLine) { $versionLine[0].Groups[1].Value }): $(if ($licenceLine) { $licenceLine[0].Groups[1].Value })"
    $name = $package -replace '^mingw-w64-x86_64-', ''
    $licences = "$mingw\share\licenses\$name"
    if (Test-Path $licences) { Copy-Item -Recurse $licences "$stage\share\licenses\$name" }
}
Set-Content -Encoding utf8 "$stage\share\licenses\SOURCES.txt" $sources

# --- The zip -------------------------------------------------------------------
$name = "patrone-notepad-$version-windows-x86_64"
$pack = Join-Path $out $name
if (Test-Path $pack) { Remove-Item -Recurse -Force $pack }
New-Item -ItemType Directory -Force $pack | Out-Null
Move-Item $stage "$pack\Patrone Notepad"
Copy-Item "$PSScriptRoot\install.ps1", "$PSScriptRoot\uninstall.ps1", "$PSScriptRoot\README.txt" $pack
$zipFile = "$pack.zip"
if (Test-Path $zipFile) { Remove-Item -Force $zipFile }
Compress-Archive -Path "$pack\*" -DestinationPath $zipFile
Get-Item $zipFile | Select-Object FullName, Length
