# PromptLine Documentation Index

Welcome to the PromptLine documentation! This index will help you find the information you need.

## 📚 Documentation Overview

### Getting Started

- **[README](../README.md)** - Project overview, quick start, and basic usage
- **[CHANGELOG](../CHANGELOG.md)** - Version history and changes

### Architecture & Design

- **[ARCHITECTURE](ARCHITECTURE.md)** - System architecture, module structure, and design patterns
  - Module organization
  - Core components (Agent, Model, Tools)
  - Data flow and trait system
  - Error handling and concurrency

- **[ROADMAP](ROADMAP.md)** - Development phases and milestones
  - Phase 1: MVP (Current)
  - Phase 2: Expanded Capabilities
  - Phase 3: Hardening & Security
  - Phase 4: Full Product & Extensibility
  - Release strategy

### Technical Guides

- **[PROMPT_ENGINEERING](PROMPT_ENGINEERING.md)** - Prompt design and context management
  - System prompt design
  - Context assembly strategies
  - ReACT pattern implementation
  - Token management
  - Model-specific optimizations

- **[SAFETY](SAFETY.md)** - Security model and safety features
  - Multi-layer safety system
  - Approval workflows
  - Command validation
  - File protection
  - Sandboxing strategies
  - Prompt injection defense

- **[TESTING](TESTING.md)** - Testing strategy and guidelines
  - Unit tests
  - Integration tests
  - Property-based tests
  - Security tests
  - Performance benchmarks
  - CI/CD pipeline

### Operations

- **[DEPLOYMENT](DEPLOYMENT.md)** - Building and distribution
  - Building from source
  - Cross-compilation
  - Release process
  - Platform-specific packaging (Homebrew, Docker, etc.)
  - Configuration management

### Contributing

- **[CONTRIBUTING](CONTRIBUTING.md)** - Contribution guidelines
  - Code of conduct
  - Development workflow
  - Code style guidelines
  - Pull request process
  - Areas for contribution

### Future Plans

- **[PLUGIN_SYSTEM](PLUGIN_SYSTEM.md)** - Plugin architecture (Planned for Phase 4)
  - Plugin types
  - Plugin API
  - Security model
  - Plugin development guide
  - Plugin registry

## 🎯 Quick Navigation

### I want to...

**Use PromptLine**
→ Start with [README](../README.md) for installation and basic usage

**Understand the design**
→ Read [ARCHITECTURE](ARCHITECTURE.md) for system overview

**Contribute code**
→ Follow [CONTRIBUTING](CONTRIBUTING.md) guidelines

**Report a bug or request a feature**
→ Check [CONTRIBUTING](CONTRIBUTING.md#communication) for how to report issues

**Write effective prompts**
→ See [PROMPT_ENGINEERING](PROMPT_ENGINEERING.md)

**Understand security**
→ Review [SAFETY](SAFETY.md) documentation

**Build from source**
→ Follow [DEPLOYMENT](DEPLOYMENT.md) instructions

**Write tests**
→ Consult [TESTING](TESTING.md) guide

**Check roadmap**
→ See [ROADMAP](ROADMAP.md) for development timeline

**Develop a plugin** (future)
→ Refer to [PLUGIN_SYSTEM](PLUGIN_SYSTEM.md)

## 📋 Documentation Structure

```
promptline-rust/
├── README.md              # Project overview and quick start
├── LICENSE                # MIT License
├── CHANGELOG.md           # Version history
├── Cargo.toml             # Rust project configuration
│
├── docs/                  # Detailed documentation
│   ├── INDEX.md           # This file
│   ├── ARCHITECTURE.md    # System design
│   ├── ROADMAP.md         # Development plan
│   ├── SAFETY.md          # Security features
│   ├── PROMPT_ENGINEERING.md  # Prompt design
│   ├── TESTING.md         # Testing guide
│   ├── DEPLOYMENT.md      # Build & distribution
│   ├── CONTRIBUTING.md    # Contribution guide
│   └── PLUGIN_SYSTEM.md   # Plugin architecture
│
└── src/                   # Source code
    └── main.rs            # Entry point
```

## 🔍 Search Tips

Use your IDE's search function to find specific topics across all docs:

- **Agent loop** → ARCHITECTURE.md
- **ReACT pattern** → ARCHITECTURE.md, PROMPT_ENGINEERING.md
- **Safety** → SAFETY.md
- **Dangerous commands** → SAFETY.md
- **Tool trait** → ARCHITECTURE.md
- **Testing mocks** → TESTING.md
- **Cross-compilation** → DEPLOYMENT.md
- **Plugin development** → PLUGIN_SYSTEM.md
- **Code style** → CONTRIBUTING.md
- **Milestones** → ROADMAP.md

## 📞 Getting Help

- **Questions?** → [GitHub Discussions](https://github.com/yourusername/promptline-rust/discussions)
- **Bug reports** → [GitHub Issues](https://github.com/yourusername/promptline-rust/issues)
- **Discord** → Coming soon
- **Email** → maintainer@promptline.dev

## 🤝 Contributing to Docs

Documentation improvements are always welcome! See [CONTRIBUTING](CONTRIBUTING.md#non-code-contributions) for how to help improve these docs.

---

**Last Updated:** 2025-11-17  
**Documentation Version:** 0.1.0
