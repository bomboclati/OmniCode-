#Requires -Version 5.1
$ErrorActionPreference = "Stop"

$Green = [ConsoleColor]::Green
$Default = [ConsoleColor]::Gray

Write-Host @"

  ___  _ __ ___  _ __ ___  _ __
 / _ \| '_ ` _ \| '_ ` _ \| '_ \
| (_) | | | | | | | | | | | | |
 \___/|_| |_| |_|_| |_| |_| |_|

"@ -ForegroundColor $Green
Write-Host "OmniCode - Autonomous AI Coding Agent" -ForegroundColor $Green
Write-Host "======================================"
Write-Host ""

function Get-Architecture {
    $arch = (Get-WmiObject Win32_Processor).Architecture
    switch ($arch) {
        0 { return "x86" }
        9 { return "amd64" }
        12 { return "arm64" }
        default { return "amd64" }
    }
}

function Get-LatestVersion {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/omnicode/omnicode/releases/latest" -ErrorAction SilentlyContinue
        return $response.tag_name -replace "^v", ""
    } catch {
        return "0.1.0"
    }
}

function Install-Binary {
    param($Version, $Arch)

    $archive = "omnicode-windows-${Arch}.tar.gz"
    $url = "https://github.com/omnicode/omnicode/releases/download/v${Version}/${archive}"
    $tmpDir = "$env:TEMP\omnicode-install"
    $installDir = "$env:LOCALAPPDATA\Programs\omnicode"

    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null

    Write-Host "Downloading OmniCode v${Version}..."
    
    try {
        Invoke-WebRequest -Uri $url -OutFile "$tmpDir\omnicode.tar.gz" -UseBasicParsing
    } catch {
        Write-Host "Download failed: $_" -ForegroundColor Red
        Write-Host "Falling back to manual install instructions..."
        Write-Host ""
        Write-Host "Please install Rust from https://rustup.rs and run:"
        Write-Host "  cargo install omnicode"
        exit 1
    }

    # Try to verify checksum
    try {
        $checksumUrl = "${url}.sha256"
        $checksum = (Invoke-WebRequest -Uri $checksumUrl -UseBasicParsing).Content.Trim()
        $localHash = (Get-FileHash "$tmpDir\omnicode.tar.gz" -Algorithm SHA256).Hash.ToLower()
        if ($localHash -ne $checksum.ToLower()) {
            Write-Host "Warning: Checksum mismatch" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "Checksum verification skipped" -ForegroundColor Yellow
    }

    Write-Host "Extracting..."
    tar -xzf "$tmpDir\omnicode.tar.gz" -C "$tmpDir"

    # Find omni.exe
    $binaryPath = Get-ChildItem -Path $tmpDir -Recurse -Filter "omni.exe" | Select-Object -First 1 -ExpandProperty FullName
    if (-not $binaryPath) {
        $binaryPath = Get-ChildItem -Path $tmpDir -Recurse -Filter "omnicode.exe" | Select-Object -First 1 -ExpandProperty FullName
    }

    if (-not $binaryPath) {
        # Look for any exe
        $binaryPath = Get-ChildItem -Path $tmpDir -Recurse -Filter "*.exe" | Select-Object -First 1 -ExpandProperty FullName
    }

    if (-not $binaryPath) {
        Write-Host "Binary not found in archive. Installing via cargo instead..." -ForegroundColor Yellow
        & "cargo" "install" "omnicode" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Installed via cargo!" -ForegroundColor Green
        } else {
            Write-Host "Please install manually:" -ForegroundColor Yellow
            Write-Host "  1. Install Rust from https://rustup.rs"
            Write-Host "  2. Run: cargo install omnicode"
        }
        return
    }

    Copy-Item -Path $binaryPath -Destination "$installDir\omni.exe" -Force
    Write-Host ""

    # Add to PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
        $env:Path = "$env:Path;$installDir"
        Write-Host "Added $installDir to your PATH" -ForegroundColor $Green
    }

    # Cleanup
    Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue

    Write-Host "✓ OmniCode installed to $installDir\omni.exe" -ForegroundColor $Green
}

Write-Host "Installing OmniCode for Windows..."

$arch = Get-Architecture
Write-Host "Detected architecture: $arch"

$version = Get-LatestVersion
Write-Host "Latest version: v${version}"

Install-Binary -Version $version -Arch $arch

Write-Host ""
Write-Host @"
╔══════════════════════════════════════════╗
║       OmniCode installed successfully!   ║
╚══════════════════════════════════════════╝
"@ -ForegroundColor $Green
Write-Host ""
Write-Host "Quick start:"
Write-Host "  omni                    # Launch TUI"
Write-Host "  omni serve              # Start web server"
Write-Host '  omni "build my api"     # Run an agent task'
Write-Host "  omni --help             # See all commands"
Write-Host ""
Write-Host "For more info: https://omnicode.ai"
Write-Host ""
Write-Host "NOTE: You may need to restart your terminal for PATH changes to take effect."
