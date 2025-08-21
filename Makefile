# Makefile for ALECC

.PHONY: all build clean test bench install uninstall help

# Default target
all: build

# Build the project
build:
	cargo build --release

# Build in debug mode
debug:
	cargo build

# Clean build artifacts
clean:
	cargo clean

# Run tests
test:
	cargo test

# Run benchmarks
bench:
	cargo bench

# Check code without building
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run clippy lints
clippy:
	cargo clippy -- -D warnings

# Install to system
install: build
	sudo cp target/release/alecc /usr/local/bin/

# Uninstall from system
uninstall:
	sudo rm -f /usr/local/bin/alecc

# Build documentation
docs:
	cargo doc --open

# Run all quality checks
qa: fmt clippy test

# Show help
help:
	@echo "Available targets:"
	@echo "  all       - Build the project (default)"
	@echo "  build     - Build in release mode"
	@echo "  debug     - Build in debug mode"
	@echo "  clean     - Clean build artifacts"
	@echo "  test      - Run tests"
	@echo "  bench     - Run benchmarks"
	@echo "  check     - Check code without building"
	@echo "  fmt       - Format code"
	@echo "  clippy    - Run clippy lints"
	@echo "  install   - Install to /usr/local/bin"
	@echo "  uninstall - Remove from /usr/local/bin"
	@echo "  docs      - Build and open documentation"
	@echo "  qa        - Run quality assurance checks"
	@echo "  help      - Show this help"
