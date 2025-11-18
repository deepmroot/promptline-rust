# Plugin System Architecture

*Status: Planned for Phase 4*

This document outlines the design for PromptLine's plugin system, enabling third-party extensions and community-contributed tools.

## Table of Contents

- [Overview](#overview)
- [Design Goals](#design-goals)
- [Plugin Types](#plugin-types)
- [Plugin Interface](#plugin-interface)
- [Plugin Discovery](#plugin-discovery)
- [Security Model](#security-model)
- [Plugin Development](#plugin-development)
- [Plugin Registry](#plugin-registry)

## Overview

The plugin system allows developers to extend PromptLine's capabilities without modifying core code:

```
┌─────────────────────────────────────┐
│         PromptLine Core             │
├─────────────────────────────────────┤
│        Plugin Manager               │
│  ┌───────────────────────────────┐  │
│  │    Plugin Loader & Sandbox    │  │
│  └───────────────────────────────┘  │
├─────────────────────────────────────┤
│           Plugin API                │
├─────────────────────────────────────┤
│  ┌──────┬──────┬──────┬──────────┐  │
│  │Tool  │Model │Prompt│Context   │  │
│  │Plugin│Plugin│Plugin│Plugin    │  │
│  └──────┴──────┴──────┴──────────┘  │
└─────────────────────────────────────┘
```

## Design Goals

1. **Safe by Default** - Plugins run in isolation, can't harm system
2. **Easy to Develop** - Simple API, good documentation
3. **Discoverable** - Central registry, easy installation
4. **Composable** - Plugins work together seamlessly
5. **Performant** - Minimal overhead from plugin system

## Plugin Types

### 1. Tool Plugins

Add new capabilities (e.g., database access, cloud APIs):

```rust
// Example: Docker plugin
pub struct DockerPlugin;

impl ToolPlugin for DockerPlugin {
    fn name(&self) -> &str {
        "docker"
    }
    
    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(DockerRunTool),
            Box::new(DockerBuildTool),
            Box::new(DockerPsTool),
        ]
    }
}
```

### 2. Model Provider Plugins

Integrate additional LLM providers:

```rust
// Example: Google PaLM plugin
pub struct PaLMPlugin;

impl ModelPlugin for PaLMPlugin {
    fn name(&self) -> &str {
        "google-palm"
    }
    
    fn provider(&self) -> Box<dyn LanguageModel> {
        Box::new(PaLMProvider::new(self.config))
    }
}
```

### 3. Prompt Template Plugins

Share reusable prompts:

```rust
// Example: Code review plugin
pub struct CodeReviewPlugin;

impl PromptPlugin for CodeReviewPlugin {
    fn templates(&self) -> Vec<PromptTemplate> {
        vec![
            PromptTemplate {
                name: "code-review",
                description: "Review code for issues",
                template: include_str!("templates/review.md"),
            },
        ]
    }
}
```

### 4. Context Provider Plugins

Add new context sources:

```rust
// Example: Jira integration
pub struct JiraPlugin;

impl ContextPlugin for JiraPlugin {
    fn name(&self) -> &str {
        "jira"
    }
    
    async fn get_context(&self, query: &str) -> Result<String> {
        // Fetch relevant Jira tickets
        let tickets = self.client.search(query).await?;
        Ok(format_tickets(tickets))
    }
}
```

## Plugin Interface

### Core Plugin Trait

```rust
/// Base trait all plugins must implement
pub trait Plugin: Send + Sync {
    /// Unique plugin identifier
    fn id(&self) -> &str;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// Plugin version
    fn version(&self) -> &str;
    
    /// Plugin description
    fn description(&self) -> &str;
    
    /// Required PromptLine version (semver)
    fn requires_version(&self) -> &str {
        ">=0.1.0"
    }
    
    /// Initialize plugin with config
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    
    /// Cleanup on unload
    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
```

### Tool Plugin Trait

```rust
#[async_trait]
pub trait ToolPlugin: Plugin {
    /// Register tools provided by this plugin
    fn tools(&self) -> Vec<Box<dyn Tool>>;
    
    /// Optional: Handle tool errors
    async fn on_tool_error(
        &self,
        tool: &str,
        error: &ToolError
    ) -> Result<RecoveryAction> {
        Ok(RecoveryAction::Propagate)
    }
}
```

### Model Plugin Trait

```rust
#[async_trait]
pub trait ModelPlugin: Plugin {
    /// Create model provider instance
    fn provider(&self) -> Box<dyn LanguageModel>;
    
    /// Check if provider is available
    async fn is_available(&self) -> bool {
        true
    }
}
```

## Plugin Discovery

### Loading Mechanisms

#### 1. Rust Crate Plugins (Compile-Time)

```toml
# Cargo.toml
[dependencies]
promptline-plugin-docker = "0.1"
```

```rust
// main.rs
use promptline_plugin_docker::DockerPlugin;

fn main() {
    let mut manager = PluginManager::new();
    manager.register_plugin(Box::new(DockerPlugin::new()));
}
```

#### 2. Dynamic Libraries (Runtime)

```yaml
# ~/.promptline/config.yaml
plugins:
  - path: ~/.promptline/plugins/docker.so
  - path: ~/.promptline/plugins/jira.dylib
```

```rust
impl PluginManager {
    pub fn load_dynamic(&mut self, path: &Path) -> Result<()> {
        use libloading::{Library, Symbol};
        
        // Load shared library
        let lib = unsafe { Library::new(path)? };
        
        // Get plugin constructor function
        let constructor: Symbol<fn() -> Box<dyn Plugin>> = unsafe {
            lib.get(b"create_plugin")?
        };
        
        // Create plugin instance
        let plugin = constructor();
        
        // Register
        self.register(plugin)?;
        
        // Keep library handle alive
        self.libraries.push(lib);
        
        Ok(())
    }
}
```

#### 3. WASI Plugins (WebAssembly)

```yaml
plugins:
  - wasm: ~/.promptline/plugins/database.wasm
    permissions:
      network: allow
      filesystem: read-only
```

```rust
use wasmtime::*;

impl PluginManager {
    pub async fn load_wasm(&mut self, path: &Path) -> Result<()> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, path)?;
        
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;
        
        // Create plugin wrapper
        let plugin = WasmPlugin::new(instance, store);
        
        self.register(Box::new(plugin))?;
        
        Ok(())
    }
}
```

#### 4. External Process Plugins

```yaml
plugins:
  - command: promptline-plugin-database
    args: ["--config", "db.yaml"]
    protocol: stdio  # JSON over stdin/stdout
```

```rust
impl ProcessPlugin {
    async fn call_tool(&self, tool: &str, args: Value) -> Result<ToolResult> {
        // Send request over stdin
        let request = json!({
            "method": "execute_tool",
            "tool": tool,
            "args": args,
        });
        
        self.stdin.write_all(&serde_json::to_vec(&request)?).await?;
        self.stdin.flush().await?;
        
        // Read response from stdout
        let mut line = String::new();
        self.stdout.read_line(&mut line).await?;
        
        let response: ToolResult = serde_json::from_str(&line)?;
        Ok(response)
    }
}
```

### Plugin Discovery

```rust
impl PluginManager {
    pub fn discover_plugins(&mut self) -> Result<Vec<PluginInfo>> {
        let mut plugins = Vec::new();
        
        // 1. Check standard locations
        let search_paths = vec![
            dirs::config_dir().unwrap().join("promptline/plugins"),
            dirs::data_dir().unwrap().join("promptline/plugins"),
            PathBuf::from("/usr/local/lib/promptline/plugins"),
        ];
        
        for dir in search_paths {
            if !dir.exists() {
                continue;
            }
            
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                // Check for plugin manifest
                if path.join("plugin.yaml").exists() {
                    let info = self.read_plugin_manifest(&path)?;
                    plugins.push(info);
                }
            }
        }
        
        Ok(plugins)
    }
}
```

## Security Model

### Capability-Based Security

Plugins declare required capabilities:

```yaml
# plugin.yaml
name: database-plugin
version: 0.1.0
capabilities:
  network:
    - "postgres://localhost:5432"  # Only this connection
  filesystem:
    read:
      - "~/.config/database/"
    write: []  # No write access
  environment:
    - "DATABASE_URL"  # Only this env var
```

### Sandboxing

Different isolation levels:

```rust
pub enum SandboxLevel {
    /// No sandboxing (trust plugin)
    None,
    
    /// Basic process isolation
    Process,
    
    /// WASM sandbox with capabilities
    Wasm,
    
    /// Full container isolation
    Container,
}
```

#### WASM Sandbox

```rust
use wasmtime::*;

impl WasmPlugin {
    pub fn new(wasm_path: &Path, capabilities: Capabilities) -> Result<Self> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        
        // Grant only specified capabilities
        if capabilities.filesystem.read {
            linker.func_wrap("env", "read_file", |path: &str| {
                // Validate path against allow-list
                if capabilities.can_read(path) {
                    std::fs::read_to_string(path)
                } else {
                    Err(anyhow!("Permission denied"))
                }
            })?;
        }
        
        // No write capability = no write function exposed
        
        let module = Module::from_file(&engine, wasm_path)?;
        let store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module)?;
        
        Ok(Self { instance, store })
    }
}
```

### Permission Prompts

```rust
impl PluginManager {
    async fn request_permission(
        &self,
        plugin: &str,
        capability: &Capability
    ) -> Result<bool> {
        println!("Plugin '{}' requests permission:", plugin);
        println!("  {:?}", capability);
        println!();
        println!("Allow? [y/N]: ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().eq_ignore_ascii_case("y"))
    }
}
```

### Signature Verification

```rust
impl PluginVerifier {
    pub fn verify_plugin(&self, path: &Path) -> Result<()> {
        // Read signature file
        let sig_path = path.with_extension("sig");
        let signature = std::fs::read(sig_path)?;
        
        // Read plugin binary
        let plugin_data = std::fs::read(path)?;
        
        // Verify with public key
        let public_key = self.load_trusted_keys()?;
        
        use ed25519_dalek::{Signature, Verifier};
        public_key.verify(&plugin_data, &Signature::from_bytes(&signature)?)?;
        
        Ok(())
    }
}
```

## Plugin Development

### Quick Start

```bash
# Create plugin project
cargo new --lib promptline-plugin-mytools
cd promptline-plugin-mytools

# Add dependency
cargo add promptline-plugin-api
```

### Example Plugin

```rust
// lib.rs
use promptline_plugin_api::*;

pub struct MyPlugin {
    config: PluginConfig,
}

impl Plugin for MyPlugin {
    fn id(&self) -> &str {
        "mytools"
    }
    
    fn name(&self) -> &str {
        "My Awesome Tools"
    }
    
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn description(&self) -> &str {
        "Collection of useful tools"
    }
    
    fn initialize(&mut self, config: PluginConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

#[async_trait]
impl ToolPlugin for MyPlugin {
    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(CalculatorTool),
            Box::new(DateTool),
        ]
    }
}

// For dynamic loading
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(MyPlugin {
        config: Default::default(),
    }))
}

// Calculator tool implementation
struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculate"
    }
    
    fn description(&self) -> &str {
        "Evaluate mathematical expressions"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Math expression to evaluate"
                }
            },
            "required": ["expression"]
        })
    }
    
    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let expr = args["expression"].as_str()
            .ok_or(anyhow!("Missing expression"))?;
        
        let result = evaluate_expression(expr)?;
        
        Ok(ToolResult {
            success: true,
            output: result.to_string(),
            ..Default::default()
        })
    }
}
```

### Building Plugin

```bash
# Rust crate (static linking)
cargo build --release

# Dynamic library
cargo build --release --crate-type cdylib

# WASM
cargo build --release --target wasm32-wasi
```

### Testing Plugin

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_loads() {
        let mut plugin = MyPlugin::default();
        plugin.initialize(PluginConfig::default()).unwrap();
        
        let tools = plugin.tools();
        assert_eq!(tools.len(), 2);
    }
    
    #[tokio::test]
    async fn test_calculator_tool() {
        let tool = CalculatorTool;
        
        let result = tool.execute(json!({
            "expression": "2 + 2"
        })).await.unwrap();
        
        assert_eq!(result.output, "4");
    }
}
```

## Plugin Registry

### Publishing Plugins

```bash
# Create plugin manifest
cat > plugin.yaml << EOF
name: my-awesome-plugin
version: 0.1.0
author: Your Name
description: Does cool things
homepage: https://github.com/you/plugin
repository: https://github.com/you/plugin
license: MIT
keywords: [tools, automation]
EOF

# Build
cargo build --release

# Publish to registry
promptline plugin publish
```

### Installing Plugins

```bash
# From registry
promptline plugin install database-tools

# From GitHub
promptline plugin install github:user/repo

# From local path
promptline plugin install ./my-plugin

# List installed
promptline plugin list

# Update
promptline plugin update database-tools

# Remove
promptline plugin remove database-tools
```

### Plugin Marketplace

Central registry at `https://plugins.promptline.dev`:

- Browse available plugins
- Read documentation and reviews
- Check compatibility
- View source code
- Report issues

## Best Practices

### For Plugin Developers

1. **Documentation** - Clear README with examples
2. **Testing** - Comprehensive test suite
3. **Error Handling** - Graceful failures
4. **Performance** - Minimal overhead
5. **Security** - Request minimal permissions
6. **Versioning** - Follow semver
7. **Compatibility** - Test with multiple PromptLine versions

### For Users

1. **Review Code** - Check plugin source before installing
2. **Check Permissions** - Understand what plugin can do
3. **Use Official Plugins** - Prefer verified plugins
4. **Keep Updated** - Update plugins regularly
5. **Report Issues** - Help improve plugins

## Future Enhancements

- **Hot Reloading** - Update plugins without restart
- **Plugin Dependencies** - Plugins can depend on other plugins
- **Plugin Marketplace UI** - Web interface for browsing
- **Signed Plugins** - Cryptographic verification
- **Plugin Analytics** - Usage statistics for developers
- **Plugin Sandboxing** - Enhanced isolation

---

**Status:** This is a design document for the planned plugin system. Implementation will begin in Phase 4 of development.

**See Also:**
- [Architecture](ARCHITECTURE.md) - Core system design
- [Contributing](CONTRIBUTING.md) - How to contribute
- [Roadmap](ROADMAP.md) - When this will be implemented
