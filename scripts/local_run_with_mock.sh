set -a 

source .env 

cargo leptos build --lib-features "local-lib-with-mock" --bin-features "local-bin-with-mock" || exit 1

export LEPTOS_SITE_ROOT="target/site"
export PAYMENTS_SKIP_LOCAL="true"

./target/debug/estate-fe

set +a 