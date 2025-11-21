#!/bin/bash

# Apiary - Honeycomb CLI Build and Setup Script
#
# This script sets up the development environment and builds the apiary CLI tool

set -e

echo "🐝 Setting up Apiary - Honeycomb CLI"
echo "===================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "📦 Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✅ Rust installed successfully!"
else
    echo "✅ Rust is already installed"
fi

# Update Rust if needed
echo "🔄 Updating Rust toolchain..."
rustup update

# Build the project
echo "🔨 Building apiary..."
cargo build --release

# Check if build was successful
if [ -f "target/release/apiary" ]; then
    echo "✅ Build successful!"
    echo "📍 Binary location: target/release/apiary"
    
    # Show help output
    echo ""
    echo "🚀 Apiary CLI Help:"
    echo "=================="
    ./target/release/apiary --help
    
    # Suggest installation
    echo ""
    echo "💡 To install globally, run:"
    echo "   cp target/release/apiary /usr/local/bin/"
    echo "   # or add to your PATH:"
    echo "   export PATH=\"\$PATH:$(pwd)/target/release\""
else
    echo "❌ Build failed!"
    exit 1
fi

echo ""
echo "🎉 Setup complete! You can now use apiary to interact with the Honeycomb API."
echo ""
echo "📚 Quick start examples:"
echo "  export HONEYCOMB_API_KEY='your-api-key'"
echo "  ./target/release/apiary auth validate"
echo "  ./target/release/apiary datasets list"
echo "  ./target/release/apiary --help"