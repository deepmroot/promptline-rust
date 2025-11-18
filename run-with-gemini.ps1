# PowerShell script to run PromptLine with Gemini API

# Set your Gemini API key here
$env:GEMINI_API_KEY = "YOUR_GEMINI_API_KEY_HERE"

# Tell PromptLine to use Gemini provider
$env:PROMPTLINE_PROVIDER = "gemini"

# Build and run
Write-Host "🦀 Building PromptLine..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful!" -ForegroundColor Green
    Write-Host "🚀 Running with Gemini API..." -ForegroundColor Cyan
    
    # Run the CLI with your task
    .\target\release\promptline.exe $args
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
}
