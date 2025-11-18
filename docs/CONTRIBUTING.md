# Contributing to PromptLine

Thank you for your interest in contributing to PromptLine! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing Guidelines](#testing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Areas for Contribution](#areas-for-contribution)

## Code of Conduct

### Our Pledge

We pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

**Positive behavior:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable behavior:**
- Trolling, insulting comments, and personal attacks
- Public or private harassment
- Publishing others' private information
- Other conduct which could reasonably be considered inappropriate

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported by contacting the project team. All complaints will be reviewed and investigated promptly and fairly.

## Getting Started

### Prerequisites

- Rust 1.70+ ([install from rustup.rs](https://rustup.rs/))
- Git
- A GitHub account
- Familiarity with Rust basics

### Development Environment Setup

1. **Fork the repository**
   ```bash
   # Click "Fork" on GitHub, then:
   git clone https://github.com/YOUR_USERNAME/promptline-rust.git
   cd promptline-rust
   ```

2. **Add upstream remote**
   ```bash
   git remote add upstream https://github.com/yourusername/promptline-rust.git
   ```

3. **Install development tools**
   ```bash
   # Formatter
   rustup component add rustfmt
   
   # Linter
   rustup component add clippy
   
   # Code coverage (optional)
   cargo install cargo-tarpaulin
   
   # Security audit
   cargo install cargo-audit
   ```

4. **Build and test**
   ```bash
   cargo build
   cargo test
   ```

5. **Set up pre-commit hooks** (optional but recommended)
   ```bash
   cp scripts/pre-commit .git/hooks/
   chmod +x .git/hooks/pre-commit
   ```

## Development Workflow

### 1. Find or Create an Issue

- Browse [existing issues](https://github.com/yourusername/promptline-rust/issues)
- Look for issues labeled `good-first-issue` or `help-wanted`
- Or create a new issue describing what you want to work on

### 2. Discuss Your Approach

For significant changes:
- Comment on the issue with your proposed approach
- Wait for feedback from maintainers
- This prevents duplicate work and ensures alignment

### 3. Create a Branch

```bash
# Sync with upstream
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

**Branch naming conventions:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes

### 4. Make Your Changes

- Write clean, documented code
- Follow the [code style guidelines](#code-style)
- Add tests for new functionality
- Update documentation as needed

### 5. Commit Your Changes

```bash
# Stage changes
git add .

# Commit with descriptive message
git commit -m "feat: add context management for conversations

- Implement ConversationMemory struct
- Add context providers for file, git, env
- Include tests for context assembly
- Update documentation

Closes #123"
```

**Commit message format:**
```
<type>: <subject>

<body>

<footer>
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `style` - Formatting, no code change
- `refactor` - Code restructuring
- `test` - Adding tests
- `chore` - Maintenance tasks

### 6. Push and Create Pull Request

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create PR on GitHub
```

## Code Style

### Rust Style Guide

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

**Key points:**

1. **Use rustfmt**
   ```bash
   cargo fmt
   ```

2. **Naming conventions**
   ```rust
   // Modules: snake_case
   mod tool_registry;
   
   // Types: PascalCase
   struct AgentConfig;
   enum ModelProvider;
   
   // Functions/variables: snake_case
   fn execute_command() -> Result<()>;
   let user_input = "...";
   
   // Constants: SCREAMING_SNAKE_CASE
   const MAX_ITERATIONS: usize = 10;
   ```

3. **Documentation**
   ```rust
   /// Summary line (< 100 chars)
   ///
   /// More detailed explanation if needed.
   ///
   /// # Arguments
   ///
   /// * `arg1` - Description of arg1
   /// * `arg2` - Description of arg2
   ///
   /// # Returns
   ///
   /// Description of return value
   ///
   /// # Errors
   ///
   /// When this function errors
   ///
   /// # Examples
   ///
   /// ```
   /// let result = function(arg1, arg2)?;
   /// assert_eq!(result, expected);
   /// ```
   pub fn function(arg1: Type1, arg2: Type2) -> Result<ReturnType> {
       // Implementation
   }
   ```

4. **Error handling**
   ```rust
   // Use Result<T, E> for recoverable errors
   fn may_fail() -> Result<String, MyError> {
       // ...
   }
   
   // Use ? operator for propagation
   fn caller() -> Result<(), MyError> {
       let value = may_fail()?;
       Ok(())
   }
   
   // Add context to errors
   use anyhow::Context;
   
   fn read_config() -> Result<Config> {
       let content = std::fs::read_to_string("config.yaml")
           .context("Failed to read config file")?;
       // ...
   }
   ```

5. **Async functions**
   ```rust
   use tokio;
   
   #[tokio::test]
   async fn test_async_function() {
       let result = async_function().await.unwrap();
       assert_eq!(result, expected);
   }
   ```

### Project-Specific Conventions

1. **Module organization**
   ```rust
   // Public API at top
   pub use self::agent::Agent;
   pub use self::config::Config;
   
   // Internal modules
   mod agent;
   mod config;
   
   // Re-exports for convenience
   pub mod prelude {
       pub use super::Agent;
       pub use super::Config;
   }
   ```

2. **Trait implementations**
   ```rust
   // Implement in the same file as trait definition
   // Or in a separate impl file if complex
   
   // tool/mod.rs
   pub trait Tool {
       fn name(&self) -> &str;
   }
   
   // tool/shell.rs
   use super::Tool;
   
   pub struct ShellTool;
   
   impl Tool for ShellTool {
       fn name(&self) -> &str {
           "shell_execute"
       }
   }
   ```

3. **Configuration**
   ```rust
   // Use builder pattern for complex structs
   let agent = AgentBuilder::new()
       .model(model_provider)
       .tools(tool_registry)
       .config(config)
       .build()?;
   
   // Or use ..Default::default() for partial initialization
   let config = Config {
       max_iterations: 20,
       ..Default::default()
   };
   ```

## Testing Guidelines

### Test Coverage

- Aim for >80% coverage overall
- 100% coverage for safety-critical code
- Every public function should have tests

### Test Organization

```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name() {
        // Arrange
        let input = create_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/cli_integration.rs
use assert_cmd::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("promptline").unwrap();
    cmd.arg("--help").assert().success();
}
```

### Test Naming

```rust
// Pattern: test_<function>_<scenario>_<expected>
#[test]
fn test_validate_command_with_dangerous_pattern_returns_denied() {
    // ...
}

#[test]
fn test_parse_config_with_invalid_yaml_returns_error() {
    // ...
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_agent_loop

# With output
cargo test -- --nocapture

# Run ignored tests (e.g., API tests)
cargo test -- --ignored
```

## Pull Request Process

### Before Submitting

1. **Run all checks**
   ```bash
   # Format
   cargo fmt
   
   # Lint
   cargo clippy -- -D warnings
   
   # Tests
   cargo test
   
   # Security
   cargo audit
   ```

2. **Update documentation**
   - Add/update rustdoc comments
   - Update README.md if needed
   - Update CHANGELOG.md

3. **Self-review**
   - Read through your diff
   - Check for debug prints, commented code
   - Ensure commit messages are clear

### PR Template

When creating a PR, include:

```markdown
## Description

Brief description of the changes.

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Related Issues

Closes #123
Related to #456

## Changes Made

- Detailed list of changes
- Another change

## Testing

How was this tested?

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist

- [ ] Code follows project style guidelines
- [ ] Tests added for new functionality
- [ ] All tests passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No new compiler warnings
```

### Review Process

1. **Automated checks** must pass (CI/CD pipeline)
2. **Code review** by at least one maintainer
3. **Requested changes** should be addressed
4. **Approval** from maintainer
5. **Squash and merge** (maintainer will do this)

### Feedback

- Be patient - reviews take time
- Respond to all comments
- Push additional commits to the same branch
- Don't force-push after review starts

## Areas for Contribution

### Good First Issues

- Documentation improvements
- Adding tests
- Fixing typos
- Small bug fixes

### Feature Development

- New tool implementations
- Model provider integrations
- Prompt template system
- Performance optimizations

### Advanced Topics

- Plugin system architecture
- Sandboxing implementation
- Multi-agent coordination
- Streaming UI improvements

### Non-Code Contributions

- **Documentation** - Tutorials, guides, examples
- **Testing** - Manual testing, bug reports
- **Design** - UI/UX improvements, logos
- **Community** - Answer questions, help others

## Communication

### Channels

- **GitHub Issues** - Bug reports, feature requests
- **GitHub Discussions** - General questions, ideas
- **Discord** - Real-time chat (coming soon)
- **Email** - maintainer@promptline.dev

### Asking Questions

**Before asking:**
1. Check existing issues/discussions
2. Read the documentation
3. Search for similar problems

**When asking:**
- Provide context (what you're trying to do)
- Include error messages (full text)
- Share relevant code snippets
- Mention your environment (OS, Rust version)

### Reporting Bugs

Use the bug report template:

```markdown
## Description

Clear description of the bug.

## Steps to Reproduce

1. Run command X
2. See error Y

## Expected Behavior

What should happen.

## Actual Behavior

What actually happens.

## Environment

- OS: Ubuntu 22.04
- Rust: 1.70.0
- PromptLine: 0.1.0

## Additional Context

Logs, screenshots, etc.
```

### Suggesting Features

Use the feature request template:

```markdown
## Feature Description

What feature do you want?

## Use Case

Why is this feature useful?

## Proposed Solution

How could this work?

## Alternatives

Other ways to achieve the goal?
```

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Credited in documentation

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Questions?** Open an issue or start a discussion!

**Thank you for contributing to PromptLine!** 🙏
