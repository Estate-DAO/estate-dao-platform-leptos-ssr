set +a 

source .env 

cargo leptos build --release --lib-features "release-lib-prod" --bin-features "release-bin-prod" || exit 1
ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY=$ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY PROVAB_HEADERS=$PROVAB_HEADERS NOW_PAYMENTS_USDC_ETHEREUM_API_KEY=$NOW_PAYMENTS_USDC_ETHEREUM_API_KEY ./target/release/estate-fe



 set -a 