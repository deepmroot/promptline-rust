# Phase 3 Testing Guide

## Quick Start

```powershell
cd Z:\promptline-rust
.\run_phase3_test.ps1
```

This script will:
1. Check if Ollama is running
2. List available models
3. Create a test config
4. Launch PromptLine in chat mode
5. Guide you through tests

## What to Look For

### ✅ Loading Indicators
**When:** Any time the model is thinking
**Look for:** Rotating messages like:
- 🤔 Thinking...
- ⚙️ Processing your request...
- 🧠 Analyzing code...
- ✨ Brewing some wisdom...

**How to test:** Type `hi` and watch the loading messages

### ✅ Tool Execution Icons
**When:** Using any tool (file_list, file_read, etc.)
**Look for:**
- 📁 DIRECTORY LISTING
- 📄 FILE CONTENT
- 🔍 SEARCH RESULTS
- 📊 GIT STATUS

**How to test:** Type `list files`

### ✅ Clean Output
**What should NOT appear:**
- ❌ "Tool 'file_list' result:"
- ❌ Raw JSON tool calls
- ❌ Debug messages
- ❌ "Execute tool 'X' with args..."

**What SHOULD appear:**
- ✅ Icons with formatted sections
- ✅ Clean, readable responses
- ✅ Professional tone

### ✅ Identity Check
**When:** Asking "who are you?"
**Look for:**
- ✅ "I'm PromptLine"
- ✅ "PromptLine is..."

**Should NOT see:**
- ❌ "I'm Cogito"
- ❌ "I'm Claude"
- ❌ "I'm an AI assistant created by..."
- ❌ Mentions of underlying model

### ✅ Permission System
**When:** First time using a tool
**Look for:**
- Permission prompt with options:
  - [1] Once
  - [2] Always
  - [3] Never
- Clean formatting after choice
- No repeated prompts if you chose "Always"

## Test Commands

Copy and paste these into PromptLine:

```
# Test 1: Loading Indicator
hi

# Test 2: Tool Icons
list files in current directory

# Test 3: Identity
who are you?

# Test 4: File Reading (with permission)
read Cargo.toml

# Test 5: Search
search for "formatter" in src

# Exit
exit
```

## Expected Results

### Before (Old UX):
```
✔ Execute tool 'file_list' with args: {"path":"."}? · yes
Tool 'file_list' result: Found 21 items:
dir        0          .git
file       195        .gitignore
...
```

### After (New UX):
```
🤔 Thinking...

📁 DIRECTORY LISTING
   ↳ Found 21 items

PromptLine: I found 21 files and directories in the current folder.
```

## Troubleshooting

### Ollama not running
```powershell
# Start Ollama in a separate terminal
ollama serve
```

### Model not found
```powershell
# Pull gemma3:1b model
ollama pull gemma3:1b
```

### Test script fails
```powershell
# Run manually
cd Z:\promptline-rust
$env:PROMPTLINE_PROVIDER = "ollama"
.\target\release\promptline.exe --config config.test.yaml
```

### No loading indicators
- Check if responses are very fast (model might respond instantly)
- Try a more complex query that takes longer

## Reporting Results

After testing, note:
- [ ] Loading indicators appeared
- [ ] Tool icons showed correctly
- [ ] No raw tool messages visible
- [ ] Identity is "PromptLine"
- [ ] Permissions worked smoothly
- [ ] Overall UX feels polished

## Next: Phase 4

Once Phase 3 is verified, we'll add:
- Slash commands (/help, /settings, /quit)
- Command shortcuts (/h, /q, /perms)
- Better session management
