# PromptLine Development Roadmap

This document outlines the development phases and milestones for PromptLine, from MVP to full product.

## Table of Contents

- [Overview](#overview)
- [Phase 1: MVP](#phase-1-mvp)
- [Phase 2: Expanded Capabilities](#phase-2-expanded-capabilities)
- [Phase 3: Hardening & Security](#phase-3-hardening--security)
- [Phase 4: Full Product & Extensibility](#phase-4-full-product--extensibility)
- [Milestones](#milestones)
- [Release Strategy](#release-strategy)

## Overview

PromptLine follows a phased development approach, delivering a usable release at each phase. This ensures:
- Early value delivery (working MVP)
- Iterative feedback incorporation
- Progressive complexity management
- Community engagement throughout

## Phase 1: MVP

**Goal:** Establish core agentic loop with essential safety features.

### Objectives

1. **Basic Agent Loop**
   - Implement ReACT pattern (Reason → Act → Observe)
   - Multi-step reasoning capability
   - Iteration limits and loop detection

2. **Single Model Provider**
   - OpenAI API integration (GPT-4, GPT-3.5-turbo)
   - Basic prompt engineering
   - Error handling and retries

3. **Essential Tools**
   - Shell command execution (read-only initially)
   - File reading
   - File editing with diff preview

4. **Safety Foundation**
   - Dry-run mode by default
   - User approval for all actions
   - Dangerous command detection

5. **CLI Interface**
   - Command-line argument parsing (Clap)
   - Single-shot command execution
   - Basic output formatting

6. **Configuration**
   - YAML config file support
   - Environment variable integration
   - Tool permission settings

### Deliverables

- ✅ Cargo project structure
- ✅ `LanguageModel` trait and OpenAI implementation
- ✅ `Tool` trait with shell and file tools
- ✅ Agent struct with basic reasoning loop
- ✅ CLI with `run` subcommand
- ✅ Safety validator with approval prompts
- ✅ Diff generation for file edits
- ✅ Configuration loader
- ✅ Unit tests for core components

### Success Criteria

```bash
# User can execute a task like:
promptline "Find all Rust files and add TODO comments"

# Agent will:
1. Propose: find . -name "*.rs"
2. Ask for approval
3. Show diff for each file
4. Ask approval for changes
5. Apply changes
6. Report completion
```

**Target:** Milestone 0.1 (MVP Release)

---

## Phase 2: Expanded Capabilities

**Goal:** Add context awareness, local models, and richer interactions.

### Objectives

1. **Context Management**
   - Conversation history tracking
   - File content injection
   - Environment context (cwd, git branch, etc.)
   - Smart context summarization to manage token limits

2. **Local LLM Support**
   - llama.cpp integration
   - Ollama server support
   - Model switching via config/flag
   - Performance optimization for smaller models

3. **Interactive REPL**
   - Chat mode with persistent context
   - Multi-turn conversations
   - Command history
   - Graceful exit handling

4. **Extended Tools**
   - Git operations (status, diff, commit)
   - Web requests (curl-like functionality)
   - Codebase search (grep, ripgrep integration)
   - File search by content/name

5. **Prompt Engineering**
   - Template system for common tasks
   - Few-shot examples
   - Context-aware prompt assembly
   - Model-specific prompt variations

6. **Project Context**
   - `.promptline/context.md` support (like claude.md)
   - Auto-detection of project type
   - Language-specific context injection

### Deliverables

- ✅ ConversationMemory implementation
- ⏳ Local LLM provider (llama.cpp bindings)
- ⏳ Interactive REPL loop
- ✅ Git tool, Web tool, Search tool
- ✅ Prompt template system
- ✅ Context injection logic
- ⏳ Integration tests for multi-step tasks

### Success Criteria

```bash
# Interactive session
promptline chat
> What files are in this repo?
> Add error handling to src/main.rs
> Now commit those changes with a good message
> exit

# The agent maintains context across all interactions
```

**Target:** Milestone 0.2 & 0.3

---

## Phase 3: Hardening & Security

**Goal:** Production-ready safety, performance, and reliability.

### Objectives

1. **Advanced Safety**
   - Command allow/deny lists
   - Regex-based dangerous pattern detection
   - Protected file patterns (secrets, keys)
   - Sandbox execution mode (Docker/chroot)

2. **Robust Error Handling**
   - Comprehensive error types
   - Recovery strategies
   - Graceful degradation
   - User-friendly error messages

3. **Performance Optimization**
   - Prompt caching
   - Streaming responses
   - Parallel tool execution
   - Token usage optimization

4. **Testing & Quality**
   - Comprehensive unit test coverage (>80%)
   - Integration test suite
   - Property-based testing (fuzzing)
   - Security testing (prompt injection, etc.)

5. **Observability**
   - Structured logging (tracing)
   - Debug mode with verbose output
   - Session logs for debugging
   - Performance metrics

6. **Documentation**
   - User guide with examples
   - API documentation (rustdoc)
   - Architecture docs
   - Troubleshooting guide

### Deliverables

- ✅ SafetyValidator with regex patterns
- ⏳ Sandbox execution module
- ⏳ Streaming UI implementation
- ⏳ Complete test suite
- ⏳ Performance benchmarks
- ⏳ Comprehensive documentation
- ⏳ CI/CD pipeline (GitHub Actions)

### Success Criteria

- All tests passing on Linux, macOS, Windows
- Safety system blocks known dangerous patterns
- Performance: <2s latency for typical operations
- Documentation covers 100% of public API
- Ready for beta release

**Target:** Milestone 0.4 (Beta)

---

## Phase 4: Full Product & Extensibility

**Goal:** Advanced features, plugin ecosystem, and community growth.

### Objectives

1. **Advanced Reasoning**
   - Chain-of-thought improvements
   - Self-reflection and error correction
   - Multi-step plan optimization
   - Function calling for structured output

2. **Plugin System**
   - Plugin trait definition
   - Dynamic plugin loading
   - Plugin marketplace/registry
   - Sandboxed plugin execution (WASI)

3. **Multi-Agent Coordination**
   - Specialized sub-agents
   - Task delegation
   - Parallel agent execution
   - Result aggregation

4. **Enhanced UX**
   - Progress indicators
   - Real-time streaming output
   - Interactive diff editing
   - Shell completion scripts

5. **Additional Providers**
   - Anthropic Claude integration
   - Google PaLM support
   - Azure OpenAI support
   - Custom provider API

6. **Community Features**
   - Prompt template sharing
   - Tool library
   - Example repository
   - Community plugins

### Deliverables

- Plugin loading system
- Multi-agent coordinator
- Additional model providers
- Enhanced UI with TUI (ratatui)
- Plugin SDK documentation
- Community contribution guidelines
- Marketplace infrastructure

### Success Criteria

- Users can install and use community plugins
- Multiple agents can work together
- Support for 5+ model providers
- Active community contributions
- Plugin ecosystem emerging

**Target:** Milestone 1.0 (Production)

---

## Milestones

### Milestone 0.1 - MVP CLI
**Status:** In Progress  
**ETA:** 4-6 weeks

**Features:**
- ✅ Project structure
- ⏳ Basic agent loop
- ⏳ OpenAI integration
- ⏳ Shell + file tools
- ⏳ Safety with approvals
- ⏳ CLI interface

**Goal:** Prove core concept, gather initial feedback

---

### Milestone 0.2 - Multi-Step Agent & Context
**Status:** Planned  
**ETA:** +3 weeks

**Features:**
- Context management
- Conversation memory
- REPL mode
- Configuration files
- Project context support

**Goal:** Enable complex multi-turn interactions

---

### Milestone 0.3 - Extended Tools & Local Models
**Status:** Planned  
**ETA:** +4 weeks

**Features:**
- Git tools
- Web request tools
- Codebase search
- Local LLM support (llama.cpp)
- Provider switching

**Goal:** Expand capabilities, reduce API dependency

---

### Milestone 0.4 - Beta Release
**Status:** Planned  
**ETA:** +6 weeks

**Features:**
- Complete safety system
- Comprehensive tests
- Performance optimization
- Full documentation
- CI/CD pipeline

**Goal:** Production-ready beta for community testing

---

### Milestone 1.0 - Production Release
**Status:** Planned  
**ETA:** +8 weeks

**Features:**
- Plugin system
- Multi-agent support
- Multiple providers
- Polish and UX
- Community features

**Goal:** Stable, extensible, community-driven product

---

## Release Strategy

### Versioning

Follow Semantic Versioning (semver):
- **0.x.y** - Pre-1.0 releases (breaking changes allowed)
- **1.x.y** - Post-1.0 (backward compatibility)
- **x.y.z** - Major.Minor.Patch

### Release Process

1. **Feature Development** on feature branches
2. **PR Review** with tests and docs
3. **Merge to main** after approval
4. **Version bump** and changelog update
5. **Tag release** (e.g., v0.1.0)
6. **CI builds** binaries for all platforms
7. **Publish to crates.io**
8. **Release notes** on GitHub
9. **Announcement** (blog, Twitter, Reddit)

### Release Frequency

- **MVP Phase:** Weekly releases (0.1.x)
- **Expansion Phase:** Bi-weekly (0.2.x, 0.3.x)
- **Hardening Phase:** Monthly (0.4.x)
- **Post-1.0:** As needed (patches), quarterly (features)

### Communication

- **Changelog:** `CHANGELOG.md` with detailed changes
- **Migration Guides:** For breaking changes
- **Blog Posts:** Major milestone announcements
- **Discord/Forum:** Community discussions

---

## Long-Term Vision (Post-1.0)

### Year 1
- Mature plugin ecosystem
- Enterprise features (SSO, audit logs)
- Cloud-hosted agents (optional)
- Integration with IDEs (VS Code extension)

### Year 2
- Visual workflow builder
- Team collaboration features
- Agent marketplace
- Training/fine-tuning support

### Year 3
- Autonomous agent swarms
- Industry-specific agents
- SaaS offering
- Open-source community governance

---

## How to Contribute

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Feature requests
- Bug reports
- Pull request guidelines
- Code review process

## Feedback

We welcome feedback at any stage:
- **GitHub Issues:** Bug reports and feature requests
- **Discussions:** General questions and ideas
- **Discord:** Real-time community chat (coming soon)

---

**Last Updated:** 2025-11-17  
**Current Phase:** Phase 1 (MVP)  
**Next Milestone:** 0.1 (MVP CLI)
