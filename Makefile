# Rancer Makefile
# Common development commands for the Rancer digital art application

# Build targets
.PHONY: run build build-linux build-windows test clippy fmt doc clean

# Run the application
run:
	cargo run

# Build release binary
build:
	cargo build --release

# Build for Linux (requires GTK4)
build-linux:
	./build-linux.sh

# Build for Windows (uses cross-compile or native)
build-windows:
	./build-windows.bat

# Run all tests
test:
	cargo test

# Run clippy lints
clippy:
	cargo clippy --all-targets -- -D warnings

# Check formatting
fmt:
	cargo fmt --check

# Generate documentation
doc:
	cargo doc --no-deps

# Clean build artifacts
clean:
	cargo clean

# Additional CI commands (used in GitHub Actions)
.PHONY: ci

ci:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test