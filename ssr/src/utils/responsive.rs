use leptos::prelude::*;
use leptos_use::{breakpoints_tailwind, use_breakpoints, BreakpointsTailwind};

/// Signal that is true when the screen is at least `sm` (Tailwind breakpoint, >=640px), i.e. desktop/tablet.
/// Returns false for mobile (xs).
pub fn use_is_desktop() -> Signal<bool> {
    let screen_width = use_breakpoints(breakpoints_tailwind());
    screen_width.ge(BreakpointsTailwind::Sm)
}

/// Signal that is true when the screen is strictly mobile (below `sm`, <640px).
pub fn use_is_mobile() -> Signal<bool> {
    let screen_width = use_breakpoints(breakpoints_tailwind());
    screen_width.lt(BreakpointsTailwind::Sm)
}
