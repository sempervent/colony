# Colony Simulator - Build and Verification Makefile
# M8: End-to-End Verification & Release Candidate

.PHONY: help build test e2e bench lint fmt audit verify rc clean install-deps

# Default target
help:
	@echo "Colony Simulator Build System"
	@echo "============================="
	@echo ""
	@echo "Available targets:"
	@echo "  build        - Build all crates"
	@echo "  test         - Run unit tests"
	@echo "  e2e          - Run end-to-end integration tests"
	@echo "  bench        - Run performance benchmarks"
	@echo "  lint         - Run clippy and format checks"
	@echo "  fmt          - Format all code"
	@echo "  audit        - Run security audit"
	@echo "  verify       - Run complete verification suite"
	@echo "  rc           - Build release candidate"
	@echo "  clean        - Clean build artifacts"
	@echo "  install-deps - Install required dependencies"
	@echo ""
	@echo "Feature flags:"
	@echo "  DESKTOP=1    - Enable desktop features"
	@echo "  CAN_REAL=1   - Enable real CAN hardware"
	@echo "  MODBUS_REAL=1 - Enable real Modbus hardware"
	@echo ""
	@echo "Examples:"
	@echo "  make verify"
	@echo "  make build DESKTOP=1"
	@echo "  make rc VERSION=v0.9.0-rc1"

# Feature flags
DESKTOP ?= 0
CAN_REAL ?= 0
MODBUS_REAL ?= 0

# Build features
FEATURES :=
ifeq ($(DESKTOP),1)
	FEATURES += desktop
endif
ifeq ($(CAN_REAL),1)
	FEATURES += can_real
endif
ifeq ($(MODBUS_REAL),1)
	FEATURES += modbus_real
endif

FEATURE_FLAGS := $(if $(FEATURES),--features "$(shell echo $(FEATURES) | tr ' ' ',')")

# Build configuration
CARGO_FLAGS := $(FEATURE_FLAGS)
CARGO_TEST_FLAGS := $(FEATURE_FLAGS) --all-features
CARGO_BENCH_FLAGS := $(FEATURE_FLAGS) --all-features

# Version for RC builds
VERSION ?= v0.9.0-rc1

# Install dependencies
install-deps:
	@echo "Installing dependencies..."
	cargo install cargo-nextest
	cargo install cargo-audit
	cargo install cargo-deny
	cargo install cargo-criterion
	cargo install wasm-pack
	@echo "Dependencies installed."

# Build all crates
build:
	@echo "Building Colony Simulator..."
	cargo build $(CARGO_FLAGS) --workspace
	@echo "Build complete."

# Run unit tests
test:
	@echo "Running unit tests..."
	cargo test $(CARGO_TEST_FLAGS) --workspace
	@echo "Unit tests complete."

# Run tests with nextest for speed
test-fast:
	@echo "Running fast tests with nextest..."
	cargo nextest run $(CARGO_TEST_FLAGS) --workspace
	@echo "Fast tests complete."

# Run end-to-end integration tests
e2e:
	@echo "Running end-to-end integration tests..."
	cargo run -p xtask -- e2e
	@echo "E2E tests complete."

# Run performance benchmarks
bench:
	@echo "Running performance benchmarks..."
	cargo bench $(CARGO_BENCH_FLAGS) --workspace
	@echo "Benchmarks complete."

# Run linting and format checks
lint:
	@echo "Running linting checks..."
	cargo fmt -- --check
	cargo clippy $(CARGO_FLAGS) --workspace -- -D warnings
	@echo "Linting complete."

# Format all code
fmt:
	@echo "Formatting code..."
	cargo fmt --all
	@echo "Code formatted."

# Run security audit
audit:
	@echo "Running security audit..."
	cargo audit
	cargo deny check
	@echo "Security audit complete."

# Run complete verification suite
verify:
	@echo "Running complete verification suite..."
	cargo run -p xtask -- verify
	@echo "Verification complete."

# Build release candidate
rc:
	@echo "Building release candidate $(VERSION)..."
	cargo run -p xtask -- rc --version $(VERSION)
	@echo "Release candidate $(VERSION) built."

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf target/verify/
	rm -rf target/rc/
	@echo "Clean complete."

# Quick verification (no benchmarks)
verify-lite:
	@echo "Running lite verification..."
	cargo fmt -- --check
	cargo clippy $(CARGO_FLAGS) --workspace -- -D warnings
	cargo test $(CARGO_TEST_FLAGS) --workspace
	cargo run -p xtask -- e2e
	@echo "Lite verification complete."

# Development setup
dev-setup: install-deps
	@echo "Setting up development environment..."
	mkdir -p mods/vanilla
	mkdir -p saves
	mkdir -p logs
	@echo "Development setup complete."

# CI-specific targets
ci-lint:
	cargo fmt -- --check
	cargo clippy $(CARGO_FLAGS) --workspace -- -D warnings

ci-test:
	cargo test $(CARGO_TEST_FLAGS) --workspace --lib

ci-integration:
	cargo run -p xtask -- e2e

ci-security:
	cargo audit
	cargo deny check

ci-bench:
	cargo bench $(CARGO_BENCH_FLAGS) --workspace -- --quick

# Help for CI
ci-help:
	@echo "CI targets:"
	@echo "  ci-lint        - Format and clippy checks"
	@echo "  ci-test        - Unit tests only"
	@echo "  ci-integration - Integration tests"
	@echo "  ci-security    - Security audit"
	@echo "  ci-bench       - Quick benchmarks"
