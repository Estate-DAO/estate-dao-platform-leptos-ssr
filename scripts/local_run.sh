set -a 
source .env 
# export $(cat .env | grep -v '^#' | tr  '\n' ' ' | xargs)

# PROVAB_HEADERS=$PROVAB_HEADERS cargo leptos watch --lib-features "local-lib" --bin-features "local-bin"
cargo leptos serve --lib-features "local-lib" --bin-features "local-bin"
#  PROVAB_HEADERS=$PROVAB_HEADERS cargo leptos serve
set +a