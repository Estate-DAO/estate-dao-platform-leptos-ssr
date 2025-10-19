# Leptos 0.6 to 0.8 Migration Plan

## Overview
This document outlines the complete migration strategy for upgrading the Estate DAO Platform from Leptos 0.6 to Leptos 0.8. This is a major version upgrade that requires careful attention to breaking changes.

## Breaking Changes Summary

### Leptos 0.7 Changes (0.6 → 0.7)
1. **Mount API Changes**
   - `mount_to_body(App)` → `mount_to_body(|| view! { <App /> })`
   - Mount functions now take closures that return views

2. **Router Changes**
   - `<Router>` and `<Routes>` syntax simplified
   - `<Route>` component now uses `view` prop instead of bare component
   - Nested routes structure changed
   - `fallback` prop behavior updated

3. **Component Signatures**
   - More use of `impl IntoView` return types
   - Props now use `#[component]` macro with better inference
   - Signal types more strictly typed

4. **Server Functions**
   - `#[server]` macro syntax updates
   - Context extraction patterns changed
   - Error handling improvements

5. **Actions and Resources**
   - `create_action` and `create_resource` API refinements
   - Better type inference
   - Improved error handling

6. **Context API**
   - `provide_context` and `use_context` more strictly typed
   - Optional context handling improved

### Leptos 0.8 Changes (0.7 → 0.8)
1. **Reactive System Refinements**
   - Signal cloning patterns improved
   - Memo and derived signal optimizations
   - Effect system updates

2. **Router Enhancements**
   - Query parameter handling improved
   - Navigation API updates
   - Route params extraction refined

3. **Meta Tags**
   - `leptos_meta` API updates
   - Title and meta tag handling improved

4. **SSR Improvements**
   - Hydration improvements
   - Server context handling refined

## Migration Steps

### Phase 1: Dependency Updates
- [ ] Update `Cargo.toml` workspace dependencies
- [ ] Update `ssr/Cargo.toml` leptos dependencies to 0.8
- [ ] Update leptos_axum to 0.8
- [ ] Update leptos_meta to 0.8
- [ ] Update leptos_router to 0.8
- [ ] Update leptos-use to compatible version (0.14+)
- [ ] Update leptos_query to compatible version (0.6+)
- [ ] Update leptos_query_devtools if needed

### Phase 2: Core Library Changes
- [ ] Fix `hydrate()` function in `lib.rs`
  - Change `leptos::mount_to_body(App)` to `leptos::mount::mount_to_body(|| view! { <App /> })`
- [ ] Update component imports if needed
- [ ] Fix any `IntoView` trait implementations

### Phase 3: App.rs Router Migration
- [ ] Update `<Router>` component syntax
- [ ] Update `<Routes>` and `<Route>` components
- [ ] Fix fallback handling
- [ ] Update route view props
- [ ] Ensure all routes use closure syntax if required

### Phase 4: Component Updates
- [ ] Review all `#[component]` macros
- [ ] Update component prop types
- [ ] Fix `impl IntoView` returns
- [ ] Update signal usage patterns
- [ ] Fix any `view!` macro issues

### Phase 5: Server Functions
- [ ] Review all `#[server]` function signatures
- [ ] Update context extraction in server functions
- [ ] Fix error handling patterns
- [ ] Update response types

### Phase 6: State Management
- [ ] Update `provide_context` calls
- [ ] Update `use_context` calls
- [ ] Fix signal creation patterns
- [ ] Update derived signals and memos
- [ ] Review effect usage

### Phase 7: Actions and Resources
- [ ] Update `create_action` calls
- [ ] Update `create_resource` calls
- [ ] Fix action submission patterns
- [ ] Update resource refetch patterns

### Phase 8: Meta Tags
- [ ] Update `leptos_meta` imports
- [ ] Fix `<Title>`, `<Meta>`, `<Link>` components
- [ ] Update `provide_meta_context` if needed

### Phase 9: Main.rs and SSR Setup
- [ ] Update `LeptosRoutes` usage
- [ ] Fix `generate_route_list` calls
- [ ] Update server function handlers
- [ ] Fix SSR context setup

### Phase 10: Testing and Validation
- [ ] Run `bash scripts/local_check.sh`
- [ ] Fix any type errors
- [ ] Test hydration behavior
- [ ] Test routing
- [ ] Test server functions
- [ ] Test SSR rendering
- [ ] Run full test suite

## Key API Changes Reference

### Mount API (lib.rs)
```rust
// OLD (0.6)
leptos::mount_to_body(App);

// NEW (0.8)
leptos::mount::mount_to_body(|| view! { <App /> });
```

### Router Syntax (app.rs)
```rust
// OLD (0.6)
<Router fallback=|| view! { <NotFound /> }.into_view()>
    <Routes>
        <Route path="/path" view=MyComponent />
    </Routes>
</Router>

// NEW (0.8)
<Router>
    <Routes fallback=|| view! { <NotFound /> }>
        <Route path="/path" view=MyComponent />
    </Routes>
</Router>
```

### Component Signatures
```rust
// OLD (0.6)
#[component]
pub fn MyComponent(cx: Scope, prop: String) -> impl IntoView { }

// NEW (0.8)
#[component]
pub fn MyComponent(prop: String) -> impl IntoView { }
```

### Server Functions
```rust
// OLD (0.6)
#[server(MyServerFn, "/api")]

// NEW (0.8)
#[server]
pub async fn my_server_fn() -> Result<T, ServerFnError> { }
```

### Context Usage
```rust
// OLD (0.6)
let value = use_context::<MyType>(cx).expect("context missing");

// NEW (0.8)
let value = expect_context::<MyType>();
// or
let value = use_context::<MyType>().expect("context missing");
```

## Dependencies Version Matrix

| Package | Old Version | New Version |
|---------|-------------|-------------|
| leptos | 0.6 | 0.8 |
| leptos_axum | 0.6 | 0.8 |
| leptos_meta | 0.6 | 0.8 |
| leptos_router | 0.6 | 0.8 |
| leptos-use | 0.13.11 | 0.14+ |
| leptos_query | 0.5.3 | 0.6+ |
| leptos_query_devtools | 0.1.3 | 0.2+ |

## Risk Assessment

### High Risk Areas
- Router configuration (many routes to update)
- Server functions (many custom routes)
- Component prop signatures (many components)
- SSR/Hydration boundary

### Medium Risk Areas
- Context providers (many contexts used)
- Actions and Resources
- Meta tags

### Low Risk Areas
- Utility functions
- Domain logic
- API calls

## Rollback Plan
1. Keep backup of current working `Cargo.toml`
2. Use git branches for migration
3. Test thoroughly before merging
4. Have previous Leptos 0.6 dependencies documented

## Post-Migration Checklist
- [ ] All type checks pass
- [ ] All tests pass
- [ ] SSR works correctly
- [ ] Hydration works without errors
- [ ] Routing works on all pages
- [ ] Server functions work
- [ ] Actions submit correctly
- [ ] Resources load correctly
- [ ] Meta tags render correctly
- [ ] Mobile compatibility maintained
- [ ] Performance benchmarks stable
- [ ] Update documentation

## Notes
- Leptos 0.8 removes the `Scope` parameter from components entirely
- The reactive system is now more implicit and uses context internally
- Router is more flexible but requires syntax updates
- Server functions have better ergonomics
- Type inference is significantly improved

## Resources
- [Leptos 0.7 Migration Guide](https://github.com/leptos-rs/leptos/releases/tag/v0.7.0)
- [Leptos 0.8 Release Notes](https://github.com/leptos-rs/leptos/releases/tag/v0.8.0)
- [Leptos Book - Latest](https://leptos-rs.github.io/leptos/)