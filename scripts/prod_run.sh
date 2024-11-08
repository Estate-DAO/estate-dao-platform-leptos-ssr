source .env 

LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS cargo leptos build --release --lib-features "local-lib" --bin-features "local-bin" || exit 1
LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS ./target/release/estate-fe

