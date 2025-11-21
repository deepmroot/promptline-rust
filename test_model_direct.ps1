# Direct test of Ollama API to see exactly what it returns

$apiKey = "edbed07b7b0945599c0111133eb98dfa.mOilfWJN_ypipy0UFdd7XgJ7"
$headers = @{ "Authorization" = "Bearer $apiKey" }

$body = @{
    model = "gpt-oss:120b-cloud"
    messages = @(
        @{
            role = "system"
            content = @"
You are PromptLine. When you've completed a task, respond with: FINISH

User said: hi
"@
        }
        @{
            role = "user"
            content = "hi"
        }
    )
    stream = $false
} | ConvertTo-Json -Depth 10

Write-Host "Calling Ollama API..." -ForegroundColor Cyan
try {
    $response = Invoke-WebRequest -Uri "https://ollama.com/api/chat" -Method POST -Body $body -ContentType "application/json" -Headers $headers -TimeoutSec 15
    
    $result = $response.Content | ConvertFrom-Json
    
    Write-Host "`nModel Response:" -ForegroundColor Green
    Write-Host $result.message.content
    Write-Host "`n---" -ForegroundColor Gray
    Write-Host "Ends with FINISH?: $($result.message.content.Trim().EndsWith('FINISH'))" -ForegroundColor Yellow
    Write-Host "Contains 'FINISH'?: $($result.message.content.Contains('FINISH'))" -ForegroundColor Yellow
    
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
}
