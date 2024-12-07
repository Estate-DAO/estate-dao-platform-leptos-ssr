set -a 

source .env 

# LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS cargo leptos watch --lib-features "local-lib" --bin-features "local-bin"
cargo leptos build --lib-features "local-lib" --bin-features "local-bin" || exit 1

# # LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS
LEPTOS_SITE_ROOT="target/site"
PAYMENTS_SKIP_LOCAL="true"

./target/debug/estate-fe --features "local-bin"

# LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS cargo leptos serve

set +a 