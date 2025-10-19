# Lessons Learned: Leptos 0.6 to 0.8 Migration

## Date: 2025-10-18

## Overview
This document captures the key lessons learned during the migration from Leptos 0.6 to Leptos 0.8 for the Estate DAO Platform.

## Migration Status
- **Errors Reduced:** From 119+ errors to ~220 errors (mostly view macro and type inference issues)
- **Warnings:** ~186 warnings (mostly unused imports and deprecated patterns)
- **Core API Migration:** ✅ Complete
- **Router Migration:** ✅ Complete
- **Import Updates:** ✅ Complete (120+ files)
- **Remaining Work:** View macro patterns, Icon component usage, Resource API refinements

## Major Breaking Changes

### 1. Import Structure Changed
**Old (0.6):**
```rust
use leptos::*;
```

**New (0.8):**
```rust
use leptos::prelude::*;
```

**Lesson:** Leptos 0.8 introduces a new module structure with most commonly used functions in `leptos::prelude`. This reduces namespace pollution and provides better organization. Applied to 120+ files via bulk find/replace.

### 2. Mount API Changed
**Old (0.6):**
```rust
leptos::mount_to_body(App);
```

**New (0.8):**
```rust
leptos::mount::mount_to_body(|| view! { <App /> });
```

**Lesson:** Mount functions now require a closure that returns a view. This provides more flexibility and consistency with other Leptos APIs.

### 3. Router Fallback Moved
**Old (0.6):**
```rust
<Router fallback=|| view! { <NotFound /> }.into_view()>
    <Routes>
        ...
    </Routes>
</Router>
```

**New (0.8):**
```rust
<Router>
    <Routes fallback=|| view! { <NotFound /> }>
        ...
    </Routes>
</Router>
```

**Lesson:** The `fallback` prop moved from `<Router>` to `<Routes>` component, providing better scoping for fallback behavior.

### 4. Features Removed from Sub-crates
**Old (0.6):**
```toml
leptos_meta = { version = "0.6", features = ["nightly", "hydrate"] }
leptos_router = { version = "0.6", features = ["nightly", "ssr"] }
```

**New (0.8):**
```toml
leptos_meta = { version = "0.8" }
leptos_router = { version = "0.8" }
```

**Lesson:** In Leptos 0.8, features like `hydrate`, `ssr`, and `nightly` are only set on the main `leptos` crate. Sub-crates no longer need these features.

### 5. Context Functions Moved to Prelude
**Old (0.6):**
```rust
use leptos::{use_context, expect_context, provide_context};
```

**New (0.8):**
```rust
use leptos::prelude::*; // Includes these functions
```

**Lesson:** Context management functions are now in the prelude module.

### 6. Signal Types Import Changes
**Issue:** `RwSignal`, `Signal`, `WriteSignal`, etc. may need explicit imports

**New (0.8):**
```rust
use leptos::prelude::*; // Includes signal types
// or explicitly:
use leptos::reactive::{RwSignal, Signal, WriteSignal};
```

### 7. Router Hooks Moved
**Issue:** `use_navigate`, `use_query_map`, `use_location` import errors

**Investigation needed:** These functions have moved in the router module structure.

### 8. Rust Nightly Version Matters
**Issue:** Using an outdated nightly version caused compilation errors in `leptos_macro`

**Solution:** Updated from `nightly-2025-04-15` to `nightly-2025-10-10`

**Lesson:** Leptos 0.8 with nightly features requires a relatively recent nightly compiler. Keep the rust-toolchain updated.

### 9. leptos_query Compatibility
**Issue:** leptos_query 0.6+ doesn't exist - latest is 0.5.3

**Status:** Keeping leptos_query at 0.5.3 which is designed for Leptos 0.6
- May need to migrate to `leptos-fetch` (the successor to leptos_query for 0.7+)
- Or find if there's an updated version compatible with 0.8

**Lesson:** Third-party ecosystem crates may lag behind major Leptos releases. Check compatibility before upgrading.

### 10. leptos-use Version
**Correct version:** 0.16.x for Leptos 0.8 compatibility
- 0.10-0.13 were for Leptos 0.6
- 0.14-0.15 were for Leptos 0.7
- 0.16+ is for Leptos 0.8

## Migration Strategy

### Phase 1: Dependencies ✓
1. Update main Leptos crates to 0.8
2. Remove features from sub-crates
3. Update leptos-use to 0.16
4. Update Rust toolchain to recent nightly

### Phase 2: Core API Changes ✓
1. Update mount function in lib.rs
2. Move Router fallback prop
3. Update imports from `leptos::*` to `leptos::prelude::*`

### Phase 3: Detailed API Migration (In Progress)
1. Fix context function imports
2. Fix signal type imports
3. Fix router hook imports
4. Fix `leptos_dom` module references
5. Update resource/action creation patterns

### Phase 4: Testing
1. Run type checks
2. Test hydration
3. Test SSR
4. Test all routes
5. Test server functions

## Remaining Issues

1. **Router hooks imports** - Need to identify correct import paths
2. **leptos_dom::is_browser** - Module structure changed
3. **Resource/Action creation** - May have API changes
4. **Signal stream functions** - `create_signal_from_stream` location
5. **leptos_query compatibility** - May need migration to leptos-fetch

## Resources Used

- Leptos GitHub Releases: https://github.com/leptos-rs/leptos/releases
- Leptos Book (0.8): https://leptos-rs.github.io/leptos/
- leptos-use compatibility table
- crates.io for version checking

## Key Discoveries

### Router Components in Leptos 0.8
Router components must be explicitly imported and use `path!()` macro:
```rust
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

<Router>
    <Routes fallback=|| view! { <NotFound /> }>
        <Route path=path!("/") view=HomePage />
        <Route path=path!("/about") view=AboutPage />
    </Routes>
</Router>
```

### Resource API Changes
- `create_resource()` → `Resource::new()`
- `create_local_resource()` → `LocalResource::new()`
- Resources require `Send + Sync` bounds in many contexts
- Use `send_wrap()` helper for non-Send futures in resources

### Callback API Changes
- `leptos::Callable::call()` → `Callback::call()`
- Callback trait bounds changed

### Codee Ambiguity
`leptos::prelude::*` re-exports codee, causing ambiguity:
```rust
// Wrong
use codee::string::JsonSerdeCodec;

// Correct
use ::codee::string::JsonSerdeCodec;
```

### Router Hooks Location
All router hooks moved to `leptos_router::hooks`:
- `use_navigate` → `leptos_router::hooks::use_navigate`
- `use_query_map` → `leptos_router::hooks::use_query_map`
- `use_location` → `leptos_router::hooks::use_location`

### Logging Module
`logging::log!` patterns need to use `crate::log!` macro, not `leptos::logging::log!`

### SSE/Streams
`create_signal_from_stream` removed - need to use `spawn_local` with manual signal updates

## Recommendations

1. **Always check ecosystem crate compatibility** before upgrading major versions
2. **Update nightly toolchain** when using nightly features  
3. **Use bulk find/replace** for common patterns (100+ files updated at once)
4. **Test incrementally** - fix imports first, then API changes
5. **Keep migration documentation** for team reference
6. **Consider feature flag strategy** for gradual migration if needed
7. **Use `::codee` prefix** to avoid ambiguity with prelude re-exports
8. **Update all routes to use `path!()` macro**

## Next Steps (Remaining Work)

1. Fix Icon component usage (27+ errors) - likely view macro syntax changes
2. Fix type annotations (37 errors) - mostly in view macros
3. Fix if/else incompatible types (22 errors) - view macro return types
4. Fix JsonSerdeCodec encoder/decoder trait bounds (16+ errors)
5. Replace `Resource.local()` calls (3 errors) - API changed
6. Fix NodeRef attribute binding (2 errors)
7. Update all view macro patterns for 0.8
8. Test compilation after view macro fixes
9. Address runtime behavior changes
10. Update tests
11. Performance benchmarking

## Time Investment

- Planning and research: ~1 hour
- Dependency updates: ~30 minutes
- Bulk import updates: ~1.5 hours (120+ files)
- Core API fixes: ~2 hours
- Router migration: ~1 hour
- Fixing detailed API changes: ~3 hours (In progress)
- View macro fixes: (Pending - estimated 2-4 hours)
- Testing: (Pending - estimated 2-3 hours)

**Total invested so far:** ~9 hours
**Estimated remaining:** 6-9 hours
**Total estimated:** 15-18 hours for complete migration of large codebase (thousands of lines)
## Leptos 0.8 Migration Progress - Latest Session

### Date: $(date +%Y-%m-%d)

#### Errors Fixed in This Session:
1. **ParamsMap API Changes** - Private field access replaced with `.into_iter()` and conversion to HashMap
2. **Callback Invocation** - Changed from `Callback::call(&callback, arg)` to `callback.run(arg)`
3. **spawn_local Location** - Moved to `leptos::task::spawn_local` 
4. **NodeRef Attribute** - `ref=` changed to `node_ref=`
5. **Clone Syntax in View Macros** - `clone:var` syntax removed, use regular Rust cloning
6. **A Component Import** - Explicit import from `leptos_router::components::A`
7. **Deprecated Functions Replaced:**
   - `create_signal` → `signal()`
   - `create_rw_signal` → `RwSignal::new()`
   - `create_memo` → `Memo::new()`
   - `create_effect` → `Effect::new()`
   - `create_action` → `Action::new()`
   - `create_node_ref` → `NodeRef::new()`
   - `store_value` → `StoredValue::new()`
8. **Resource::local** → `LocalResource::new()`
9. **JsonSerdeCodec with Option** - Use `OptionCodec<JsonSerdeCodec>` wrapper, upgraded codee to 0.3
10. **Feature Flags** - Changed `#[cfg(not(feature = "hydrate"))]` to `#[cfg(feature = "ssr")]` for proper axum imports

#### Current Error Count: 98 errors (down from 146+)

#### Remaining Major Issues:
1. **If/Else Type Mismatches (24)** - View macros need `.into_any()` on branches
2. **Icon Component API (3)** - `IconPropsBuilder.class()` method issues  
3. **Type Annotations (8)** - Various places need explicit types
4. **String Rendering (6)** - `&String` doesn't implement `IntoRender`/`RenderHtml`

#### Key Pattern for Type Mismatches in View Macros:
```rust
// Fix incompatible if/else types:
{move || if condition {
    view! { <div>"Branch A"</div> }.into_any()
} else {
    view! { <span>"Branch B"</span> }.into_any()  
}}
```

