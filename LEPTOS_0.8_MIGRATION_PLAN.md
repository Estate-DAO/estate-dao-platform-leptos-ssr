# Leptos 0.8 Migration Plan

## Problem
After upgrading from Leptos 0.6 to 0.8, runtime compilation errors occur due to `Resource::new` now requiring `Send +()` - for !Send futures (client-side only, 'static` futures. Code using `g WASM code)

## Files to Fix

### 1. ssr/src/page/wishlist.rs
**loo-net` and WASM bindings (Lines to change:**
- Line 38: `let wishlist_details = Resource::newwhich are `!Send`)(` → `let wishlist_details = LocalResource::new(`
- Line 63 fails to compile.

## Root Cause
In: `let hotel_details_resource = Resource::new(` → `let hotel_details_resource = Local Leptos 0.8:
- `Resource::new()Resource::new(`

**Reason:** Uses` requires futures to be `Send + 'static `reqwest::Client` for` (for SSR compatibility)
- `LocalResource::new()` is `/api/user-wishlist` and `ClientSideApiClient` which for `!Send` futures (client-side only, WASM code uses `gloo-net` internally.

### 2. ssr)
- Our client-side code uses `req/src/component/yral_auth_provider.west::Client` and `gloo-rs
**Lines to change:**
- Line 36: `let profile_details = Resource::new(` → `let profile_details = Localnet` which are not `Send` inResource::new(`
- Line 61: `let wishlist_details = WASM context

## Files Requiring Resource::new(` → `let wishlist_details = LocalResource:: Changes

### High Priority (Causingnew(`

**Reason:** Uses `reqwest::Client` for `/api/user-info` and `/api/user-wishlist`. Build Failures)
1. **

### 3. ssr/src/component/data_table_3.rs
**Lines to checkssr/src/page/wishlist.rs**:**
- Line 296: `let all
   - Line 38: `wishlist_details_user_bookings = Resource::new(`
- Line 302` Resource - uses reqwest Client
   - Line 63: `let filtered_sorted_bookings = Resource::new(: `hotel_details_resource` Resource - uses ClientSideApiClient (gloo-net)
   - Fix: Replace `Resource::new`

**Action:** Check if these use server functions` with `LocalResource::new`

2. **ssr/src/component/yral_auth_provider.rs**
   - Line 36: `profile_details` Resource - uses reqwest Client
   - Line or client-side fetching. If server functions with #[server] macro, they 61: `wishlist_details` Resource - uses reqwest Client
   - Fix: Replace `Resource::new should be fine. If they` with `LocalResource::new`

3. **ssr/src/ use client-side code, change to `LocalResource::new()`.

###component/data_table 4. ssr/src/page_3.rs**
   - Line 295: `all_user_book/hotel_details_v1.rs
**Lines to check:**
- Line 227: `let hotel_details_ings` Resource
   - Line 302: `filtereresource = Resource::new(`

**Action:**d_sorted_bookings` Resource
   - Fix Likely needs to be `LocalResource::new()` if it uses client-side API calls.

### 5. ssr/src/page/hotel_list.: Need to check if server functions are used, mayrs
**Lines to check:**
- Line 114: `let hotel_search_resource = Resource::new( need `LocalResource::new`

4. **ssr/src/page`

**Action:** Check if uses server functions or client-side API./hotel_details_v1.rs**
   - Line Change to `LocalResource` if client-side.

### 6. ssr/src/page/my 224: `hotel_details_resource` Resource_bookings.rs
**Lines to check:**
- Line 123: `let bookings_resource = Resource::new(`

**Action:** Check
   - Fix: Check if uses client-side API, replace with `LocalResource::new`

5. **ssr/src/ `load_my_bookings()page/hotel_list.rs**
   - Line 112` - if it's a: `hotel_search_resource` Resource
   - Fix client-side function, use `LocalResource::new()`.: Check if uses client-side API, replace with `LocalResource::new`

6. **ssr/src/page/my_bookings.rs**
   - Line 123

## Import Changes Needed

For: `bookings_resource` Resource
   - all files that change to `LocalResource`, Fix: Check if uses server functions, may be fine as- ensure imports include:
```rust
use lepis

### Other Deprecation Warnings (tos::prelude::*; // ShoulNon-Breaking)
- Replace `create_noded already include LocalResource
```

If `LocalResource` is not in scope_ref()` with `NodeRef::new()`
- Replace, add explicitly:
```rust
use leptos::prelude::{LocalResource, ...};
```

## Testing Strategy `MaybeSignal<T>` with `Signal<T>`

## Migration Strategy

### Step

After changes:
1. Run `bash 1: Fix Resource Issues
For scripts/local_check.sh` to verify no type errors
2. Run `bash scripts each Resource that uses client-side APIs (req/local_run.sh` to test runtimewest, gloo-net):
```rust
// OLD (0.6)
let resource = Resource
3. Test each affected page:
   - Wishlist page::new(
    move || signal (authenticated and unauthenticated)(),
    move |data| async
   - Hotel details page
   - Hotel list/ move { /* client-side APIsearch
   - My bookings page
   - User authentication flow

## Additional Notes

- call */ }
);

// NEW (0.8)
let resource = LocalResource::new(
    move || signal(),
    move |data| async move { /* client-side API call */ }
);
```

### Step 2: Import `LocalResource` works exactly like `Resource` but for !Send futures
- No other API changes needed
- Server Changes
Add to imports:
```rust
use leptos::prelude::*; // Already includes functions (#[server] annotated) can still Resource
// LocalResource is use `Resource::new()` as they also in leptos::prelude
```

### are Send
- Only client-side browser Step 3: Testing
After each fix:
1. Run code needs `LocalResource`

## Deprecate `bash scripts/local_check.sh` to verify typed API Fixes (Lower errors
2. Run `bash scripts/ Priority)

Also found theselocal_run.sh` to test runtime deprecation warnings to fix later:
- Replace `create_node
3. Test affected pages in browser_ref()` with `Node

## Implementation Order
1. Fix wishlist.rs (2Ref::new()`
- Replace `MaybeSignal<T>` with `Signal resources)
2. Fix yral_auth_provider.rs (2 resources)
3. Fix<T> data_table_3.rs (`
need to verify server function usage)
4. Fix hotel_details_v1.rs
5. Fix hotel_list.rs
6. Fix my_bookings.rs
7. Address deprecation warnings (optional, non-breaking)

## Notes
- Server functions (those using `#[server]`) can still use `Resource::new` because server-side code is `Send`
- Client-side API calls (using gloo-net, WASM bindings) MUST use `LocalResource::new`
- When in doubt, if the future is spawned in browser context, use `LocalResource::new`
