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
| `hotel-providers` | Multi-provider abstraction with LiteAPI + Booking.com integration |
| `hotel-types` | Shared domain types and provider port traits |
| `LiteApiDriver` | LiteAPI provider (implements `HotelProviderPort` + `PlaceProviderPort`) |
| `BookingDriver` | Booking.com Demand API provider (implements HotelProviderPort) |
| `ProviderRegistry` | Configures and manages providers |

## Environment Variables

```bash
LEPTOS_OUTPUT_NAME="estate-fe"
LEPTOS_SITE_ROOT="site"
LEPTOS_SITE_PKG_DIR="pkg"
LEPTOS_SITE_ADDR="127.0.0.1:3000"
LEPTOS_RELOAD_PORT="3001"
LITEAPI_KEY="..."
LITEAPI_ROOM_MAPPING="true"
BOOKING_API_TOKEN="..."
BOOKING_AFFILIATE_ID="..."
BOOKING_BASE_URL="https://demandapi.booking.com/3.1"
BOOKING_CURRENCY="USD"
BOOKING_USE_MOCK="true"
```

Provider identification:
- Hotel and place API responses include an optional `provider` field (e.g., `LiteAPI`, `Booking.com`).
- Booking requests may include `provider` to route to a specific upstream when multiple providers are configured.
- Primary hotel provider default is compile-time in `ssr/src/init.rs` via `PRIMARY_HOTEL_PROVIDER`.
- Admin can update primary hotel provider at runtime via `POST /server_fn_api/admin/update_hotel_provider_config`.
- Primary place provider is compile-time and configured in `ssr/src/init.rs` via `PRIMARY_PLACE_PROVIDER` (currently `LiteApi`).

## Testing

```bash
cargo leptos end-to-end
cargo leptos end-to-end --release
```

## Deployment

After `cargo leptos build --release`, copy:
1. Server binary: `target/server/release/estate-fe`
2. Site directory: `target/site/`
