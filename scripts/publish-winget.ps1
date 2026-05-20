param(
    [Parameter(Mandatory = $false)]
    [string]$Version = "0.1.0",

    [Parameter(Mandatory = $false)]
    [string]$RepoRoot = (Resolve-Path "$PSScriptRoot/.."),

    [Parameter(Mandatory = $false)]
    [string]$WorkDir = "$env:TEMP\omnicode-winget-publish",

    [Parameter(Mandatory = $false)]
    [switch]$SkipRelease
)

$ErrorActionPreference = "Stop"

function Write-Step($msg) { Write-Host ">> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)  { Write-Host "   $msg" -ForegroundColor Green }
function Write-Err($msg) { Write-Host "   $msg" -ForegroundColor Red }

# ── Prerequisites ─────────────────────────────────────────────────────────────
if (-not (Get-Command "gh" -ErrorAction SilentlyContinue)) {
    Write-Err "GitHub CLI (gh) is required — install from https://cli.github.com/"
    exit 1
}

$token = $env:GITHUB_TOKEN
if (-not $token) {
    Write-Err "GITHUB_TOKEN not set. Use: `$env:GITHUB_TOKEN = 'ghp_...'"
    exit 1
}

Write-Step "Checked prerequisites: gh CLI + GITHUB_TOKEN"

# ── 1. Verify git tag exists ─────────────────────────────────────────────────
Push-Location $RepoRoot
try {
    $tag = "v$Version"
    $tagExists = git tag -l "$tag" | Select-String -SimpleMatch "$tag" -Quiet
    if (-not $tagExists) {
        Write-Err "Tag '$tag' not found. Create it first: git tag $tag && git push origin $tag"
        exit 1
    }
    Write-Ok "Tag $tag exists"
} finally { Pop-Location }

if (-not $SkipRelease) {
    # ── 2. Run packaging script ───────────────────────────────────────────────
    Write-Step "Building release package..."
    & "$PSScriptRoot/package.ps1" -Version $Version -RepoRoot $RepoRoot
    if (-not $?) { Write-Err "Packaging failed"; exit 1 }

    $releaseMeta = Get-Content "$RepoRoot/target/release-output.json" -Raw | ConvertFrom-Json

    # ── 3. Create GitHub Release ──────────────────────────────────────────────
    Write-Step "Creating GitHub Release for $tag ..."
    $ghRelease = gh release create "$tag" `
        "$($releaseMeta.path)" `
        --repo "bomboclati/OmniCode-" `
        --title "OmniCode v$Version" `
        --notes "## OmniCode v$Version`n`nSee [CHANGELOG](https://github.com/bomboclati/OmniCode-/blob/main/CHANGELOG.md) for details." `
        --draft 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Err "gh release create failed: $ghRelease"
        exit 1
    }
    Write-Ok "Release created: $ghRelease"
} else {
    Write-Step "Skipping GitHub Release (--SkipRelease), using cached metadata..."
    $releaseMeta = Get-Content "$RepoRoot/target/release-output.json" -Raw | ConvertFrom-Json
}

$sha256 = $releaseMeta.sha256
Write-Ok "Using SHA256: $sha256"

# ── 4. Prepare winget manifest ───────────────────────────────────────────────
Write-Step "Preparing winget manifest..."
$manifestPath = "$WorkDir/manifests/b/bomboclati/OmniCode/$Version/bomboclati.OmniCode.yaml"
$null = New-Item -ItemType Directory -Path (Split-Path $manifestPath -Parent) -Force

$manifest = Get-Content "$RepoRoot/scripts/winget/OmniCode.yaml" -Raw
$manifest = $manifest -replace "PLACEHOLDER_REPLACE_WITH_ACTUAL_SHA256", $sha256
$manifest = $manifest -replace 'InstallerUrl: .*', "InstallerUrl: https://github.com/bomboclati/OmniCode-/releases/download/$tag/OmniCode-$Version-windows-x86_64.zip"
$manifest | Set-Content $manifestPath -NoNewline
Write-Ok "Wrote manifest: $manifestPath"

# ── 5. Clone winget-pkgs and stage ──────────────────────────────────────────
Write-Step "Cloning microsoft/winget-pkgs (shallow)..."
$wingetRepo = "$WorkDir/winget-pkgs"
if (Test-Path $wingetRepo) {
    Push-Location $wingetRepo
    try { git pull 2>&1 | Out-Null } finally { Pop-Location }
} else {
    git clone --depth 1 "https://github.com/microsoft/winget-pkgs.git" $wingetRepo 2>&1
}
if (-not $?) { Write-Err "Failed to clone winget-pkgs"; exit 1 }

$wingetDest = "$wingetRepo/manifests/b/bomboclati/OmniCode/$Version/"
$null = New-Item -ItemType Directory -Path $wingetDest -Force
Copy-Item $manifestPath $wingetDest -Force
Write-Ok "Staged manifest in winget-pkgs"

# ── 6. Instructions for PR ───────────────────────────────────────────────────
Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║  Ready to submit to winget-pkgs                                 ║" -ForegroundColor Green
Write-Host "╠══════════════════════════════════════════════════════════════════╣" -ForegroundColor Green
Write-Host "║  Manifest: $wingetDest" -ForegroundColor Green
Write-Host "║  SHA256:   $sha256" -ForegroundColor Green
Write-Host "║                                                                    ║" -ForegroundColor Green
Write-Host "║  To submit:                                                       ║" -ForegroundColor Green
Write-Host "║    1. cd $wingetRepo" -ForegroundColor Green
Write-Host "║    2. git checkout -b bomboclati/OmniCode/v$Version" -ForegroundColor Green
Write-Host "║    3. git add manifests/b/bomboclati/OmniCode/" -ForegroundColor Green
Write-Host "║    4. git commit -m 'New version: bomboclati.OmniCode v$Version'" -ForegroundColor Green
Write-Host "║    5. gh pr create --repo microsoft/winget-pkgs --title 'New version: bomboclati.OmniCode v$Version' --body 'Adding version $Version of bomboclati.OmniCode'" -ForegroundColor Green
Write-Host "╚══════════════════════════════════════════════════════════════════╝" -ForegroundColor Green
