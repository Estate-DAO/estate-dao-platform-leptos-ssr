# Estate DAO - Hotel Booking Platform

A Leptos-based hotel booking platform with multi-provider support and fallback capabilities.

## Project Structure

```
estate-dao-platform-leptos-ssr/
├── ssr/                    # Main Leptos SSR application
│   ├── src/
│   │   ├── api/            # API clients and server functions
│   │   ├── application_services/  # HotelService, PlaceService
│   │   ├── domain/         # Domain types (re-exports from hotel-types)
│   │   └── init.rs         # Global initialization
├── hotel-providers/        # Multi-provider abstraction crate
│   ├── src/
│   │   ├── liteapi/        # LiteAPI driver, client, mapper
│   │   ├── composite.rs    # Fallback logic
│   │   └── registry.rs     # Provider configuration
├── hotel-types/            # Shared domain types and port traits
└── telemetry_axum/         # Telemetry utilities
```

## Quick Start

### Install pre-commit hooks
```bash
bash scripts/install_pre_commit.sh
```

### Install dependencies
```bash
cargo install cargo-leptos --locked
rustup target add wasm32-unknown-unknown
```

### Run development server
```bash
cargo leptos watch
```

### Build for production
```bash
cargo leptos build --release
```

## Architecture

See [hotel-providers/DESIGN.md](hotel-providers/DESIGN.md) for the multi-provider architecture documentation.

### Key Components

| Component | Description |
|-----------|-------------|
| `hotel-providers` | Multi-provider abstraction with LiteAPI integration |
| `hotel-types` | Shared domain types and provider port traits |
| `LiteApiDriver` | Primary hotel/place provider (implements HotelProviderPort) |
| `ProviderRegistry` | Configures and manages providers |

## Environment Variables

```bash
LEPTOS_OUTPUT_NAME="estate-fe"
LEPTOS_SITE_ROOT="site"
LEPTOS_SITE_PKG_DIR="pkg"
LEPTOS_SITE_ADDR="127.0.0.1:3000"
LEPTOS_RELOAD_PORT="3001"
```

## Testing

```bash
cargo leptos end-to-end
cargo leptos end-to-end --release
```

## Deployment

After `cargo leptos build --release`, copy:
1. Server binary: `target/server/release/estate-fe`
2. Site directory: `target/site/`
