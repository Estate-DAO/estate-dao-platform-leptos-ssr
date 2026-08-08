# Amadeus Hotel Provider Design

## Summary

Add an `AmadeusDriver` to the existing [`hotel-providers`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers) crate so Amadeus can participate in hotel-provider selection alongside LiteAPI and Booking.com. The provider work comes first. Frontend and Leptos URL/state propagation are a follow-up phase after the provider module, registry wiring, and backend selection path are stable.

The key architectural decision is that provider identity must be explicit and stable. A provider-bound hotel flow cannot silently drift to another adapter later because hotel IDs, offer IDs, and booking semantics are adapter-specific.

## Current State

- [`hotel-providers/src/lib.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/lib.rs) exposes the provider abstraction used by SSR.
- [`hotel-types/src/ports/hotel_port.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-types/src/ports/hotel_port.rs) defines the `HotelProviderPort` contract.
- [`hotel-providers/src/registry.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/registry.rs) builds either a single provider or a composite provider with fallback.
- [`ssr/src/init.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/ssr/src/init.rs) initializes LiteAPI and Booking.com and exposes the configured provider registry.
- Search, hotel details, rates, block, and book server routes currently resolve the provider through the configured registry and do not preserve provider identity in the request/URL shape.
- Shared hotel domain types already carry `provider: Option<String>` in several places, but that provenance is not yet used to route subsequent requests.

## Goals

1. Add Amadeus as a first-class provider module inside [`hotel-providers/src`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src).
2. Make provider identity explicit through stable provider keys instead of display-name matching.
3. Allow the backend to resolve either:
   - the configured default/composite provider for providerless entry points
   - a concrete provider by key for provider-bound flows
4. Keep provider work isolated from the frontend until the provider layer is test-covered.
5. Preserve a clear path to phase-2 frontend integration where deep links and hotel flows carry provider identity.

## Non-Goals

- Replacing the current place provider in phase 1.
- Reworking the booking UX in phase 1.
- Migrating all SSR logic away from the current route structure in phase 1.
- Claiming full Amadeus booking parity without explicitly solving the block-room contract gap.

## Product Decisions

### Provider identity must be sticky

If a hotel flow starts on one provider, later requests for details, rates, block, and book must resolve the same provider. This is required because:

- hotel codes are provider-specific
- rate tokens and offer IDs are provider-specific
- booking payloads are provider-specific

Providerless entry points may still use the configured default provider or composite fallback.

### Provider-first sequencing

Implementation is split into two phases:

1. Provider foundation
   - Amadeus module
   - provider key model
   - registry lookup by provider key
   - SSR registry initialization
2. Frontend and request propagation
   - URL params
   - client request DTOs
   - Leptos state binding
   - provider-aware route selection

### Stable provider keys vs display names

The codebase currently mixes admin/config keys such as `liteapi` with display names such as `LiteAPI` and `Booking.com`. That is error-prone for routing. The design introduces:

- stable provider keys for routing and config:
  - `liteapi`
  - `booking`
  - `amadeus`
- display names for logs and UI:
  - `LiteAPI`
  - `Booking.com`
  - `Amadeus`

`HotelProviderPort` should expose a stable key separately from its display name.

## Provider Architecture

Create a new module parallel to the existing provider implementations:

- [`hotel-providers/src/amadeus/mod.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/amadeus/mod.rs)
- [`hotel-providers/src/amadeus/client.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/amadeus/client.rs)
- [`hotel-providers/src/amadeus/driver.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/amadeus/driver.rs)
- [`hotel-providers/src/amadeus/mapper.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/amadeus/mapper.rs)
- [`hotel-providers/src/amadeus/models/mod.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/amadeus/models/mod.rs)
- request/response model files under [`hotel-providers/src/amadeus/models/`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/amadeus/models)

This should follow the same shape as LiteAPI and Booking.com:

- `client.rs` owns HTTP and auth concerns
- `mapper.rs` owns all domain translation
- `driver.rs` owns `HotelProviderPort` behavior
- `models/` owns provider payloads

## Registry Changes

[`hotel-providers/src/registry.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/registry.rs) currently only stores the effective hotel provider. It must also retain named hotel providers so the backend can resolve a concrete provider by key.

The registry should expose:

- `hotel_provider()` for the current configured/composite provider
- `hotel_provider_by_key(&str)` for provider-bound flows

The registry builder should preserve insertion order for composite fallback while also recording providers in a lookup map keyed by the stable provider key.

## Amadeus API Mapping

The supplied Postman collection shows the hotel surface available to us:

- OAuth token: `POST /v1/security/oauth2/token`
- hotel list by city: `GET /v1/reference-data/locations/hotels/by-city`
- hotel list by geocode: `GET /v1/reference-data/locations/hotels/by-geocode`
- hotel list by hotel IDs: `GET /v1/reference-data/locations/hotels/by-hotels`
- hotel offers search: `GET /v3/shopping/hotel-offers`
- single offer lookup: `GET /v3/shopping/hotel-offers/{hotelOfferId}`
- booking v1: `POST /v1/booking/hotel-bookings`
- booking v2: `POST /v2/booking/hotel-orders`
- hotel name autocomplete

### Auth

Amadeus requires OAuth client credentials. The client should:

- request an access token lazily
- cache the token with expiry
- refresh slightly before expiry
- add `Authorization: Bearer <token>` automatically to each request

### Search strategy

`DomainHotelSearchCriteria` currently provides `place_id` and optional `latitude`/`longitude`.

Phase-1 search should prefer coordinates:

1. Use `latitude` and `longitude` with the Amadeus geocode hotel list endpoint when coordinates are available.
2. Use the returned hotel IDs to call hotel offers search.
3. Merge hotel metadata and minimum sellable offer price into `DomainHotelListAfterSearch`.

If only `place_id` is present and no coordinates are available, the provider should return a clear invalid-request error instead of guessing how to translate opaque place IDs into Amadeus city codes.

### Static details

`get_hotel_static_details` should call the hotel list-by-ID endpoint and map hotel metadata into `DomainHotelStaticDetails`.

### Rates

`get_hotel_rates` should use hotel offers search for the requested hotel IDs and map offers into `DomainGroupedRoomRates`.

`result_token` and `offer_id` should preserve enough upstream identity for later revalidation and booking.

### Block room

Amadeus does not expose a true prebook/block endpoint in the provided collection. The current application expects `block_room -> payment -> book_room`.

Phase-1 provider design uses a synthetic revalidation block step:

1. Re-fetch the selected Amadeus offer(s) just before payment.
2. Compare price and basic policy fields against the selected room data.
3. Produce a synthetic `DomainBlockRoomResponse` with:
   - a generated `block_id`
   - `is_price_changed`
   - `is_cancellation_policy_changed`
   - normalized blocked-room data
   - serialized provider snapshot in `provider_data`

Important limitation:

- this is a validation snapshot, not an upstream inventory hold
- the spec intentionally records that limitation instead of pretending Amadeus supports a real block

### Booking

Use Amadeus booking v1 first.

Reasoning:

- the v1 request in the supplied collection is smaller and easier to map from the current domain model
- it aligns more directly with the current room/guest/payment shape used by the SSR booking pipeline

`book_room` should consume the synthetic block snapshot stored in `provider_data`, rebuild the booking payload, and call the Amadeus booking endpoint.

### Booking lookup

The supplied collection does not show an Amadeus booking-details retrieval endpoint equivalent to the current `get_booking_details` contract.

Phase-1 design therefore treats `get_booking_details` as unsupported unless a concrete upstream retrieval path is discovered during implementation. The driver should return a provider-specific error instead of inventing a fake lookup contract.

This means:

- the Amadeus provider module can still be built and wired
- full post-booking parity is not claimed until this gap is closed

## SSR Wiring

Phase 1 backend changes should stay narrow:

- add Amadeus feature wiring to [`hotel-providers/Cargo.toml`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/Cargo.toml)
- register the new module in [`hotel-providers/src/lib.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/hotel-providers/src/lib.rs)
- initialize `AmadeusDriver` in [`ssr/src/init.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/ssr/src/init.rs)
- include `amadeus` in the admin provider config endpoints in [`ssr/src/server_functions_impl_custom_routes/admin_provider.rs`](/Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr/ssr/src/server_functions_impl_custom_routes/admin_provider.rs)
- expose direct provider resolution by key for later frontend-bound integration

Phase 1 does not change the frontend request/URL model yet.

## Phase-2 Frontend Integration

After the provider layer is stable:

- add optional `provider` to hotel-list and hotel-details shareable params
- persist provider key in Leptos state during hotel flows
- include provider key in details/rates/block/book requests
- resolve provider-bound requests through `hotel_provider_by_key`

This phase preserves deep-link stability and prevents provider drift after admin config changes.

## Testing Strategy

### Provider crate

Add unit tests for:

- stable provider key registration
- registry lookup by provider key
- composite fallback still preserving order
- Amadeus search mapping
- Amadeus details mapping
- Amadeus rates grouping
- synthetic block-room revalidation behavior
- unsupported booking-details behavior

### SSR wiring

Add tests for:

- Amadeus driver initialization from env
- admin provider config now listing `amadeus`
- registry selection choosing Amadeus when configured

## Risks

### Booking contract mismatch

The application assumes an upstream block step. Amadeus appears offer/book oriented. Synthetic block-room revalidation reduces mismatch but does not create a real hold.

### Place ID mismatch

Current search criteria uses opaque `place_id` values plus optional coordinates. Amadeus does not naturally consume those IDs. The provider should use geocode search when coordinates are present and fail clearly when they are not.

### Partial provider parity

Without a verified Amadeus booking-details lookup endpoint, full provider parity remains incomplete. The design keeps this explicit so the implementation does not over-promise.

## Recommended Delivery Order

1. Provider keys and registry lookup support
2. Amadeus module scaffold and auth client
3. Search/details/rates mapping and tests
4. Synthetic block-room revalidation
5. Booking request mapping
6. SSR init and admin wiring
7. Frontend and deep-link provider propagation
