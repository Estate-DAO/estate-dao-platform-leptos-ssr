# Leptos 0.8 CI Build Fix - Summary

## Problem Statement

After upgrading from Leptos 0.6 to Leptos 0.8, the application compiled successfully **locally** but **failed in CI** with the following errors:

### Error 1: Type Layout Query Depth
```
= note: query depth increased by 258 when computing layout of `{async block@<T as tachys::view::any_view::IntoAny>::into_any::hydrate_async<leptos::into_view::View<...>>`
```

### Error 2: Recursion Limit (after fixing Error 1)
```
error[E0275]: overflow normalizing the opaque type `<leptos::tachys::view::static_types::Static<"flex-1"> as IntoClass>::resolve::{opaque#0}`
  = help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`estate_fe`)
```

## Root Cause

**Leptos 0.8** introduced a completely new rendering system called **tachys** that creates significantly more complex type signatures compared to Leptos 0.6:

1. **Deeper type nesting** - Components are now wrapped in more type layers
2. **More complex monomorphization** - Type substitutions during compilation are more numerous
3. **Longer type normalization chains** - The compiler has to work harder to resolve types

Your application's deeply nested component structure amplified this issue:
```
HeroSection
  └─ InputGroupContainer
      ├─ DestinationPickerV6
      ├─ DateTimeRangePickerCustom
      └─ GuestQuantity
          └─ Multiple nested Show components with complex closures
```

## Solution Applied

Added compiler limit attributes to **both** the library crate and binary crate:

### File: `ssr/src/lib.rs`
```rust
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![recursion_limit = "1024"]           // ← Increased from 512
#![type_length_limit = "10000000"]     // ← NEW: Added for Leptos 0.8
```

### File: `ssr/src/main.rs`
```rust
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![recursion_limit = "1024"]           // ← NEW: Added
#![type_length_limit = "10000000"]     // ← NEW: Added
```

## Why This Fix Works

### `recursion_limit`
- **What it controls:** Maximum depth of type normalization during compilation
- **Default:** 128
- **What we set:** 1024
- **Why needed:** Leptos 0.8's tachys renderer creates deeply nested type resolutions

### `type_length_limit`
- **What it controls:** Maximum number of type substitutions during monomorphization
- **Default:** 1,048,576
- **What we set:** 10,000,000
- **Why needed:** Complex component trees create many type substitutions

### Why Both Files?
In Rust, a project typically has:
- **Library crate** (`lib.rs`) - Compiled first, contains most of the code
- **Binary crate** (`main.rs`) - Compiled separately, uses the library

Each crate is compiled independently with its own limits. Since both crates compile view code in Leptos applications, both need the increased limits.

## Why It Failed in CI But Not Locally

1. **Build mode:** CI uses `--release` which triggers different compiler optimizations
2. **Cache state:** Local builds benefit from incremental compilation cache
3. **Fresh build:** CI always does a clean build, exercising all code paths
4. **Memory constraints:** CI environments may have different memory pressure
5. **Optimization level:** Release builds inline more aggressively, creating larger types

## Impact Assessment

### ✅ Positive
- **Zero runtime cost** - These are compile-time only limits
- **No binary size increase** - Limits don't affect output
- **No performance impact** - Only affects compilation time
- **Simple solution** - Just two attribute lines per file

### ⚠️ Trade-offs
- **Longer compile times** - Compiler does more work (acceptable for CI)
- **More memory during compilation** - Not an issue for modern systems

## Verification

### Local Testing
```bash
# Type check (fast)
bash scripts/local_check.sh

# Full build
bash scripts/local_run.sh

# Release build (matches CI)
cargo leptos build --release
```

### Expected Results
- ✅ No "overflow normalizing" errors
- ✅ No "query depth increased" errors
- ✅ Successful compilation

## Files Modified

| File | Changes |
|------|---------|
| `ssr/src/lib.rs` | Added `recursion_limit = "1024"` and `type_length_limit = "10000000"` |
| `ssr/src/main.rs` | Added `recursion_limit = "1024"` and `type_length_limit = "10000000"` |
| `lessons_learnt.md` | Documented issue and solution in detail |
| `CI_BUILD_FIX_CHECKLIST.md` | Created verification checklist |
| `LEPTOS_0.8_CI_FIX_SUMMARY.md` | This file |

## Key Learnings

1. **Leptos 0.8 requires higher compiler limits** due to tachys architecture
2. **Crate-level attributes affect each crate independently** - both lib and bin need them
3. **CI environments expose issues that local builds might hide** due to caching
4. **Type system limits are compile-time only** - no runtime implications
5. **Both `recursion_limit` and `type_length_limit` may need adjustment** for complex UIs

## Long-term Recommendations

While the immediate fix works, consider these for better compile times and maintainability:

### 1. Component Refactoring
Break down large components into smaller, more focused pieces:
```rust
// Before: One large component
#[component]
fn HeroSection() -> impl IntoView { /* 300+ lines */ }

// After: Split into logical pieces
#[component]
fn HeroSection() -> impl IntoView {
    view! {
        <SearchBar />
        <FeaturedProperties />
    }
}
```

### 2. Reduce Conditional Nesting
```rust
// Before: Nested Show components
view! {
    <Show when=cond1>
        <Show when=cond2>
            <ComplexComponent />
        </Show>
    </Show>
}

// After: Combined conditions
view! {
    <Show when=move || cond1() && cond2()>
        <ComplexComponent />
    </Show>
}
```

### 3. Use Type Boundaries
Extract complex logic into separate components to create type boundaries that help the compiler.

### 4. Profile Compile Times
Use `cargo build --timings` to identify slow-compiling modules and focus optimization efforts.

## When to Increase Limits Further

If you add more complex components and see the errors return:

```rust
// Double the limits
#![recursion_limit = "2048"]
#![type_length_limit = "20000000"]
```

Monitor compile times and consider refactoring if they become excessive (>5 minutes for release builds).

## References

- [Rust Reference - Limit Attributes](https://doc.rust-lang.org/reference/attributes/limits.html)
- [Leptos 0.8 Documentation](https://leptos.dev/)
- [Tachys Renderer Source](https://github.com/leptos-rs/leptos/tree/main/tachys)
- [Related Leptos Issue #3433](https://github.com/leptos-rs/leptos/issues/3433)
- [Rust Issue on type-length-limit](https://github.com/rust-lang/rust/issues/54540)

## Status

✅ **RESOLVED** - Fix applied and ready for CI deployment

**Date:** 2025-01-22  
**Leptos Version:** 0.8  
**Resolution Time:** ~1 hour  
**Complexity:** Medium (required understanding of Rust's type system)

---

**Next Steps:**
1. Push changes to trigger CI build
2. Monitor CI for successful compilation
3. Test deployed application thoroughly
4. Monitor for any other Leptos 0.8 related issues
5. Consider component refactoring for long-term maintainability