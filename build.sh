#!/bin/bash

# Build script for ALECC compiler

set -e

echo "Building ALECC compiler..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: This script must be run from the project root directory"
    exit 1
fi

# Build in release mode for optimal performance
echo "Building in release mode..."
cargo build --release

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "ALECC compiler binary is available at: target/release/alecc"
    
    # Optionally install to system
    read -p "Do you want to install ALECC to /usr/local/bin? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        sudo cp target/release/alecc /usr/local/bin/
        echo "ALECC installed to /usr/local/bin/alecc"
    fi
else
    echo "Build failed!"
    exit 1
fi
