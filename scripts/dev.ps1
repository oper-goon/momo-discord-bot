param([Parameter(ValueFromRemainingArguments = $true)][string[]]$CargoArgs)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$env:CARGO_HOME = Join-Path $root '.tools/cargo'
$env:RUSTUP_HOME = Join-Path $root '.tools/rustup'
$compiler = Get-ChildItem (Join-Path $root '.tools') -Directory -Filter 'llvm-mingw-*' | Select-Object -First 1
if ($compiler) {
    $env:PATH = "$($compiler.FullName)\bin;$env:PATH"
    $env:CC = 'x86_64-w64-mingw32-clang'
    $env:CXX = 'x86_64-w64-mingw32-clang++'
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER = 'x86_64-w64-mingw32-clang'
}
$cmake = Get-ChildItem (Join-Path $root '.tools') -Directory -Filter 'cmake-*-windows-x86_64' | Select-Object -First 1
if ($cmake) {
    $env:PATH = "$($cmake.FullName)\bin;$root\.tools\ninja;$env:PATH"
    $env:CMAKE_GENERATOR = 'Ninja'
}
$env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
$toolchainFile = Get-Content -LiteralPath (Join-Path $root 'rust-toolchain.toml') -Raw
$toolchainMatch = [regex]::Match($toolchainFile, '(?m)^\s*channel\s*=\s*"([^"]+)"')
if (-not $toolchainMatch.Success) {
    throw 'rust-toolchain.toml must declare the Rust channel.'
}
$windowsToolchain = "$($toolchainMatch.Groups[1].Value)-x86_64-pc-windows-gnullvm"
& cargo "+$windowsToolchain" @CargoArgs
exit $LASTEXITCODE
