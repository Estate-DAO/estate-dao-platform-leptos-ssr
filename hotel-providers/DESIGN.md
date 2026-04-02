# Hotel Providers - Multi-Provider Architecture

## Overview

The `hotel-providers` crate provides a multi-provider abstraction layer for hotel and place services, enabling:

- **Provider Abstraction**: Common traits for any hotel/place provider
- **Fallback Support**: Automatic failover between providers
- **Easy Extension**: Add new providers without modifying existing code

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         SSR Application                         │
├─────────────────────────────────────────────────────────────────┤
│  HotelService / PlaceService                                    │
│       │                                                         │
│       ▼                                                         │
│  init::get_liteapi_driver()  ──────► LiteApiDriver              │
│       │                                                         │
│       ▼                                                         │
│  ProviderRegistry ◄─────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                      hotel-providers crate                       │
├─────────────────────────────────────────────────────────────────┤
│  Providers                                                      │
│  ├── LiteApiDriver (implements HotelProviderPort, PlacePort)    │
│  │   ├── LiteApiClient (HTTP client)                            │
│  │   └── LiteApiMapper (domain ↔ API type conversion)           │
│  ├── BookingDriver (implements HotelProviderPort)               │
│  │   ├── BookingClient (HTTP client)                            │
│  │   └── BookingMapper (domain ↔ API type conversion)           │
│                                                                 │
│  Ports (Traits) - from hotel-types                              │
│  ├── HotelProviderPort                                          │
│  └── PlaceProviderPort                                          │
│                                                                 │
│  Registry                                                       │
│  └── ProviderRegistry (builder pattern)                         │
└─────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### Provider Traits

```rust
#[async_trait]
pub trait HotelProviderPort: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_healthy(&self) -> bool;
    
    async fn search_hotels(...) -> Result<..., ProviderError>;
    async fn get_hotel_rates(...) -> Result<..., ProviderError>;
    async fn block_room(...) -> Result<..., ProviderError>;
    async fn book_room(...) -> Result<..., ProviderError>;
    // ... more methods
}
```

### LiteApiDriver

The `LiteApiDriver` directly implements `HotelProviderPort` and `PlaceProviderPort`:

```rust
// hotel-providers/src/liteapi/driver.rs
impl HotelProviderPort for LiteApiDriver {
    fn name(&self) -> &'static str { "LiteAPI" }
    // ... all provider methods
}
```

## Integration with SSR

### Direct Driver Usage

SSR uses `ProviderRegistry` to compose providers with fallback:

```rust
// ssr/src/init.rs
pub fn initialize_provider_registry() {
    let liteapi = get_liteapi_driver();
    let booking = get_booking_driver();

    let registry = configure_place_provider(
        configure_hotel_providers(
            ProviderRegistry::builder(),
            &liteapi,
            &booking,
        ),
        &liteapi,
    )
    .build();
    // ...
}
```

### Usage in Server Functions

```rust
use crate::init::get_liteapi_driver;

pub async fn search_hotel_api(...) {
    let driver = get_liteapi_driver();
    let hotel_service = HotelService::new(driver);
    // ...
}
```

### Provider Identification

All provider-facing domain responses include an optional `provider` field (e.g., `"LiteAPI"`, `"Booking.com"`),
so the frontend can display or log which upstream handled the request. The booking flow can optionally include
`provider` in `DomainBookRoomRequest` to force the composite provider to route the booking to a specific
provider instead of defaulting to the primary.

For Booking.com, the `block_id` returned from preview is encoded to include both the
`order_token` and the preview `product_ids`, so the subsequent booking call can construct
the correct `orders/create` payload without extra storage.

## Adding a New Provider

1. **Create a new driver module** in `hotel-providers/src/`:

```rust
// hotel-providers/src/booking/driver.rs
pub struct BookingDriver { /* ... */ }

#[async_trait]
impl HotelProviderPort for BookingDriver {
    fn name(&self) -> &'static str { "Booking.com" }
    // ... implement all methods
}
```

2. **Register in the SSR crate**:

```rust
// ssr/src/init.rs
let primary = get_liteapi_driver();
let fallback = BookingDriver::new(...);

let registry = ProviderRegistry::builder()
    .with_hotel_provider(primary)
    .with_hotel_provider(fallback)  // Fallback
    .build();
```

## Error Handling

```rust
pub enum ProviderErrorKind {
    Network,           // Retryable
    Timeout,           // Retryable
    RateLimited,       // Retryable
    Unavailable,       // Should fallback
    InvalidRequest,    // Don't retry
    NotFound,          // Don't retry
    // ...
}
```

## Module Structure

```
hotel-providers/
├── Cargo.toml
└── src/
    ├── lib.rs              # Crate entry, re-exports
    ├── liteapi/            # LiteAPI integration
    │   ├── mod.rs
    │   ├── driver.rs       # LiteApiDriver
    │   ├── client.rs       # HTTP client
    │   ├── mapper.rs       # Type conversions
    │   └── models/         # LiteAPI-specific types
    ├── booking/            # Booking.com Demand API integration
    │   ├── mod.rs
    │   ├── driver.rs       # BookingDriver
    │   ├── client.rs       # HTTP client + mock
    │   ├── mapper.rs       # Type conversions
    │   └── models/         # Booking-specific types
    ├── composite.rs        # Composite providers (fallback)
    └── registry.rs         # ProviderRegistry builder
```

## Configuration

The driver reads configuration from environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `LITEAPI_KEY` | API key for LiteAPI | Required |
| `LITEAPI_ROOM_MAPPING` | Enable room type consolidation | `true` |
| `BOOKING_API_TOKEN` | Booking.com Demand API bearer token | Optional (required for real requests) |
| `BOOKING_AFFILIATE_ID` | Booking.com affiliate ID (`X-Affiliate-Id`) | Optional |
| `BOOKING_BASE_URL` | Booking.com Demand API base URL | `https://demandapi.booking.com/3.1` |
| `BOOKING_CURRENCY` | Currency for Booking.com requests | `USD` |
| `BOOKING_USE_MOCK` | Use mock Booking client (no real HTTP) | `true` when token is missing |

Primary provider ordering is configured in `ssr/src/init.rs`:
- `PRIMARY_HOTEL_PROVIDER` is the startup default for hotel provider priority.
- Hotel provider priority can be changed at runtime via admin API (`update_hotel_provider_config`).
- `PRIMARY_PLACE_PROVIDER` controls place provider selection (currently LiteAPI).
