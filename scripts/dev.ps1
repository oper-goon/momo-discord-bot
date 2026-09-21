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
& cargo +stable-x86_64-pc-windows-gnullvm @CargoArgs
exit $LASTEXITCODE
