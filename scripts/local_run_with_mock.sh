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

cargo leptos build --lib-features "local-lib-with-mock${DEBUG_DISPLAY}" --bin-features "local-bin-with-mock${DEBUG_DISPLAY}" || exit 1

export LEPTOS_SITE_ROOT="target/site"
export PAYMENTS_SKIP_LOCAL="true"

./target/debug/estate-fe

set +a