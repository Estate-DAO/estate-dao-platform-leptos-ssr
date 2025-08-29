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

### Development (Script-based)
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

### Development (Just-based - Alternative)
```bash
# Run local development server
just dev

# Type checking only
just check

# Run with mocked APIs
just dev-mock

# Run with debug display
just dev-debug

# View logs
just logs

# Run specific test
just test <test_name>

# Production build and run
just prod
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
- **mock-block-room-fail**: Mock room blocking failures (testing)
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
- **Task runner**: `justfile` - Alternative commands using Just task runner
- **Logs**: `logs/estate-fe.log` - Current day's logs (symlinked), check with `tail -f logs/estate-fe.log`

## Memories
- use working_method.md to work on issues
- you are a senior software engineer who is good at making LLD designs. for current context, read CLAUDE.md and notes from notes/ folder if you need to know the approach / codebase awareness.
- amenities is also called facilities in the litepai adapter. they mean the same thing and are used interchangably.
- when writing a file to notes folder, prefix with the next number in the folder <three digit padded number>-<type>-<filename>.md
- log files for the local development is at `logs/estate-fe.log` make sure to read it from end (tail) when debugging with logs . api issues etc.
- when implementing a plan, keep updating the checklist as you finish the tasks fro the checklist. see if you can do multiple tasks in checklist parallely - by spawning sub agents
- You run in an environment where ast-grep ('sg') is available; whenever a search requires syntax-aware or structural matching, default to 'sg --lang rust -p '<pattern>'' (or set '--lang' appropriately) and avoid falling back to text-only tools like 'rg' or 'grep' unless I explicitly request a plain-text search.
- the website is running on port 3002, after you make the changes, only check via bash scripts/local_check.sh
-  **IMPORTANT CAUTION**: do not try to kill the estate-fe server. Do not run bash scripts/local_run.sh
-  you have grep-app MCP server available to you. use it to search for specific function calls across github. this mcp server will help you understand how the method is written in other codebases. WHen using SearchCode tool, write queries like fn <function_name>  or use <module_name> . this will give you a list of numbered files. You can ask full version of those files in subsequent requests -- ask only 1,2 files at a time. 