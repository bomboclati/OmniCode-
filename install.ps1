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
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/bomboclati/OmniCode-/releases/latest" -ErrorAction SilentlyContinue
        return $response.tag_name -replace "^v", ""
    } catch {
        return "0.1.0"
    }
}

function Install-Binary {
    param($Version, $Arch)

    $archive = "omni-${Version}-windows-x86_64.zip"
    $url = "https://github.com/bomboclati/OmniCode-/releases/download/v${Version}/${archive}"
    $tmpDir = "$env:TEMP\omni-install"
    $installDir = "$env:LOCALAPPDATA\Programs\OmniCode"

    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null

    Write-Host "Downloading OmniCode v${Version}..."

    try {
        Invoke-WebRequest -Uri $url -OutFile "$tmpDir\$archive" -UseBasicParsing
    } catch {
        Write-Host "Download failed. Trying old archive format..." -ForegroundColor Yellow
        $archiveOld = "OmniCode-v${Version}-windows-x86_64.zip"
        $urlOld = "https://github.com/bomboclati/OmniCode-/releases/download/v${Version}/${archiveOld}"
        try {
            Invoke-WebRequest -Uri $urlOld -OutFile "$tmpDir\$archiveOld" -UseBasicParsing
            $archive = $archiveOld
            $url = $urlOld
        } catch {
            Write-Host "Download failed. Install via cargo or build from source." -ForegroundColor Red
            exit 1
        }
    }

    Write-Host "Extracting..."
    Expand-Archive -Path "$tmpDir\$archive" -DestinationPath "$tmpDir\extracted" -Force

    # Try new binary name first (omni.exe), fall back to old (omnicode.exe)
    $binaryPath = Get-ChildItem -Path "$tmpDir\extracted" -Recurse -Filter "omni.exe" | Select-Object -First 1 -ExpandProperty FullName
    if (-not $binaryPath) {
        $binaryPath = Get-ChildItem -Path "$tmpDir\extracted" -Recurse -Filter "omnicode.exe" | Select-Object -First 1 -ExpandProperty FullName
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

    Move-Item -Path $binaryPath -Destination "$installDir\omni.exe" -Force

    $desktop = [Environment]::GetFolderPath("Desktop")
    $shortcutPath = "$desktop\OmniCode.lnk"

    $wsh = New-Object -ComObject WScript.Shell
    $shortcut = $wsh.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = "$installDir\omni.exe"
    $shortcut.Description = "OmniCode - Autonomous AI Coding Agent"
    $shortcut.WorkingDirectory = "%USERPROFILE%"
    $shortcut.Save()

    $startMenu = [Environment]::GetFolderPath("StartMenu")
    $startMenuPath = "$startMenu\Programs\OmniCode"
    New-Item -ItemType Directory -Path $startMenuPath -Force | Out-Null
    Copy-Item -Path $shortcutPath -Destination "$startMenuPath\OmniCode.lnk" -Force

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
        $env:Path = "$env:Path;$installDir"
        Write-Host "Added $installDir to your PATH" -ForegroundColor $Green
    }

    Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue

    Write-Host ""
    Write-Host "✓ OmniCode installed!" -ForegroundColor $Green
    Write-Host "  Binary: $installDir\omni.exe"
    Write-Host "  Desktop shortcut created."
    Write-Host "  Start menu shortcut created."
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
