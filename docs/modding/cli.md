# Modding: CLI Tools

The Colony Mod CLI provides comprehensive tools for creating, managing, and testing mods. This guide explains how to use the CLI tools effectively for mod development.

## Overview

The Colony Mod CLI provides:

- **Mod Creation**: Scaffold new mods with proper structure
- **Mod Validation**: Validate mod manifests and dependencies
- **Mod Building**: Build WASM modules and package mods
- **Mod Testing**: Test mods in isolation and integration
- **Mod Signing**: Sign mods for authenticity and integrity
- **Mod Management**: Manage mod installations and updates

## Installation

### Install the CLI

```bash
# Install from crates.io
cargo install colony-mod-cli

# Install from source
git clone https://github.com/colony-simulator/colony.git
cd colony
cargo install --path crates/colony-mod-cli
```

### Verify Installation

```bash
# Check CLI version
colony-mod --version

# Get help
colony-mod --help
```

## Basic Usage

### Command Structure

```bash
colony-mod <command> [options] [arguments]
```

### Available Commands

```bash
# Create a new mod
colony-mod new <mod-name>

# Validate a mod
colony-mod validate [path]

# Build a mod
colony-mod build [path]

# Test a mod
colony-mod test [path]

# Sign a mod
colony-mod sign [path]

# Install a mod
colony-mod install [path]

# Uninstall a mod
colony-mod uninstall <mod-name>

# List installed mods
colony-mod list

# Show mod information
colony-mod info <mod-name>

# Update a mod
colony-mod update <mod-name>

# Publish a mod
colony-mod publish [path]
```

## Mod Creation

### Create a New Mod

```bash
# Create a new mod with default template
colony-mod new my-awesome-mod

# Create a mod with specific template
colony-mod new my-awesome-mod --template wasm

# Create a mod with custom author
colony-mod new my-awesome-mod --author "Your Name" --email "your@email.com"

# Create a mod with specific capabilities
colony-mod new my-awesome-mod --capabilities "sim_time,enqueue_job,event_register"
```

### Mod Structure

The CLI creates a standard mod structure:

```
my-awesome-mod/
├── mod.toml              # Mod manifest
├── src/                  # Source code
│   ├── lib.rs           # Main library file
│   └── ops/             # WASM operations
├── lua/                  # Lua scripts
│   └── main.lua         # Main Lua script
├── assets/               # Mod assets
├── tests/                # Test files
├── docs/                 # Documentation
├── Cargo.toml           # Rust dependencies
└── README.md            # Mod documentation
```

### Mod Manifest

The CLI generates a basic mod manifest:

```toml
[mod]
name = "my-awesome-mod"
version = "0.1.0"
description = "A new mod created with colony-mod CLI"
author = "Your Name"
email = "your@email.com"
license = "MIT"
repository = "https://github.com/your-username/my-awesome-mod"

[mod.capabilities]
capabilities = [
    "sim_time",
    "sim_state",
    "event_register"
]

[mod.dependencies]
# Add dependencies here

[mod.build]
target = "wasm32-unknown-unknown"
features = ["default"]
```

## Mod Validation

### Validate Mod Structure

```bash
# Validate current directory
colony-mod validate

# Validate specific mod
colony-mod validate /path/to/mod

# Validate with verbose output
colony-mod validate --verbose

# Validate and show detailed errors
colony-mod validate --strict
```

### Validation Checks

The CLI performs comprehensive validation:

1. **Manifest Validation**: Check mod.toml syntax and content
2. **Dependency Validation**: Verify all dependencies are available
3. **Capability Validation**: Check capability declarations
4. **File Structure**: Validate mod file structure
5. **Code Validation**: Check Rust and Lua code syntax
6. **Asset Validation**: Validate asset files
7. **Security Checks**: Perform security validation

### Validation Output

```bash
$ colony-mod validate
Validating mod: my-awesome-mod
✓ Manifest validation passed
✓ Dependency validation passed
✓ Capability validation passed
✓ File structure validation passed
✓ Code validation passed
✓ Asset validation passed
✓ Security validation passed

Mod validation completed successfully!
```

## Mod Building

### Build WASM Modules

```bash
# Build current mod
colony-mod build

# Build specific mod
colony-mod build /path/to/mod

# Build with release optimizations
colony-mod build --release

# Build with specific target
colony-mod build --target wasm32-unknown-unknown

# Build with custom features
colony-mod build --features "gpu,network"
```

### Build Configuration

```bash
# Build with custom configuration
colony-mod build --config build.toml

# Build with environment variables
colony-mod build --env "RUST_LOG=debug"

# Build with custom output directory
colony-mod build --output /path/to/output
```

### Build Output

```bash
$ colony-mod build
Building mod: my-awesome-mod
✓ Compiling Rust code
✓ Building WASM module
✓ Optimizing WASM module
✓ Packaging mod
✓ Generating checksums

Build completed successfully!
Output: /path/to/my-awesome-mod/pkg/
```

## Mod Testing

### Test Mod Functionality

```bash
# Test current mod
colony-mod test

# Test specific mod
colony-mod test /path/to/mod

# Run specific test
colony-mod test --test unit_tests

# Run integration tests
colony-mod test --test integration_tests

# Run with coverage
colony-mod test --coverage
```

### Test Configuration

```bash
# Test with custom configuration
colony-mod test --config test.toml

# Test with specific environment
colony-mod test --env "TEST_MODE=integration"

# Test with verbose output
colony-mod test --verbose

# Test with parallel execution
colony-mod test --parallel
```

### Test Output

```bash
$ colony-mod test
Testing mod: my-awesome-mod
✓ Unit tests passed (5/5)
✓ Integration tests passed (3/3)
✓ WASM tests passed (2/2)
✓ Lua tests passed (4/4)
✓ Performance tests passed (1/1)

Test coverage: 95.2%
All tests passed successfully!
```

## Mod Signing

### Sign Mod for Distribution

```bash
# Sign current mod
colony-mod sign

# Sign specific mod
colony-mod sign /path/to/mod

# Sign with custom key
colony-mod sign --key /path/to/private.key

# Sign with passphrase
colony-mod sign --passphrase "your-passphrase"

# Sign with timestamp
colony-mod sign --timestamp
```

### Signing Configuration

```bash
# Sign with custom algorithm
colony-mod sign --algorithm "rsa-sha256"

# Sign with custom certificate
colony-mod sign --certificate /path/to/cert.pem

# Sign with custom output
colony-mod sign --output /path/to/signed-mod
```

### Signing Output

```bash
$ colony-mod sign
Signing mod: my-awesome-mod
✓ Generating signature
✓ Creating signature file
✓ Validating signature
✓ Updating manifest

Mod signed successfully!
Signature: /path/to/my-awesome-mod.sig
```

## Mod Management

### Install Mods

```bash
# Install from file
colony-mod install /path/to/mod.tar.gz

# Install from URL
colony-mod install https://example.com/mod.tar.gz

# Install with specific version
colony-mod install my-mod --version 1.2.3

# Install with dependencies
colony-mod install my-mod --with-dependencies
```

### List Installed Mods

```bash
# List all installed mods
colony-mod list

# List with details
colony-mod list --detailed

# List with versions
colony-mod list --versions

# List with status
colony-mod list --status
```

### Show Mod Information

```bash
# Show mod info
colony-mod info my-mod

# Show with dependencies
colony-mod info my-mod --dependencies

# Show with capabilities
colony-mod info my-mod --capabilities

# Show with files
colony-mod info my-mod --files
```

### Update Mods

```bash
# Update specific mod
colony-mod update my-mod

# Update all mods
colony-mod update --all

# Update with check
colony-mod update my-mod --check

# Update with backup
colony-mod update my-mod --backup
```

### Uninstall Mods

```bash
# Uninstall mod
colony-mod uninstall my-mod

# Uninstall with dependencies
colony-mod uninstall my-mod --with-dependencies

# Uninstall with confirmation
colony-mod uninstall my-mod --confirm

# Uninstall with backup
colony-mod uninstall my-mod --backup
```

## Advanced Usage

### Custom Templates

```bash
# Create mod with custom template
colony-mod new my-mod --template /path/to/template

# List available templates
colony-mod templates list

# Create custom template
colony-mod templates create my-template
```

### Configuration Files

```bash
# Use custom config file
colony-mod --config /path/to/config.toml build

# Show current configuration
colony-mod config show

# Set configuration value
colony-mod config set build.target wasm32-unknown-unknown

# Get configuration value
colony-mod config get build.target
```

### Environment Variables

```bash
# Set environment variables
export COLONY_MOD_CONFIG=/path/to/config.toml
export COLONY_MOD_CACHE=/path/to/cache
export COLONY_MOD_LOG_LEVEL=debug

# Use environment variables
colony-mod build
```

### Plugin System

```bash
# List available plugins
colony-mod plugins list

# Install plugin
colony-mod plugins install my-plugin

# Use plugin
colony-mod build --plugin my-plugin

# Remove plugin
colony-mod plugins remove my-plugin
```

## Troubleshooting

### Common Issues

1. **Build Failures**: Check Rust toolchain and dependencies
2. **Validation Errors**: Review mod manifest and file structure
3. **Test Failures**: Check test configuration and environment
4. **Signing Issues**: Verify key and certificate configuration
5. **Installation Problems**: Check permissions and dependencies

### Debug Mode

```bash
# Enable debug mode
colony-mod --debug build

# Set log level
colony-mod --log-level debug build

# Show verbose output
colony-mod --verbose build
```

### Error Reporting

```bash
# Report error
colony-mod report-error

# Show error details
colony-mod show-error <error-id>

# List recent errors
colony-mod errors list
```

## Best Practices

### Development Workflow

1. **Create Mod**: Use `colony-mod new` to create mod structure
2. **Develop**: Write code and tests
3. **Validate**: Use `colony-mod validate` to check mod
4. **Test**: Use `colony-mod test` to run tests
5. **Build**: Use `colony-mod build` to build mod
6. **Sign**: Use `colony-mod sign` to sign mod
7. **Install**: Use `colony-mod install` to install mod
8. **Publish**: Use `colony-mod publish` to publish mod

### Configuration Management

1. **Use Config Files**: Store configuration in files
2. **Version Control**: Track configuration changes
3. **Environment Variables**: Use environment variables for secrets
4. **Default Values**: Provide sensible defaults
5. **Validation**: Validate configuration values

### Error Handling

1. **Check Return Codes**: Always check command return codes
2. **Read Error Messages**: Read and understand error messages
3. **Use Debug Mode**: Enable debug mode for troubleshooting
4. **Report Issues**: Report issues with detailed information
5. **Document Solutions**: Document solutions for future reference

---

**The Colony Mod CLI provides powerful tools for mod development. Understanding these tools is key to efficient mod creation and management.** 🏭🛠️
