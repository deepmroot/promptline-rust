# Phase 3 Manual Test Runner
# This script helps you test the new UX features

Write-Host "`n╔═══════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║       Phase 3 UX Overhaul - Manual Testing       ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════╝`n" -ForegroundColor Cyan

# Check cloud Ollama connectivity
Write-Host "🔍 Checking prerequisites..." -ForegroundColor Yellow
$cloudOllamaAvailable = $false
try {
    $response = Invoke-WebRequest -Uri "https://ollama.com/api/tags" -Method GET -TimeoutSec 5 -ErrorAction SilentlyContinue
    if ($response.StatusCode -eq 200) {
        $cloudOllamaAvailable = $true
        Write-Host "✓ Cloud Ollama (https://ollama.com) is accessible" -ForegroundColor Green
    }
} catch {
    Write-Host "⚠️  Could not verify cloud Ollama connection" -ForegroundColor Yellow
    Write-Host "  Endpoint: https://ollama.com" -ForegroundColor Gray
    Write-Host "  Continuing anyway..." -ForegroundColor Gray
}

# Note about model
Write-Host "`n📦 Using cloud model:" -ForegroundColor Yellow
Write-Host "  Model: gpt-oss:120b-cloud" -ForegroundColor White
Write-Host "  Endpoint: https://ollama.com" -ForegroundColor White

Write-Host "`n" + "="*60 -ForegroundColor Gray
Write-Host "TEST PLAN" -ForegroundColor Cyan
Write-Host "="*60 -ForegroundColor Gray

$tests = @"

Test 1: Loading Indicators
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Command: hi
  Expected: 
    - See rotating loading messages (🤔 Thinking...)
    - Messages change every 2 seconds
    - Auto-clears when response arrives

Test 2: Tool Execution with Icons
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Command: list files in current directory
  Expected:
    - 📁 DIRECTORY LISTING icon appears
    - Clean formatted output
    - NO "Tool 'file_list' result:" visible

Test 3: Identity Check
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Command: who are you?
  Expected:
    - Response says "I'm PromptLine"
    - NO mention of Cogito, Claude, GPT
    - Professional tone

Test 4: Permission System
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Command: read Cargo.toml
  Expected:
    - Permission prompt if first time
    - Tool execution with 📄 icon
    - NO raw "Execute tool" messages

Test 5: Search with Formatting
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Command: search for "formatter" in src
  Expected:
    - 🔍 SEARCH RESULTS icon
    - Clean formatted results
    - Loading indicator during search

"@

Write-Host $tests -ForegroundColor White

Write-Host "`n" + "="*60 -ForegroundColor Gray
Write-Host "STARTING PROMPTLINE" -ForegroundColor Cyan
Write-Host "="*60 -ForegroundColor Gray
Write-Host ""
Write-Host "Type the test commands above to verify each feature." -ForegroundColor Yellow
Write-Host "Type 'exit' or 'quit' to end testing.`n" -ForegroundColor Yellow

# Set environment and run
$env:PROMPTLINE_PROVIDER = "ollama"
$env:RUST_LOG = "warn"  # Keep it clean

# Create temporary config for testing with cloud Ollama
$tempConfig = @"
models:
  default: "gpt-oss:120b-cloud"
  providers:
    ollama:
      base_url: "https://ollama.com"
      api_key: "edbed07b7b0945599c0111133eb98dfa.mOilfWJN_ypipy0UFdd7XgJ7"
      models:
        - gpt-oss:120b-cloud
        - gemma3:1b

tools:
  file_read: allow
  file_write: ask
  file_delete: deny
  shell_execute: ask

safety:
  require_approval: false
  require_diff_preview: true
  max_iterations: 10
  enable_backups: true
  dangerous_commands:
    - "^rm -rf /"
    - "^mkfs"
    - "^dd if="
    - "^format [A-Z]:"

agent:
  default_mode: "chat"
  use_chain_of_thought: true
  explain_before_action: true
"@

$tempConfigPath = "Z:\promptline-rust\config.test.yaml"
$tempConfig | Out-File -FilePath $tempConfigPath -Encoding UTF8
Write-Host "✓ Created test config: gpt-oss:120b-cloud @ https://ollama.com" -ForegroundColor Green

Write-Host ""
Write-Host "Press any key to start PromptLine..." -ForegroundColor Cyan
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
Write-Host ""

# Run PromptLine with test config
& "Z:\promptline-rust\target\release\promptline.exe" --config $tempConfigPath

Write-Host "`n`n" + "="*60 -ForegroundColor Gray
Write-Host "TEST RESULTS" -ForegroundColor Cyan
Write-Host "="*60 -ForegroundColor Gray
Write-Host ""
Write-Host "Please verify the following worked:" -ForegroundColor Yellow
Write-Host ""
Write-Host "  [ ] Loading indicators showed during API calls" -ForegroundColor White
Write-Host "  [ ] Tool execution showed icons (📄, 📁, 🔍)" -ForegroundColor White
Write-Host "  [ ] No raw 'Tool result:' messages visible" -ForegroundColor White
Write-Host "  [ ] Identity is 'PromptLine' not Cogito" -ForegroundColor White
Write-Host "  [ ] Permission prompts appeared correctly" -ForegroundColor White
Write-Host "  [ ] Output was clean and professional" -ForegroundColor White
Write-Host ""

# Ask for feedback
$allPassed = Read-Host "Did all tests pass? (y/n)"
if ($allPassed -eq "y") {
    Write-Host "`n✅ Phase 3 testing complete! Ready for Phase 4." -ForegroundColor Green
} else {
    Write-Host "`n⚠️  Some tests failed. Please review output above." -ForegroundColor Yellow
    Write-Host "   You can re-run this script or report issues." -ForegroundColor Yellow
}

# Cleanup temp config
if (Test-Path $tempConfigPath) {
    Remove-Item $tempConfigPath
    Write-Host "✓ Cleaned up test config" -ForegroundColor Gray
}

Write-Host ""
