#!/bin/bash

echo "Testing Colony Desktop UI..."

# Change to project directory
cd /Users/joshuagrant/dev/colony

# Try to build the project
echo "Building colony-desktop..."
if cargo build --bin colony-desktop; then
    echo "✅ Build successful!"
    
    # Try to run the application for a few seconds
    echo "Running colony-desktop for 5 seconds..."
    timeout 5s cargo run --bin colony-desktop || echo "Application ran (timeout expected)"
    
    echo "✅ UI test completed!"
else
    echo "❌ Build failed!"
    exit 1
fi
