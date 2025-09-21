# Getting Started

Welcome to the Colony Simulator! This guide will help you get up and running quickly, whether you're a player, developer, or modder.

## 🎮 For Players

### Installation

#### Option 1: Pre-built Binaries (Recommended)

1. **Download** the latest release from [GitHub Releases](https://github.com/colony-simulator/colony/releases)
2. **Extract** the archive to your desired location
3. **Verify** the checksums: `sha256sum -c SHA256SUMS`
4. **Run** the desktop application: `./colony-desktop`

#### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/colony-simulator/colony.git
cd colony

# Build the desktop application
cargo build --release --bin colony-desktop

# Run the application
./target/release/colony-desktop
```

### First Steps

1. **Launch** the desktop application
2. **Select** "First Light Chill" scenario (recommended for beginners)
3. **Configure** your colony:
   - Set power cap to 1000 kW
   - Allocate 4 CPU workers
   - Set bandwidth to 32 Gbps
4. **Start** the simulation
5. **Monitor** the dashboard for:
   - Power consumption vs. capacity
   - Heat levels in workyards
   - Bandwidth utilization
   - Corruption field growth

### Quick Tips

- **Watch the power meter** - don't exceed your power capacity
- **Monitor heat levels** - high heat reduces efficiency
- **Keep corruption low** - it increases fault rates
- **Research early** - unlock new capabilities
- **Save frequently** - use the autosave feature

## 👨‍💻 For Developers

### Prerequisites

- **Rust** 1.70+ with Cargo
- **Git** for version control
- **Make** (optional, for build automation)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/colony-simulator/colony.git
cd colony

# Install development dependencies
make install-deps

# Build all crates
make build

# Run tests
make test

# Run linting
make lint

# Format code
make fmt
```

### Project Structure

```
colony/
├── crates/
│   ├── colony-core/          # Core simulation engine
│   ├── colony-desktop/       # Desktop GUI application
│   ├── colony-headless/      # Headless server
│   ├── colony-mod-cli/       # Mod development CLI
│   ├── colony-modsdk/        # Modding SDK
│   └── xtask/                # Build and verification tools
├── docs/                     # Documentation
├── mods/                     # Example mods
├── scripts/                  # Build and utility scripts
└── tests/                    # Integration tests
```

### Development Workflow

1. **Create** a feature branch: `git checkout -b feature/my-feature`
2. **Implement** your changes
3. **Test** locally: `make test`
4. **Lint** your code: `make lint`
5. **Format** your code: `make fmt`
6. **Commit** with descriptive messages
7. **Push** and create a pull request

### Running Tests

```bash
# Run all tests
cargo test --workspace --all-features

# Run specific test suite
cargo test --package colony-core --test unit_tests

# Run integration tests
cargo test --package colony-headless --test e2e_integration

# Run benchmarks
cargo bench --workspace --all-features
```

### Debugging

- **Use** `RUST_LOG=debug` for verbose logging
- **Enable** debug builds for better error messages
- **Use** the replay system to debug deterministic issues
- **Check** the [Debugging Guide](kb/debugging.md) for more tips

## 🔧 For Modders

### Prerequisites

- **Rust** 1.70+ with Cargo
- **WASM** toolchain (for WASM mods)
- **Lua** 5.4+ (for Lua mods)

### Installation

```bash
# Install the mod CLI
cargo install colony-mod

# Or build from source
cargo build --release --bin colony-mod
```

### Creating Your First Mod

```bash
# Create a new mod
colony-mod new com.example.mymod

# Navigate to the mod directory
cd com.example.mymod

# Edit the mod manifest
vim mod.toml

# Add your code
vim src/lib.rs  # For WASM mods
vim scripts/on_tick.lua  # For Lua mods

# Validate the mod
colony-mod validate .

# Test the mod
colony-mod test .
```

### Mod Structure

```
com.example.mymod/
├── mod.toml                  # Mod manifest
├── src/
│   └── lib.rs               # WASM operations
├── scripts/
│   ├── on_tick.lua          # Lua event handlers
│   └── on_init.lua
├── assets/                  # Mod assets
└── README.md               # Mod documentation
```

### Mod Development Workflow

1. **Create** your mod: `colony-mod new com.example.mymod`
2. **Edit** the manifest: `mod.toml`
3. **Implement** your code: WASM ops or Lua events
4. **Validate** your mod: `colony-mod validate .`
5. **Test** your mod: `colony-mod test .`
6. **Package** your mod: `colony-mod package .`
7. **Share** your mod with the community

## 🚀 Headless Server

### Running the Headless Server

```bash
# Build the headless server
cargo build --release --bin colony-headless

# Run with default settings
./target/release/colony-headless

# Run with custom port
./target/release/colony-headless --port 8080

# Run with specific scenario
./target/release/colony-headless --scenario first_light_chill
```

### API Endpoints

The headless server provides a REST API:

- `GET /health` - Health check
- `GET /session/status` - Session status
- `POST /session/start` - Start a new session
- `POST /session/pause` - Pause the session
- `GET /metrics/summary` - Get simulation metrics
- `GET /mods` - List installed mods
- `POST /mods/enable` - Enable/disable a mod

### Example API Usage

```bash
# Start a session
curl -X POST http://localhost:8080/session/start \
  -H "Content-Type: application/json" \
  -d '{"scenario_id": "first_light_chill", "tick_scale": "RealTime"}'

# Get metrics
curl http://localhost:8080/metrics/summary

# Pause the session
curl -X POST http://localhost:8080/session/pause
```

## 🛠️ Build System

### Makefile Targets

```bash
make help          # Show available targets
make build         # Build all crates
make test          # Run all tests
make e2e           # Run end-to-end tests
make bench         # Run benchmarks
make lint          # Run linting
make fmt           # Format code
make audit         # Security audit
make verify        # Full verification suite
make rc            # Build release candidate
make clean         # Clean build artifacts
```

### Feature Flags

The build system supports several feature flags:

- `desktop` - Enable desktop GUI features
- `can_real` - Enable real CAN bus hardware
- `modbus_real` - Enable real Modbus hardware

```bash
# Build with desktop features
make build DESKTOP=1

# Build with all features
make build DESKTOP=1 CAN_REAL=1 MODBUS_REAL=1
```

## 🔍 Troubleshooting

### Common Issues

#### Build Failures

```bash
# Clean and rebuild
make clean
make build

# Check Rust version
rustc --version  # Should be 1.70+

# Update dependencies
cargo update
```

#### Runtime Errors

```bash
# Enable debug logging
RUST_LOG=debug ./colony-desktop

# Check system requirements
./colony-desktop --check-requirements
```

#### Mod Issues

```bash
# Validate mod structure
colony-mod validate com.example.mymod

# Check mod dependencies
colony-mod deps com.example.mymod

# Test mod in isolation
colony-mod test com.example.mymod
```

### Getting Help

- **Documentation**: Browse the [Knowledge Base](kb/perf-tuning.md)
- **GitHub Issues**: [Report bugs](https://github.com/colony-simulator/colony/issues)
- **Discord**: [Join our community](https://discord.gg/colony-simulator)
- **Discussions**: [Community discussions](https://github.com/colony-simulator/colony/discussions)

## 📚 Next Steps

Now that you're set up, explore the documentation:

- **Players**: Start with the [Player Manual](player-manual/basics.md)
- **Developers**: Dive into the [Developer Docs](developer-docs/architecture.md)
- **Modders**: Follow the [Modding Guide](modding/intro.md)

Welcome to the Colony! 🏭✨
