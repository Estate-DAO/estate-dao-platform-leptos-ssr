source .env 

LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS cargo leptos watch --lib-features "local-lib" --bin-features "local-bin"
# LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS cargo leptos serve
