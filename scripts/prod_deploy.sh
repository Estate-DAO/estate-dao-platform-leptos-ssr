cargo leptos build --release --lib-features "release-lib-prod" --bin-features "release-bin-prod" 

export FLY_API_TOKEN=$(cat .env.fly)
FLY_API_TOKEN=$FLY_API_TOKEN  fly deploy
