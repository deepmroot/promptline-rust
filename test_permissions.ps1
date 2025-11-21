# Test script for Phase 2 - Permission System
# This script will test the permission prompts

Write-Host "`n=== Phase 2 Permission System Test ===" -ForegroundColor Cyan
Write-Host ""

# Set environment for Ollama
$env:PROMPTLINE_PROVIDER = "ollama"

# Check if permissions file exists before test
$permFile = "$env:USERPROFILE\.promptline\permissions.yaml"
if (Test-Path $permFile) {
    Write-Host "📋 Existing permissions file found:" -ForegroundColor Yellow
    Get-Content $permFile
    Write-Host ""
    $backup = "${permFile}.backup"
    Copy-Item $permFile $backup
    Write-Host "✓ Backed up to: $backup" -ForegroundColor Green
    Remove-Item $permFile
    Write-Host "✓ Removed for clean test" -ForegroundColor Green
    Write-Host ""
}

Write-Host "🧪 Test Plan:" -ForegroundColor Cyan
Write-Host "  1. Start PromptLine in chat mode"
Write-Host "  2. Ask it to 'list files' (will prompt for permission)"
Write-Host "  3. Select 'Always' (option 2)"
Write-Host "  4. Exit and check permissions.yaml was created"
Write-Host ""
Write-Host "📝 To test:" -ForegroundColor Yellow
Write-Host "  1. Type: list files in current directory"
Write-Host "  2. When prompted, press: 2 (for Always)"
Write-Host "  3. Type: exit"
Write-Host ""

# Run PromptLine
& "Z:\promptline-rust\target\release\promptline.exe"

# Check results after exit
Write-Host "`n=== Test Results ===" -ForegroundColor Cyan
if (Test-Path $permFile) {
    Write-Host "✓ Permissions file created!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Contents:" -ForegroundColor Yellow
    Get-Content $permFile
} else {
    Write-Host "✗ Permissions file not found!" -ForegroundColor Red
}
