$ErrorActionPreference = "Stop"
$sdk = if ($env:ANDROID_HOME) { $env:ANDROID_HOME } else { "$env:LOCALAPPDATA\Android\Sdk" }
$env:ANDROID_HOME = $sdk
$env:ANDROID_SDK_ROOT = $sdk

$ndk = Get-ChildItem "$sdk\ndk" -Directory | Sort-Object Name -Descending | Select-Object -First 1
if (-not $ndk) {
    throw "Android NDK fehlt unter $sdk\ndk"
}
$toolchain = Join-Path $ndk.FullName "toolchains\llvm\prebuilt\windows-x86_64\bin"
$env:PATH = "$toolchain;$sdk\platform-tools;$env:PATH"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = Join-Path $toolchain "aarch64-linux-android21-clang.cmd"

$repo = Split-Path $PSScriptRoot -Parent
Push-Location $repo
try {
    rustup target add aarch64-linux-android
    cargo build --release --target aarch64-linux-android
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}

$binary = Join-Path $repo "target\aarch64-linux-android\release\auto-zygarde"
if (-not (Test-Path $binary) -and $env:CARGO_TARGET_DIR) {
    $candidate = Join-Path $env:CARGO_TARGET_DIR "aarch64-linux-android\release\auto-zygarde"
    if (Test-Path $candidate) { $binary = $candidate }
}
if (-not (Test-Path $binary)) {
    throw "Android-Binary fehlt: $binary"
}

$props = Join-Path $PSScriptRoot "local.properties"
$sdkDir = $sdk.Replace("\", "\\")
Set-Content -Path $props -Value "sdk.dir=$sdkDir" -Encoding ascii

Push-Location $PSScriptRoot
try {
    $binaryArg = "-PbotBinary=$binary"
    if (Test-Path .\gradlew.bat) {
        .\gradlew.bat assembleDebug $binaryArg
    } else {
        gradle assembleDebug $binaryArg
    }
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Pop-Location
}

$apk = Join-Path $PSScriptRoot "app\build\outputs\apk\debug\app-debug.apk"
Write-Host "APK: $apk"
