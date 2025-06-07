# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Estate DAO Frontend** is a hotel booking web application built with Rust and Leptos. The app includes a search interface (destination, dates, guests), hotel listings, room selection, and booking with crypto/traditional payments.

**Tech Stack:**
- **Leptos 0.6** - Rust web framework with SSR
- **Axum** - Web server
- **Internet Computer (IC)** - Blockchain backend
- **Payment providers** - NowPayments (crypto), Stripe
- **External APIs** - Provab hotel search API

## Essential Commands

### Development
```bash
# Run local development server
bash scripts/local_run.sh

# Type checking only
bash scripts/local_check.sh

# Run with mocked APIs (for development without external dependencies)
bash scripts/local_run_with_mock.sh

# Production build and run
bash scripts/prod_run.sh

# End-to-end tests
cargo leptos end-to-end

# Install pre-commit hooks
bash scripts/install_pre_commit.sh
```

### Single Test
```bash
cargo test --lib <test_name> -- --nocapture
```

## Architecture Overview

### Core Structure
```
ssr/src/
├── api/           # External integrations (Provab, payments, IC canister)
├── app.rs         # Main app component and routing
├── component/     # Reusable UI components
├── page/          # Route-specific page components  
├── state/         # Global state management (follows specific patterns)
├── utils/         # Utility functions
├── ssr_booking/   # Server-side booking pipeline
└── canister/      # Internet Computer integration
```

### Feature Flags System
- **local-lib/local-bin**: Local development
- **release-lib/release-bin**: Staging 
- **release-lib-prod/release-bin-prod**: Production
- **mock-provab**: Mock external hotel APIs
- **debug_log**: Enable debug logging
- **debug_display**: Show debug components in UI
- **ga4**: Google Analytics integration

## State Management Patterns

**Follow these patterns consistently:**

1. **Global State Structure** (see `ssr/src/state/api_error_state.rs` as reference):
   - Create a struct to represent state with `RwSignal` fields for mutable data
   - Implement `GlobalStateForLeptos` trait
   - Use static methods (no `self` parameter) for getters/setters
   - UI reactively updates when state changes

2. **Signal Usage**:
   - `RwSignal<T>` for mutable state
   - `Signal<T>` for read-only state
   - Access via `.get()` for reactive reads, `.get_untracked()` for non-reactive

3. **Component Props**:
   - Use `#[prop(into)]` when accepting signals
   - Mark optional props with `#[prop(optional)]`
   - Callbacks use `Callable::call(&callback, args)` pattern

## Code Style Requirements

### Leptos Component Guidelines
- **NEVER modify `move ||` blocks** - they handle reactivity correctly
- Use Tailwind CSS for all styling changes
- Comment changes with `// <!-- comment -->` format
- **Prefer commenting out unused code instead of deleting**
- Do not delete existing comments starting with `//`

### Error Handling
- Use `anyhow` for general errors
- Use `error-stack` for error context
- Use `thiserror` for custom error types

### Mobile Compatibility
- Use **CSS-only changes** for mobile responsiveness
- Responsive breakpoints via `leptos-use`

## Application Components

**Main UI Flow:**
1. **InputGroup** - Search interface with 3 components:
   - `DestinationPicker` - Searchable dropdown for cities
   - `DateRangePicker` - Calendar picker for check-in/out
   - `GuestQuantity` - Form for adults, children, rooms
2. **Hotel List** - Search results display
3. **Hotel Details** - Individual hotel with room selection
4. **Booking Flow** - Room blocking, payment, confirmation

## Required Environment Variables

```bash
# External API credentials
PROVAB_HEADERS="{...}"           # Hotel search API
NOW_PAYMENTS_USDC_ETHEREUM_API_KEY
STRIPE_SECRET_KEY

# Email integration  
EMAIL_CLIENT_ID
EMAIL_CLIENT_SECRET

# Basic auth for admin routes
BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE
BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE

# Development
NGROK_LOCALHOST_URL              # For local webhook testing
```

## Development Workflow

1. **Gather context** - Search functions/code paths/related files
2. **Plan changes** - Break down tasks, ensure no functionality breaks
3. **Implement changes** - Follow component and state patterns above
4. **Type check** - Run `bash scripts/local_check.sh`
5. **Test** - Verify changes work as expected

## Key Files for Common Tasks

- **State management**: `ssr/src/state/*.rs` - Follow patterns in `api_error_state.rs`
- **UI components**: `ssr/src/component/*.rs`
- **API integrations**: `ssr/src/api/*.rs`
- **Feature flags**: `ssr/Cargo.toml` [features] section
- **Build scripts**: `scripts/*.sh`