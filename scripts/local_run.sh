#!/bin/bash
set -a 

source .env


# Initialize debug_display flag
DEBUG_DISPLAY=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --debug_display)
            DEBUG_DISPLAY=",debug_display"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# cargo leptos build --lib-features "local-lib${DEBUG_DISPLAY}" --bin-features "local-bin${DEBUG_DISPLAY}" || exit 1
# cargo leptos build --lib-features "local-lib,mock-provab" --bin-features "local-bin,mock-provab" || exit 1
# cargo leptos build --lib-features "local-lib,mock-provab,debug_display" --bin-features "local-bin,mock-provab,debug_display" || exit 1
# cargo leptos build --lib-features "local-lib,mock-provab,debug_display,mock-block-room-fail" --bin-features "local-bin,mock-provab,debug_display,mock-block-room-fail" || exit 1
export RUST_LOG="estate_fe=debug,tower_http=debug"
export RUST_BACKTRACE=1
export LEPTOS_SITE_ROOT="target/site"
export PAYMENTS_SKIP_LOCAL="true"
export LEPTOS_SITE_ADDR="0.0.0.0:3002"
export NOWPAYMENTS_API_HOST="http://localhost:3001"

echo "NGROK_LOCALHOST_URL: $NGROK_LOCALHOST_URL"
export NGROK_LOCALHOST_URL="https://louse-musical-hideously.ngrok-free.app"

cargo leptos build --lib-features "local-lib,debug_display,mock-provab" --bin-features "local-bin,debug_display,mock-provab" || exit 1
# cargo leptos build --lib-features "local-lib,mock-provab,debug_display" --bin-features "local-bin,mock-provab,debug_display" || exit 1


# # LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS
./target/debug/estate-fe

# LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS cargo leptos serve

set +a