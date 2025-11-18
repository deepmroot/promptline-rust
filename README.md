# PromptLine 🤖⚡

**An Agentic AI-Powered CLI Tool for Intelligent Code Assistance**

> High-performance command-line interface built in Rust, combining AI language models with secure, extensible tooling to help developers write, refactor, debug, and understand code faster.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

---

## 🌟 Features

- 🧠 **Multi-Model Support**: Works with OpenAI, Anthropic, and local models (Ollama, llama.cpp)
- 🤖 **ReACT Agent Loop**: Implements Think → Act → Observe pattern for multi-step reasoning
- 🛡️ **Safety First**: Dry-run mode, command filtering, and diff previews before execution
- 📋 **Interactive & Headless**: REPL mode for conversations or single-shot commands
- 🔧 **Extensible Tools**: File operations, shell execution, git integration, codebase search
- 🔌 **Plugin System**: (Future) Support for third-party tool integrations
- 🎯 **Context-Aware**: Intelligent context management with conversation memory
- 🚀 **Fast & Lightweight**: Single compiled binary with minimal dependencies

## 🚀 Quick Start

### Prerequisites

Install Rust: https://rustup.rs/

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/promptline-rust.git
cd promptline-rust

# Build the project
cargo build --release

# Install locally
cargo install --path .
```

### Configuration

Create a config file at `~/.promptline/config.yaml`:

```yaml
models:
  default: "gpt-4"
  providers:
    openai:
      api_key: "${OPENAI_API_KEY}"

tools:
  file_read: allow
  file_write: ask
  shell_execute: ask

safety:
  require_approval: true
  dangerous_commands: ["rm -rf", "format", "mkfs"]
```

Or set environment variables:

```bash
export OPENAI_API_KEY="your-api-key-here"
```

### Usage Examples

**Single-shot command:**
```bash
# Find and compress large files
promptline "Find the 5 largest files and compress them"

# Refactor code
promptline "Add error handling to all functions in src/main.rs"
```

**Interactive mode:**
```bash
promptline chat
> What files are in this directory?
> Add unit tests for the authentication module
```

**File editing:**
```bash
promptline edit config.yaml "set debug mode to true"
```

**Plan mode (read-only):**
```bash
promptline plan "refactor the database layer"
```

## 📚 Documentation

- [Architecture](docs/ARCHITECTURE.md) - System design and module structure
- [Roadmap](docs/ROADMAP.md) - Development phases and milestones
- [Safety & Security](docs/SAFETY.md) - Security model and safety features
- [Prompt Engineering](docs/PROMPT_ENGINEERING.md) - Context management and prompt design
- [Testing Strategy](docs/TESTING.md) - Testing approach and guidelines
- [Deployment](docs/DEPLOYMENT.md) - Building and distributing PromptLine
- [Contributing](docs/CONTRIBUTING.md) - How to contribute to the project
- [Plugin System](docs/PLUGIN_SYSTEM.md) - Future extensibility architecture

## 🏗️ Project Structure

```
promptline-rust/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli.rs               # Command-line interface
│   ├── agent/               # Agent loop and reasoning
│   │   ├── mod.rs
│   │   ├── planner.rs
│   │   └── memory.rs
│   ├── tools/               # Tool implementations
│   │   ├── mod.rs
│   │   ├── shell.rs
│   │   └── file_edit.rs
│   ├── model/               # LLM integrations
│   │   ├── mod.rs
│   │   ├── openai.rs
│   │   └── local_llm.rs
│   ├── prompt/              # Prompt engineering
│   │   └── mod.rs
│   └── util/                # Utilities
└── tests/                   # Integration tests
```

## 🛣️ Roadmap

**Phase 1 - MVP** (Current)
- ✅ Core ReACT agent loop
- ✅ OpenAI integration
- ✅ Basic shell and file tools
- ✅ Safety layer with approvals
- ⏳ CLI interface with Clap
- ⏳ Configuration system

**Phase 2 - Expanded Capabilities**
- Context management and memory
- Local LLM support (llama.cpp)
- Interactive REPL mode
- Extended toolset (git, web requests)

**Phase 3 - Hardening**
- Robust safety constraints
- Command sandboxing
- Performance optimizations
- Comprehensive testing

**Phase 4 - Full Product**
- Advanced reasoning techniques
- Plugin system
- Multi-agent coordination
- Polish and UX improvements

See [ROADMAP.md](docs/ROADMAP.md) for detailed milestones.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Built with inspiration from:
- [Continue.dev CLI](https://continue.dev)
- [GitHub Copilot CLI](https://githubnext.com/projects/copilot-cli)
- [Factory AI's Droid](https://factory.ai)
- [AutoGPT](https://github.com/Significant-Gravitas/AutoGPT)

## 🔗 Links

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/yourusername/promptline-rust/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/promptline-rust/discussions)

---

**Made with ❤️ by developers, for developers**
