# Installs og, Origin Engine's launcher (docs/release.md §1), into
# ~\.origin\bin\og.cmd ($env:ORIGIN_HOME\bin\og.cmd):
#
#   irm https://origingame.ai/downloads/engine/install.ps1 | iex
#
# The launcher is og.ts and the minisign verifier it shares with the
# engine, run by bun; it downloads each engine
# version a game pins, verified with the engine's signing key, into
# ~\.cache\origin-engine. Running this again replaces the launcher and
# leaves the installed engines alone.
$ErrorActionPreference = 'Stop'

$dir = if ($env:ORIGIN_HOME) { $env:ORIGIN_HOME } else { Join-Path $HOME '.origin' }
function Say($s) { Write-Host "origin: $s" }

New-Item -ItemType Directory -Force -Path (Join-Path $dir 'bin'), (Join-Path $dir 'lib\launcher'), (Join-Path $dir 'lib\engine\src\node') | Out-Null
# lib mirrors the repository's packages: lib\launcher\og.ts imports
# ..\engine\src\node\minisign.ts, the same path packages\launcher\og.ts runs.
$launcher = @'
__OG_LAUNCHER__
'@
$utf8 = New-Object System.Text.UTF8Encoding $false
[IO.File]::WriteAllText((Join-Path $dir 'lib\launcher\og.ts'), $launcher.Replace("`r`n", "`n"), $utf8)
$minisign = @'
__OG_MINISIGN__
'@
[IO.File]::WriteAllText((Join-Path $dir 'lib\engine\src\node\minisign.ts'), $minisign.Replace("`r`n", "`n"), $utf8)

$project = @'
__OG_PROJECT__
'@
[IO.File]::WriteAllText((Join-Path $dir 'lib\engine\src\node\project-metadata.ts'), $project.Replace("`r`n", "`n"), $utf8)

$shim = "@echo off`r`nsetlocal`r`n" +
  "set `"OG_BUN=bun`"`r`n" +
  "where bun >nul 2>nul || set `"OG_BUN=%USERPROFILE%\.bun\bin\bun.exe`"`r`n" +
  "`"%OG_BUN%`" `"$dir\lib\launcher\og.ts`" %*`r`n"
[IO.File]::WriteAllText((Join-Path $dir 'bin\og.cmd'), $shim, $utf8)
Say "installed og in $(Join-Path $dir 'bin\og.cmd')"

if (-not (Get-Command bun -ErrorAction SilentlyContinue) -and -not (Test-Path "$HOME\.bun\bin\bun.exe")) {
  Say 'og runs on bun, which is not installed: powershell -c "irm bun.sh/install.ps1 | iex"'
}
if (-not (Get-Command tar -ErrorAction SilentlyContinue)) { Say 'og unpacks engines with tar.exe (Windows 10 1803 and later carry it), which is not found' }
$bin = Join-Path $dir 'bin'
if (-not (($env:PATH -split ';') -contains $bin)) {
  Say "add $bin to PATH: [Environment]::SetEnvironmentVariable('Path', `"$bin;`" + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')"
}
Say 'then: og new my-game; cd my-game; og run .'
