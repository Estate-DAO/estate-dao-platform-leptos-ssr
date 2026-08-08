# Amadeus Provider Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an Amadeus hotel provider to `hotel-providers`, wire it into the SSR provider registry, and leave a documented phase-2 path for frontend provider propagation.

**Architecture:** The work starts in `hotel-providers` by introducing stable provider keys, direct registry lookup by provider key, and an `AmadeusDriver` that implements search, static details, rates, synthetic block-room revalidation, and booking. SSR then initializes and exposes Amadeus as a configured provider without yet changing the frontend URL/state model.

**Tech Stack:** Rust, reqwest, serde, async-trait, Leptos SSR, Axum, existing `hotel-providers` and `hotel-types` workspace crates

---

### Task 1: Add stable provider keys and registry lookup

**Files:**
- Modify: `hotel-types/src/ports/mod.rs`
- Modify: `hotel-types/src/ports/hotel_port.rs`
- Modify: `hotel-providers/src/registry.rs`
- Modify: `hotel-providers/src/composite.rs`
- Modify: `hotel-providers/src/liteapi/driver.rs`
- Modify: `hotel-providers/src/booking/driver.rs`
- Modify: `hotel-providers/src/adapters/mock/mod.rs`
- Test: `hotel-providers/src/registry.rs`

- [ ] **Step 1: Write the failing registry tests**

```rust
#[test]
fn registry_can_resolve_hotel_provider_by_stable_key() {
    let registry = ProviderRegistry::builder()
        .with_hotel_provider(MockHotelProvider::new("liteapi"))
        .with_hotel_provider(MockHotelProvider::new("amadeus"))
        .with_place_provider(MockPlaceProvider::default())
        .build();

    assert_eq!(registry.hotel_provider_by_key("amadeus").unwrap().key(), "amadeus");
}
```

- [ ] **Step 2: Run the registry test to verify it fails**

Run: `cargo test -p hotel-providers registry_can_resolve_hotel_provider_by_stable_key --features "liteapi booking mock"`

Expected: FAIL because `hotel_provider_by_key` and `key()` do not exist yet.

- [ ] **Step 3: Add stable provider-key support**

```rust
#[async_trait]
pub trait HotelProviderPort: Send + Sync {
    fn key(&self) -> &'static str;
    fn name(&self) -> &'static str;
}
```

- [ ] **Step 4: Teach the registry to keep named providers**

```rust
pub struct ProviderRegistry {
    hotel_provider: Arc<dyn HotelProviderPort>,
    hotel_provider_by_key: HashMap<&'static str, Arc<dyn HotelProviderPort>>,
    place_provider: Arc<dyn PlaceProviderPort>,
}
```

- [ ] **Step 5: Re-run the registry test**

Run: `cargo test -p hotel-providers registry_can_resolve_hotel_provider_by_stable_key --features "liteapi booking mock"`

Expected: PASS

- [ ] **Step 6: Run the provider crate test suite**

Run: `cargo test -p hotel-providers --features "liteapi booking mock"`

Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add hotel-types/src/ports/mod.rs hotel-types/src/ports/hotel_port.rs hotel-providers/src/registry.rs hotel-providers/src/composite.rs hotel-providers/src/liteapi/driver.rs hotel-providers/src/booking/driver.rs hotel-providers/src/adapters/mock/mod.rs
git commit -m "feat: add stable hotel provider keys"
```

### Task 2: Scaffold the Amadeus provider module

**Files:**
- Modify: `hotel-providers/Cargo.toml`
- Modify: `hotel-providers/src/lib.rs`
- Add: `hotel-providers/src/amadeus/mod.rs`
- Add: `hotel-providers/src/amadeus/client.rs`
- Add: `hotel-providers/src/amadeus/driver.rs`
- Add: `hotel-providers/src/amadeus/mapper.rs`
- Add: `hotel-providers/src/amadeus/models/mod.rs`
- Add: `hotel-providers/src/amadeus/models/auth.rs`
- Add: `hotel-providers/src/amadeus/models/search.rs`
- Add: `hotel-providers/src/amadeus/models/hotel_details.rs`
- Add: `hotel-providers/src/amadeus/models/booking.rs`
- Test: `hotel-providers/src/amadeus/driver.rs`

- [ ] **Step 1: Write a failing compile test for the module exports**

```rust
#[test]
fn amadeus_driver_reports_expected_identity() {
    let driver = AmadeusDriver::new_mock();
    assert_eq!(driver.key(), "amadeus");
    assert_eq!(driver.name(), "Amadeus");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p hotel-providers amadeus_driver_reports_expected_identity --features "amadeus mock"`

Expected: FAIL because the module and feature do not exist yet.

- [ ] **Step 3: Add the new feature and module exports**

```toml
[features]
amadeus = ["dep:reqwest"]
```

- [ ] **Step 4: Add the module skeleton**

```rust
pub mod client;
pub mod driver;
pub mod mapper;
pub mod models;

pub use client::AmadeusClient;
pub use driver::AmadeusDriver;
```

- [ ] **Step 5: Re-run the identity test**

Run: `cargo test -p hotel-providers amadeus_driver_reports_expected_identity --features "amadeus mock"`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add hotel-providers/Cargo.toml hotel-providers/src/lib.rs hotel-providers/src/amadeus
git commit -m "feat: scaffold amadeus provider module"
```

### Task 3: Implement OAuth token handling and HTTP client plumbing

**Files:**
- Modify: `hotel-providers/src/amadeus/client.rs`
- Modify: `hotel-providers/src/amadeus/models/auth.rs`
- Test: `hotel-providers/src/amadeus/client.rs`

- [ ] **Step 1: Write the failing token-cache tests**

```rust
#[tokio::test]
async fn client_reuses_unexpired_access_token() {
    let client = AmadeusClient::new_for_test(/* ... */);
    let first = client.access_token().await.unwrap();
    let second = client.access_token().await.unwrap();
    assert_eq!(first, second);
}
```

- [ ] **Step 2: Run the token-cache test to verify it fails**

Run: `cargo test -p hotel-providers client_reuses_unexpired_access_token --features "amadeus"`

Expected: FAIL because token caching is not implemented.

- [ ] **Step 3: Implement OAuth request, token cache, and bearer injection**

```rust
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}
```

- [ ] **Step 4: Re-run the token-cache tests**

Run: `cargo test -p hotel-providers client_reuses_unexpired_access_token --features "amadeus"`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add hotel-providers/src/amadeus/client.rs hotel-providers/src/amadeus/models/auth.rs
git commit -m "feat: add amadeus oauth client"
```

### Task 4: Map Amadeus hotel search into domain search results

**Files:**
- Modify: `hotel-providers/src/amadeus/models/search.rs`
- Modify: `hotel-providers/src/amadeus/mapper.rs`
- Modify: `hotel-providers/src/amadeus/driver.rs`
- Test: `hotel-providers/src/amadeus/mapper.rs`

- [ ] **Step 1: Write the failing mapper tests for geocode search**

```rust
#[test]
fn maps_amadeus_geocode_and_offers_to_domain_hotel_list() {
    let result = AmadeusMapper::map_search_to_domain(hotels_resp, offers_resp, None);
    assert_eq!(result.provider.as_deref(), Some("Amadeus"));
    assert!(!result.hotel_results.is_empty());
}
```

- [ ] **Step 2: Run the mapper test to verify it fails**

Run: `cargo test -p hotel-providers maps_amadeus_geocode_and_offers_to_domain_hotel_list --features "amadeus"`

Expected: FAIL because the search mapping is not implemented.

- [ ] **Step 3: Implement geocode-first hotel search**

```rust
if let (Some(latitude), Some(longitude)) = (criteria.latitude, criteria.longitude) {
    // by-geocode -> hotel IDs -> hotel offers
} else {
    return Err(ProviderError::new("Amadeus", ProviderErrorKind::InvalidRequest, ProviderSteps::HotelSearch, "Amadeus search requires coordinates"));
}
```

- [ ] **Step 4: Re-run the mapper tests**

Run: `cargo test -p hotel-providers maps_amadeus_geocode_and_offers_to_domain_hotel_list --features "amadeus"`

Expected: PASS

- [ ] **Step 5: Run the provider crate test suite**

Run: `cargo test -p hotel-providers --features "amadeus liteapi booking mock"`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add hotel-providers/src/amadeus/models/search.rs hotel-providers/src/amadeus/mapper.rs hotel-providers/src/amadeus/driver.rs
git commit -m "feat: implement amadeus hotel search"
```

### Task 5: Implement Amadeus static details and rates

**Files:**
- Modify: `hotel-providers/src/amadeus/models/hotel_details.rs`
- Modify: `hotel-providers/src/amadeus/models/search.rs`
- Modify: `hotel-providers/src/amadeus/mapper.rs`
- Modify: `hotel-providers/src/amadeus/driver.rs`
- Test: `hotel-providers/src/amadeus/mapper.rs`

- [ ] **Step 1: Write the failing details and rates tests**

```rust
#[test]
fn maps_amadeus_hotel_details_to_domain_static_details() {}

#[test]
fn maps_amadeus_offers_to_grouped_room_rates() {}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p hotel-providers maps_amadeus_hotel_details_to_domain_static_details --features "amadeus"`

Expected: FAIL because details and grouped rates are not mapped.

- [ ] **Step 3: Implement details and rates mapping**

```rust
let static_details = AmadeusMapper::map_hotel_by_id_to_domain(detail);
let grouped_rates = AmadeusMapper::map_offers_to_grouped_rates(offers);
```

- [ ] **Step 4: Re-run the tests**

Run: `cargo test -p hotel-providers maps_amadeus_hotel_details_to_domain_static_details --features "amadeus"`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add hotel-providers/src/amadeus/models/hotel_details.rs hotel-providers/src/amadeus/mapper.rs hotel-providers/src/amadeus/driver.rs
git commit -m "feat: implement amadeus hotel details and rates"
```

### Task 6: Implement synthetic block-room revalidation and booking

**Files:**
- Modify: `hotel-providers/src/amadeus/models/booking.rs`
- Modify: `hotel-providers/src/amadeus/mapper.rs`
- Modify: `hotel-providers/src/amadeus/driver.rs`
- Test: `hotel-providers/src/amadeus/driver.rs`

- [ ] **Step 1: Write the failing block-room and booking tests**

```rust
#[tokio::test]
async fn block_room_revalidates_offer_and_returns_provider_snapshot() {}

#[tokio::test]
async fn book_room_uses_snapshot_to_create_amadeus_booking() {}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p hotel-providers block_room_revalidates_offer_and_returns_provider_snapshot --features "amadeus"`

Expected: FAIL because Amadeus block/book are not implemented.

- [ ] **Step 3: Implement synthetic block-room revalidation**

```rust
let snapshot = serde_json::to_string(&offer)?;
Ok(DomainBlockRoomResponse {
    block_id: uuid_like_id(),
    provider_data: Some(snapshot),
    provider: Some("Amadeus".to_string()),
    // ...
})
```

- [ ] **Step 4: Implement booking v1 request mapping**

```rust
let req = AmadeusMapper::map_domain_book_to_amadeus_booking_v1(&request, &snapshot)?;
let resp = self.client.book_hotel_v1(&req).await?;
```

- [ ] **Step 5: Return explicit unsupported error for booking-details lookup**

```rust
Err(ProviderError::new(
    "Amadeus",
    ProviderErrorKind::Other,
    ProviderSteps::GetBookingDetails,
    "Amadeus booking lookup is not implemented",
))
```

- [ ] **Step 6: Re-run the tests**

Run: `cargo test -p hotel-providers --features "amadeus liteapi booking mock"`

Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add hotel-providers/src/amadeus/models/booking.rs hotel-providers/src/amadeus/mapper.rs hotel-providers/src/amadeus/driver.rs
git commit -m "feat: add amadeus block and booking flow"
```

### Task 7: Wire Amadeus into SSR initialization and admin config

**Files:**
- Modify: `ssr/Cargo.toml`
- Modify: `ssr/src/init.rs`
- Modify: `ssr/src/server_functions_impl_custom_routes/admin_provider.rs`
- Test: `ssr/src/init.rs`

- [ ] **Step 1: Write the failing SSR wiring tests**

```rust
#[test]
fn admin_provider_config_lists_amadeus() {}

#[test]
fn init_can_build_registry_with_amadeus_primary() {}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p estate-fe admin_provider_config_lists_amadeus --features ssr`

Expected: FAIL because SSR does not know about Amadeus yet.

- [ ] **Step 3: Add SSR wiring**

```rust
enum PrimaryHotelProvider {
    LiteApi,
    Booking,
    Amadeus,
}
```

- [ ] **Step 4: Register `AmadeusDriver` in the provider registry builder**

```rust
PrimaryHotelProvider::Amadeus => builder
    .with_hotel_provider(amadeus_driver.clone())
    .with_hotel_provider(liteapi_driver.clone())
    .with_hotel_provider(booking_driver.clone())
```

- [ ] **Step 5: Re-run the SSR wiring tests**

Run: `cargo test -p estate-fe admin_provider_config_lists_amadeus --features ssr`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add ssr/Cargo.toml ssr/src/init.rs ssr/src/server_functions_impl_custom_routes/admin_provider.rs
git commit -m "feat: wire amadeus into ssr provider config"
```

### Task 8: Add phase-2 provider propagation for hotel flows

**Files:**
- Modify: `ssr/src/page/hotel_list_params.rs`
- Modify: `ssr/src/page/hotel_details_params.rs`
- Modify: `ssr/src/view_state_layer/view_state.rs`
- Modify: `ssr/src/api/client_side_api.rs`
- Modify: `ssr/src/server_functions_impl_custom_routes/get_hotel_details.rs`
- Modify: `ssr/src/server_functions_impl_custom_routes/get_hotel_rates.rs`
- Modify: `ssr/src/server_functions_impl_custom_routes/block_room.rs`
- Modify: `ssr/src/server_functions_impl_custom_routes/book_room.rs`
- Test: `ssr/src/page/hotel_list_params.rs`
- Test: `ssr/src/page/hotel_details_params.rs`

- [ ] **Step 1: Write the failing URL-param tests**

```rust
#[test]
fn hotel_details_params_round_trip_provider_key() {}

#[test]
fn hotel_list_params_round_trip_provider_key() {}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p estate-fe hotel_details_params_round_trip_provider_key --features ssr`

Expected: FAIL because provider is not part of the params.

- [ ] **Step 3: Add provider to Leptos state and shareable params**

```rust
pub struct HotelDetailsParams {
    pub provider: Option<String>,
}
```

- [ ] **Step 4: Add provider-aware backend selection**

```rust
let provider = match request.provider.as_deref() {
    Some(key) => registry.hotel_provider_by_key(key).unwrap_or_else(|| registry.hotel_provider()),
    None => registry.hotel_provider(),
};
```

- [ ] **Step 5: Re-run the URL and SSR tests**

Run: `cargo test -p estate-fe hotel_details_params_round_trip_provider_key --features ssr`

Expected: PASS

- [ ] **Step 6: Run repo verification**

Run: `bash scripts/local_check.sh`

Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add ssr/src/page/hotel_list_params.rs ssr/src/page/hotel_details_params.rs ssr/src/view_state_layer/view_state.rs ssr/src/api/client_side_api.rs ssr/src/server_functions_impl_custom_routes/get_hotel_details.rs ssr/src/server_functions_impl_custom_routes/get_hotel_rates.rs ssr/src/server_functions_impl_custom_routes/block_room.rs ssr/src/server_functions_impl_custom_routes/book_room.rs
git commit -m "feat: preserve hotel provider across frontend flows"
```
