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
│  get_liteapi_adapter()  ──────► LiteApiAdapter                  │
│       │                              │                          │
│       ▼                              ▼                          │
│  ProviderRegistry ◄───────── LiteApiProviderBridge              │
└─────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                      hotel-providers crate                       │
├─────────────────────────────────────────────────────────────────┤
│  Ports (Traits)                                                 │
│  ├── HotelProviderPort                                          │
│  └── PlaceProviderPort                                          │
│                                                                 │
│  Composite Providers (Fallback)                                 │
│  ├── CompositeHotelProvider                                     │
│  └── CompositePlaceProvider                                     │
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

### Fallback Strategies

| Strategy | Behavior |
|----------|----------|
| `Sequential` | Try providers in order until one succeeds |
| `OnRetryableError` | Only fallback on network/timeout errors |
| `NeverFallback` | Use only the primary provider |

### Provider Registry

```rust
let registry = ProviderRegistry::builder()
    .with_hotel_provider(primary_provider)
    .with_hotel_provider(fallback_provider)
    .with_fallback_strategy(FallbackStrategy::Sequential)
    .build();

// Access providers
let hotel_provider = registry.hotel_provider();
```

## Integration with SSR

### Current Architecture (Bridge Pattern)

The SSR crate uses `LiteApiProviderBridge` to wrap the existing `LiteApiAdapter`:

```rust
// ssr/src/init.rs
pub fn get_liteapi_adapter() -> LiteApiAdapter {
    LiteApiAdapter::new(get_liteapi_client().clone())
}

pub fn initialize_provider_registry() {
    let bridge = LiteApiProviderBridge::from_client(get_liteapi_client().clone());
    
    let registry = ProviderRegistry::builder()
        .with_hotel_provider(bridge.clone())
        .with_place_provider(bridge)
        .build();
    // ...
}
```

### Usage in Server Functions

```rust
use crate::init::get_liteapi_adapter;

pub async fn search_hotel_api(...) {
    let adapter = get_liteapi_adapter();
    let hotel_service = HotelService::new(adapter);
    // ...
}
```

## Adding a New Provider

1. **Implement the traits** in `hotel-providers/src/adapters/`:

```rust
pub struct NewProviderAdapter { /* ... */ }

#[async_trait]
impl HotelProviderPort for NewProviderAdapter {
    fn name(&self) -> &'static str { "NewProvider" }
    // ... implement all methods
}
```

2. **Register in the SSR crate**:

```rust
// ssr/src/init.rs
let primary = LiteApiProviderBridge::from_client(client);
let fallback = NewProviderAdapter::new();

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

Error kinds determine whether to trigger fallback:

```rust
impl ProviderError {
    pub fn should_fallback(&self) -> bool {
        matches!(self.kind, 
            ProviderErrorKind::Network |
            ProviderErrorKind::Timeout |
            ProviderErrorKind::Unavailable
        )
    }
}
```

## Module Structure

```
hotel-providers/
├── Cargo.toml
└── src/
    ├── lib.rs              # Crate entry, re-exports
    ├── domain/
    │   ├── mod.rs
    │   ├── hotel_search_types.rs
    │   └── booking_types.rs
    ├── ports/
    │   ├── mod.rs
    │   ├── hotel_port.rs   # HotelProviderPort trait
    │   ├── place_port.rs   # PlaceProviderPort trait
    │   └── error.rs        # ProviderError, ProviderErrorKind
    ├── adapters/
    │   ├── mod.rs
    │   ├── mock/mod.rs     # MockProvider for testing
    │   └── liteapi/mod.rs  # Placeholder for future migration
    ├── composite.rs        # CompositeHotelProvider with fallback
    └── registry.rs         # ProviderRegistry builder
```
