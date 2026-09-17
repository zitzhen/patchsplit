@echo off
rem Update the project version everywhere: Cargo.toml, Cargo.lock,
rem debian/changelog and packaging/patchsplit.spec.
rem Usage: scripts\update-version.bat [X.Y.Z]  (prompts for a version if omitted)

cd /d "%~dp0.."

set "VERSION=%~1"
if not defined VERSION set /p "VERSION=Enter new version (e.g. 1.3.0): "

if not defined VERSION (
    echo Error: no version entered.
    exit /b 1
)

powershell -NoProfile -Command ^
    "$ErrorActionPreference = 'Stop';" ^
    "$v = $env:VERSION;" ^
    "if ($v -notmatch '^[0-9]+\.[0-9]+\.[0-9]+$') { Write-Host ('Error: invalid version: ' + $v + ' (expected X.Y.Z)'); exit 1 };" ^
    "$debSuffix = '1~ubuntu26.04.1';" ^
    "$debDist = 'resolute';" ^
    "$now = Get-Date;" ^
    "$ci = [Globalization.CultureInfo]::InvariantCulture;" ^
    "$debDate = [regex]::Replace($now.ToString('ddd, dd MMM yyyy HH:mm:ss zzz', $ci), '([+-]\d{2}):(\d{2})$', '$1$2');" ^
    "$specDate = $now.ToString('ddd MMM dd yyyy', $ci);" ^
    "$p = 'Cargo.toml';" ^
    "$t = [IO.File]::ReadAllText($p);" ^
    "$t = $t -replace '(?m)^version = \"[^\"]*\"', ('version = \"' + $v + '\"');" ^
    "[IO.File]::WriteAllText($p, $t);" ^
    "$p = 'Cargo.lock';" ^
    "$t = [IO.File]::ReadAllText($p);" ^
    "$t = $t -replace '(?m)(name = \"patchsplit\"\r?\nversion = \")[^\"]*\"', ('$1' + $v + '\"');" ^
    "[IO.File]::WriteAllText($p, $t);" ^
    "$p = 'debian/changelog';" ^
    "$t = [IO.File]::ReadAllText($p);" ^
    "$eol = if ($t.Contains([char]13)) { \"`r`n\" } else { \"`n\" };" ^
    "$entry = 'patchsplit (' + $v + '-' + $debSuffix + ') ' + $debDist + '; urgency=medium' + $eol + $eol + '  *' + $eol + $eol + ' -- Oliver <oliver@liuxiaozhen.dev>  ' + $debDate + $eol + $eol;" ^
    "[IO.File]::WriteAllText($p, $entry + $t);" ^
    "$p = 'packaging/patchsplit.spec';" ^
    "$t = [IO.File]::ReadAllText($p);" ^
    "$t = $t -replace '(?m)^Version:.*$', ('Version:        ' + $v);" ^
    "$t = $t -replace ('(?m)^' + [char]37 + 'changelog\r?\n'), ([char]37 + 'changelog' + $eol + '* ' + $specDate + ' Oliver Lin <oliver@liuxiaozhen.dev> - ' + $v + '-1' + $eol + $eol);" ^
    "[IO.File]::WriteAllText($p, $t);"

if errorlevel 1 (
    echo Version update failed.
    exit /b 1
)

echo Updated to %VERSION%:
echo   Cargo.toml
echo   Cargo.lock
echo   debian/changelog
echo   packaging/patchsplit.spec
