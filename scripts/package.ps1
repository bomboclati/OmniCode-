param(
    [Parameter(Mandatory = $false)]
    [string]$Version = "0.1.0",

    [Parameter(Mandatory = $false)]
    [string]$RepoRoot = (Resolve-Path "$PSScriptRoot/..")
)

$ErrorActionPreference = "Stop"

function Write-Step($msg) { Write-Host ">> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)  { Write-Host "   $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "   $msg" -ForegroundColor Yellow }

# ── Validate ──────────────────────────────────────────────────────────────────
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Warn "Rust toolchain not found — install from https://rustup.rs"
    exit 1
}

# ── Build release binary ──────────────────────────────────────────────────────
Write-Step "Building omnicode v$Version (release)..."
Push-Location $RepoRoot
try {
    cargo build --release
    if (-not $?) { throw "cargo build failed" }
} finally {
    Pop-Location
}

$binDir = "$RepoRoot/target/release"
$bin = "$binDir/omnicode.exe"
if (-not (Test-Path $bin)) {
    throw "Binary not found at $bin — build may have failed"
}
Write-Ok "Built: $bin"

# ── Stage files for packaging ─────────────────────────────────────────────────
Write-Step "Staging files..."
$stage = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
try {
    Copy-Item $bin "$stage/omnicode.exe"

    $assetDirs = @("assets")
    foreach ($d in $assetDirs) {
        $src = "$RepoRoot/$d"
        if (Test-Path $src) {
            Copy-Item -Recurse $src "$stage/$d"
            Write-Ok "Included: $d/"
        }
    }

    # ── Create ZIP ────────────────────────────────────────────────────────────
    $zipName = "OmniCode-$Version-windows-x86_64.zip"
    $zipPath = "$RepoRoot/target/$zipName"
    Write-Step "Creating archive: $zipName ..."

    if (Test-Path $zipPath) { Remove-Item $zipPath }

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory($stage, $zipPath)

    Write-Ok "Created: $zipPath"

    # ── Compute SHA256 ────────────────────────────────────────────────────────
    $hash = (Get-FileHash $zipPath -Algorithm SHA256).Hash.ToLower()
    Write-Ok "SHA256: $hash"

    # ── Output for CI consumption ─────────────────────────────────────────────
    $output = @{
        version   = $Version
        archive   = $zipName
        path      = $zipPath
        sha256    = $hash
        size      = (Get-Item $zipPath).Length
    }
    $output | ConvertTo-Json -Compress | Set-Content "$RepoRoot/target/release-output.json"
    Write-Ok "Release metadata: target/release-output.json"

    Write-Host ""
    Write-Host "╔══════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║  Package ready for winget                           ║" -ForegroundColor Green
    Write-Host "╠══════════════════════════════════════════════════════╣" -ForegroundColor Green
    Write-Host "║  Archive: $zipName" -ForegroundColor Green
    Write-Host "║  SHA256:  $hash" -ForegroundColor Green
    Write-Host "║  Size:    $($output.size) bytes" -ForegroundColor Green
    Write-Host "╚══════════════════════════════════════════════════════╝" -ForegroundColor Green
} finally {
    Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
}
