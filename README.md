# Estate DAO - Hotel Booking Platform

A Leptos-based hotel booking platform with multi-provider support and fallback capabilities.

## Project Structure

```
estate-dao-platform-leptos-ssr/
├── ssr/                    # Main Leptos SSR application
│   ├── src/
│   │   ├── adapters/       # Provider adapters (LiteAPI, bridge)
│   │   ├── api/            # API clients and server functions
│   │   ├── application_services/  # HotelService, PlaceService
│   │   ├── domain/         # Domain types
│   │   └── init.rs         # Global initialization
├── hotel-providers/        # Multi-provider abstraction crate
│   ├── src/
│   │   ├── domain/         # Provider-agnostic types
│   │   ├── ports/          # Provider traits (HotelProviderPort, PlaceProviderPort)
│   │   ├── adapters/       # Mock provider, LiteAPI placeholder
│   │   ├── composite.rs    # Fallback logic
│   │   └── registry.rs     # Provider configuration
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
| `hotel-providers` | Multi-provider abstraction with fallback support |
| `LiteApiAdapter` | Primary hotel/place provider (LiteAPI) |
| `LiteApiProviderBridge` | Bridge between SSR and hotel-providers traits |
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



--------------------------------------------



[ ] payment page design
[ ] booking confirmation page design
[ ] Destination - City, Country - on_select - CityId
[ ] 'See all photos' in center
_______________________________________________________________________

[x] room api call - de-duplication + take top 5

[x] hotel list - "no hotel found in center"
  git Branch - "block_room"

[] use_query in the room + hotel_details 
  https://github.com/Estate-DAO/estatedao_fe/blob/6175c04c0e7088d5414a8f2d9b69d07b1b216db2/ssr/src/component/destination_picker.rs#L80-L91

[] Destination - In a separate Branch - destination_dynamic

[] sort_by -- component -- in separate branch -- filters_sort_dynamic




Toggle dialog - Current: None, Requested: CityListComponent
 Dialog matches current - closing
 Setting dialog to: None
 is_open called
 Checking if destination is open: false
 is_open called
 Checking if destination is open: false






#[derive(Clone, Copy, Debug, Default)]
pub enum OpenDialogComponent{
    CityListComponent,   
    DateComponent, 
    GuestComponent,
    #[default] 
    None, 
}

impl OpenDialogComponent{
    pub fn matches(&self, other: OpenDialogComponent) -> bool {
    }

    pub fn is_destination_open(&self) -> bool {
    }

    pub fn is_date_open(&self) -> bool {
    }

    pub fn is_guest_open(&self) -> bool {
    }
}
