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
│  LiteAPI Integration                                            │
│  ├── LiteApiDriver (implements HotelProviderPort, PlacePort)    │
│  ├── LiteApiClient (HTTP client)                                │
│  └── LiteApiMapper (domain ↔ API type conversion)               │
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

SSR uses `LiteApiDriver` directly without any bridge/adapter layer:

```rust
// ssr/src/init.rs
pub fn get_liteapi_driver() -> LiteApiDriver {
    LITEAPI_DRIVER.get().expect("...").clone()
}

pub fn initialize_provider_registry() {
    let driver = get_liteapi_driver();
    
    let registry = ProviderRegistry::builder()
        .with_hotel_provider(driver.clone())
        .with_place_provider(driver)
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
    ├── composite.rs        # Composite providers (fallback)
    └── registry.rs         # ProviderRegistry builder
```

## Configuration

The driver reads configuration from environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `LITEAPI_KEY` | API key for LiteAPI | Required |
| `LITEAPI_ROOM_MAPPING` | Enable room type consolidation | `true` |
