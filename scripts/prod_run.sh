source .env 

cargo leptos build --release || exit 1
LOCAL=true PROVAB_HEADERS=$PROVAB_HEADERS ./target/release/estate-fe


