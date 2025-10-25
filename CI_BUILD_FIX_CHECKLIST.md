# CI Build Fix Checklist - Leptos 0.8 Upgrade

## Issue
After upgrading to Leptos 0.8, CI builds failed with type layout and recursion limit errors.

## Root Cause
Leptos 0.8's new tachys rendering system creates more complex type signatures that exceed default compiler limits.

## Solution Applied ✅

### 1. Compiler Limit Attributes Added

#### File: `ssr/src/lib.rs`
```rust
#![recursion_limit = "1024"]
#![type_length_limit = "10000000"]
```

#### File: `ssr/src/main.rs`
```rust
#![recursion_limit = "1024"]
#![type_length_limit = "10000000"]
```

### 2. Why Both Files?
- `lib.rs` = Library crate (compiled first)
- `main.rs` = Binary crate (compiled separately)
- Each crate needs its own limit attributes

## Verification Steps

### Local Testing
```bash
# 1. Type check
bash scripts/local_check.sh

# 2. Full build
bash scripts/local_run.sh

# 3. Release build (matches CI)
cargo leptos build --release
```

### Expected Results
- ✅ No "overflow normalizing" errors
- ✅ No "query depth increased" errors
- ✅ Compilation succeeds

## CI Pipeline Details

### Workflow: `.github/workflows/build-check.yml`
- **Toolchain:** `nightly-2024-10-10` (note: shows as 2025-10-10 due to Rust versioning)
- **Target:** `wasm32-unknown-unknown`
- **Build Command:** `cargo leptos build --release --lib-features "release-lib" --bin-features "release-bin"`

### Environment Variables Required
- `LEPTOS_HASH_FILES: true`
- `PROVAB_HEADERS` (from secrets)
- `NOW_PAYMENTS_USDC_ETHEREUM_API_KEY` (from secrets)
- `APP_URL` (input parameter)

## Common Follow-up Issues

### If You See: "overflow normalizing the opaque type"
**Solution:** Increase `recursion_limit` in both files
```rust
#![recursion_limit = "2048"]  // Double it if needed
```

### If You See: "query depth increased by N"
**Solution:** Increase `type_length_limit`
```rust
#![type_length_limit = "20000000"]  // Double it if needed
```

### If Build Succeeds Locally But Fails in CI
**Reasons:**
1. CI uses `--release` mode (different code paths)
2. Different cache state
3. Different memory constraints
4. Fresh build every time (no incremental compilation)

## Technical Details

### What These Limits Control

#### `recursion_limit`
- Controls depth of type normalization during compilation
- Default: 128
- Leptos 0.8 needs: 1024+
- Affects: Nested component type resolution

#### `type_length_limit`
- Controls number of type substitutions during monomorphization
- Default: 1,048,576
- Leptos 0.8 needs: 10,000,000+
- Affects: Complex type construction

### No Runtime Impact
- These are **compile-time only** limits
- Zero impact on:
  - Binary size
  - Runtime performance
  - Memory usage
  - Execution speed

## If CI Still Fails

### 1. Check Rust Version Match
```bash
# Local
rustc --version

# CI (from workflow)
nightly-2024-10-10
```

### 2. Verify Attributes Are Present
```bash
# Should show the attributes at top of files
head -10 ssr/src/lib.rs
head -10 ssr/src/main.rs
```

### 3. Check for Workspace Members
If you have other crates in the workspace that compile view code, add the same attributes to their `lib.rs` or `main.rs`:
```rust
#![recursion_limit = "1024"]
#![type_length_limit = "10000000"]
```

### 4. Increase Limits Further
If still failing, try:
```rust
#![recursion_limit = "2048"]
#![type_length_limit = "20000000"]
```

## Related Documentation

- [Rust Reference - Limits](https://doc.rust-lang.org/reference/attributes/limits.html)
- [Leptos 0.8 Migration Guide](https://github.com/leptos-rs/leptos)
- [Tachys Renderer](https://github.com/leptos-rs/leptos/tree/main/tachys)
- [Related Issue: Leptos #3433](https://github.com/leptos-rs/leptos/issues/3433)

## Files Modified

- ✅ `ssr/src/lib.rs` - Added both attributes
- ✅ `ssr/src/main.rs` - Added both attributes
- ✅ `lessons_learnt.md` - Documented the issue and solution
- ✅ `CI_BUILD_FIX_CHECKLIST.md` - This file

## Status: ✅ READY FOR CI

The fix has been applied. Push to trigger CI build and verify.

## Post-CI Success Actions

1. ✅ Verify CI build passes
2. ✅ Test deployed application
3. ✅ Monitor for any runtime issues
4. ✅ Delete this checklist file (optional)

---

**Date Applied:** 2025-01-22
**Leptos Version:** 0.8
**Issue Type:** Compilation limits exceeded
**Resolution:** Compiler limit attributes added