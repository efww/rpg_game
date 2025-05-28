# PowerShell script to set console to UTF-8
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$env:RUST_BACKTRACE = 1
chcp 65001
Write-Host "Console set to UTF-8"