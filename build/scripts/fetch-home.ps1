$out = Join-Path $PSScriptRoot '..\legacy-home-source.html'
Invoke-WebRequest -Uri 'https://tzyy11.vercel.app/' -OutFile $out -UseBasicParsing
Write-Host "Saved:" (Get-Item $out).Length "bytes"
