# Load all vars from .env file into the env of just commands
set dotenv-load
# Export just vars as env vars
set export

# Default recipe to display help
default:
    @just --list

help:
    @echo "Just commands:"
    @just --list

# Development environment variables (from local_run.sh)
RUST_LOG := env_var_or_default("RUST_LOG", "estate_fe=debug,tower_http=debug")
RUST_BACKTRACE := env_var_or_default("RUST_BACKTRACE", "1")
LEPTOS_SITE_ROOT := env_var_or_default("LEPTOS_SITE_ROOT", "target/site")
PAYMENTS_SKIP_LOCAL := env_var_or_default("PAYMENTS_SKIP_LOCAL", "true")
LEPTOS_SITE_ADDR := env_var_or_default("LEPTOS_SITE_ADDR", "0.0.0.0:3002")
NOWPAYMENTS_API_HOST := env_var_or_default("NOWPAYMENTS_API_HOST", "http://localhost:3001")
NGROK_LOCALHOST_URL := env_var_or_default("NGROK_LOCALHOST_URL", "https://louse-musical-hideously.ngrok-free.app")

# Build features
LOCAL_LIB_FEATURES := "local-lib,debug_display"
LOCAL_BIN_FEATURES := "local-bin,debug_display"
MOCK_LIB_FEATURES := "local-lib,mock-provab,debug_display"
MOCK_BIN_FEATURES := "local-bin,mock-provab,debug_display"
RELEASE_LIB_FEATURES := "release-lib"
RELEASE_BIN_FEATURES := "release-bin"

# Setup logs directory and symlinks
setup-logs:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p "$(pwd)/logs"
    TODAY=$(date +'%Y-%m-%d')
    ln -sf "$(pwd)/logs/telemetry.log.${TODAY}" "$(pwd)/logs/estate-fe.log"
    echo "Logs directory setup complete. Today's log: logs/telemetry.log.${TODAY}"

# Type checking only
check:
    cargo leptos build --lib-features "{{LOCAL_LIB_FEATURES}}" --bin-features "{{LOCAL_BIN_FEATURES}}"

# Run local development server (equivalent to local_run.sh)
dev: setup-logs
    #!/usr/bin/env bash
    set -euo pipefail
    echo "NGROK_LOCALHOST_URL: ${NGROK_LOCALHOST_URL}"
    cargo leptos build --lib-features "{{LOCAL_LIB_FEATURES}}" --bin-features "{{LOCAL_BIN_FEATURES}}" || exit 1
    ./target/debug/estate-fe

# Run with debug display flag
dev-debug: setup-logs
    #!/usr/bin/env bash
    set -euo pipefail
    echo "NGROK_LOCALHOST_URL: ${NGROK_LOCALHOST_URL}"
    cargo leptos build --lib-features "local-lib,debug_display" --bin-features "local-bin,debug_display" || exit 1
    ./target/debug/estate-fe

# Run local development server with mocked APIs
dev-mock: setup-logs
    #!/usr/bin/env bash
    set -euo pipefail
    echo "NGROK_LOCALHOST_URL: ${NGROK_LOCALHOST_URL}"
    cargo leptos build --lib-features "{{MOCK_LIB_FEATURES}}" --bin-features "{{MOCK_BIN_FEATURES}}" || exit 1
    ./target/debug/estate-fe

# Run with mock block room failure
dev-mock-fail: setup-logs
    #!/usr/bin/env bash
    set -euo pipefail
    echo "NGROK_LOCALHOST_URL: ${NGROK_LOCALHOST_URL}"
    cargo leptos build --lib-features "local-lib,mock-provab,debug_display,mock-block-room-fail" --bin-features "local-bin,mock-provab,debug_display,mock-block-room-fail" || exit 1
    ./target/debug/estate-fe

# Build for staging
build-staging:
    cargo leptos build --lib-features "{{RELEASE_LIB_FEATURES}}" --bin-features "{{RELEASE_BIN_FEATURES}}" --release

# Run staging build
staging: build-staging
    ./target/release/estate-fe

# Build for production
build-prod:
    cargo leptos build --lib-features "release-lib-prod" --bin-features "release-bin-prod" --release

# Run production build
prod: build-prod
    ./target/release/estate-fe

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/

# Install pre-commit hooks
install-hooks:
    bash scripts/install_pre_commit.sh

# Run end-to-end tests
e2e:
    cargo leptos end-to-end

# Run specific test
test name:
    cargo test --lib {{name}} -- --nocapture

# View logs (tail the current log file)
logs:
    tail -f logs/estate-fe.log

# View logs with date
logs-date date:
    tail -f "logs/estate_fe_local.log.{{date}}"

# Deploy to production (via fly.io)
deploy-prod:
    bash scripts/prod_deploy.sh

# Send IPN webhook for testing
test-webhook:
    bash scripts/send_ipn_webhook.sh

# Show environment variables
show-env:
    @echo "RUST_LOG: {{RUST_LOG}}"
    @echo "RUST_BACKTRACE: {{RUST_BACKTRACE}}"
    @echo "LEPTOS_SITE_ROOT: {{LEPTOS_SITE_ROOT}}"
    @echo "LEPTOS_SITE_ADDR: {{LEPTOS_SITE_ADDR}}"
    @echo "NGROK_LOCALHOST_URL: {{NGROK_LOCALHOST_URL}}"
    @echo "NOWPAYMENTS_API_HOST: {{NOWPAYMENTS_API_HOST}}"
    @echo "PAYMENTS_SKIP_LOCAL: {{PAYMENTS_SKIP_LOCAL}}"

# Provab API testing
test-provab-cities:
    bash scripts/provab-api-test/00-city-list.sh

test-provab-search:
    bash scripts/provab-api-test/01-search.sh