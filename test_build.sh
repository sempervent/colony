#!/bin/bash
set -e

echo "Testing Compute Colony Simulator build..."

echo "Checking workspace..."
cargo check --workspace

echo "Building desktop app..."
cargo build --bin colony-desktop

echo "Building headless server..."
cargo build --bin colony-headless

echo "All builds successful! 🎉"
echo ""
echo "To run:"
echo "  Desktop: cargo run --bin colony-desktop"
echo "  Headless: cargo run --bin colony-headless"
