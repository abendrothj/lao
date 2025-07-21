# Run this from your workspace root (C:\Users\jakea\Desktop\lao-orchestrator)
$releaseDir = "target/release"
$pluginDir = "plugins"

# Find all plugin DLLs in the release directory
$dlls = Get-ChildItem -Path $releaseDir -Filter "*.dll" -File

foreach ($dll in $dlls) {
    Copy-Item $dll.FullName -Destination $pluginDir -Force
    Write-Host "Copied $($dll.Name) to plugins/"
}
Write-Host "All plugin DLLs copied to plugins/ directory."