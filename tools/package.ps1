param(
  [string]$AppName   = "phastimer",
  [string]$MingwRoot = "C:\msys64\mingw64",
  [string]$Out       = "dist"
)

$ErrorActionPreference = "Stop"

$exe = Join-Path "target\release" "$AppName.exe"
$bin = Join-Path $MingwRoot "bin"
$share = Join-Path $MingwRoot "share"
$lib = Join-Path $MingwRoot "lib"

if (!(Test-Path $bin)) {
  throw "MSYS2 mingw64 not found at: $bin"
}

Write-Host "== Building release =="
cargo build --release | Out-Host

if (!(Test-Path $exe)) {
  throw "Release EXE not found: $exe"
}

Write-Host "== Creating $Out =="
Remove-Item -Recurse -Force $Out -ErrorAction SilentlyContinue
New-Item -ItemType Directory $Out | Out-Null

# ------------------------------------------------------------
# Copy EXE
# ------------------------------------------------------------
Copy-Item $exe $Out -Force

# ------------------------------------------------------------
# Copy ALL GTK / MinGW runtime DLLs (THIS IS THE KEY)
# ------------------------------------------------------------
Write-Host "== Copying ALL mingw64 DLLs =="
Copy-Item "$bin\*.dll" $Out -Force

# ------------------------------------------------------------
# GLib schemas (required)
# ------------------------------------------------------------
Write-Host "== Copying GLib schemas =="
$schemaOut = Join-Path $Out "share\glib-2.0\schemas"
New-Item -ItemType Directory $schemaOut -Force | Out-Null
Copy-Item (Join-Path $share "glib-2.0\schemas\*") $schemaOut -Recurse -Force

$glibCompile = Join-Path $bin "glib-compile-schemas.exe"
if (Test-Path $glibCompile) {
  & $glibCompile $schemaOut | Out-Null
}

# ------------------------------------------------------------
# gdk-pixbuf loaders + cache (images/icons)
# ------------------------------------------------------------
Write-Host "== Copying gdk-pixbuf loaders =="
$pixLoadersSrc = Join-Path $lib "gdk-pixbuf-2.0\2.10.0\loaders"
$pixLoadersOut = Join-Path $Out "lib\gdk-pixbuf-2.0\2.10.0\loaders"
New-Item -ItemType Directory $pixLoadersOut -Force | Out-Null
Copy-Item (Join-Path $pixLoadersSrc "*") $pixLoadersOut -Recurse -Force

Write-Host "== Generating gdk-pixbuf loaders.cache =="
$pixCacheOutDir = Join-Path $Out "share\gdk-pixbuf-2.0\2.10.0"
New-Item -ItemType Directory $pixCacheOutDir -Force | Out-Null
$cachePath = Join-Path $pixCacheOutDir "loaders.cache"

$pixQuery = Join-Path $bin "gdk-pixbuf-query-loaders.exe"
if (Test-Path $pixQuery) {
  & $pixQuery --update-cache --output=$cachePath | Out-Null
}

# ------------------------------------------------------------
# (Optional) Icon themes (uncomment if you use themed icons)
# ------------------------------------------------------------
# Write-Host "== Copying icon themes =="
# $iconsOut = Join-Path $Out "share\icons"
# New-Item -ItemType Directory $iconsOut -Force | Out-Null
# Copy-Item (Join-Path $share "icons\*") $iconsOut -Recurse -Force

Write-Host ""
Write-Host "== DONE =="
Write-Host "Portable build created at: $Out"
Write-Host ""
Write-Host "Test like a clean PC:"
Write-Host "  `$env:PATH='C:\Windows\System32;C:\Windows'"
Write-Host "  Start-Process -WorkingDirectory '$Out' -FilePath '$Out\$AppName.exe'"

& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" ".\phastimer.iss"
