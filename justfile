# Nix Pilot - Development Commands

# Default recipe - show help
default:
    @just --list

# Run the full development environment (API + UI)
dev:
    #!/usr/bin/env bash
    set -e
    echo "Starting Nix Pilot development servers..."

    # Kill any existing processes
    pkill -f "np-api" 2>/dev/null || true
    pkill -f "trunk serve" 2>/dev/null || true
    sleep 1

    # Start API server in background
    echo "Starting API server on http://localhost:3000..."
    cargo run -p np-api &
    API_PID=$!

    # Wait for API to be ready
    sleep 3

    # Start UI server
    echo "Starting UI server on http://localhost:8080..."
    cd np-ui && trunk serve --port 8080 &
    UI_PID=$!

    echo ""
    echo "Nix Pilot is running!"
    echo "  API: http://localhost:3000/api/health"
    echo "  UI:  http://localhost:8080"
    echo ""
    echo "Press Ctrl+C to stop all servers"

    # Wait for interrupt
    trap "kill $API_PID $UI_PID 2>/dev/null; exit 0" SIGINT SIGTERM
    wait

# Run only the API server
api:
    cargo run -p np-api

# Run only the UI development server
ui:
    cd np-ui && trunk serve --port 8080

# Build the entire project
build:
    cargo build --workspace

# Build for production
build-release:
    cargo build --release --workspace
    cd np-ui && trunk build --release

# Run tests
test:
    cargo test --workspace

# Run clippy
lint:
    cargo clippy --workspace -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check everything
check:
    cargo check --workspace
    cargo clippy --workspace
    cargo fmt --all -- --check

# Clean build artifacts
clean:
    cargo clean
    rm -rf np-ui/dist

# Generate a new age key for secrets
gen-key:
    #!/usr/bin/env bash
    mkdir -p ~/.config/sops/age
    age-keygen -o ~/.config/sops/age/keys.txt 2>&1 | tee /dev/stderr | grep "public key" | cut -d: -f2 | tr -d ' '

# Show age public key
show-key:
    @grep -h "public key:" ~/.config/sops/age/keys.txt 2>/dev/null | head -1 | cut -d: -f2 | tr -d ' ' || echo "No key found. Run 'just gen-key' first."
