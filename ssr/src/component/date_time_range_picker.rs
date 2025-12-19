// use console_log::log;
use crate::view_state_layer::input_group_state::OpenDialogComponent;
use leptos::*;
use leptos_icons::*;
// use leptos_use::use_timestamp;
use crate::utils::date::*;
use crate::view_state_layer::{input_group_state::InputGroupState, ui_search_state::UISearchCtx};
use chrono::{Local, NaiveDate, TimeZone, Utc};
// use leptos::logging::log;
use crate::log;
use leptos_use::{
    core::Position, use_scroll, use_throttle_fn, use_throttle_fn_with_arg,
    use_timestamp_with_controls, UseScrollReturn, UseTimestampReturn,
};
use web_sys::{Element, TouchEvent};

/// year,  month, day
/// Struct is stored in the global search state - SearchCtx and accessed from there
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
pub struct SelectedDateRange {
    pub start: (u32, u32, u32),
    // start: RwSignal<(u32, u32, u32)>,
    pub end: (u32, u32, u32),
}

impl SelectedDateRange {
    pub fn to_string(&self) -> String {
        let start_str = format!(
            "{:04}-{:02}-{:02}",
            self.start.0, self.start.1, self.start.2
        );
        let end_str = format!("{:04}-{:02}-{:02}", self.end.0, self.end.1, self.end.2);
        format!("{} - {}", start_str, end_str)
    }
    pub fn end_to_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.end.0, self.end.1, self.end.2)
    }
    pub fn start_to_string(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}",
            self.start.0, self.start.1, self.start.2
        )
    }

    pub fn display_string(&self) -> String {
        if self.start == (0, 0, 0) && self.end == (0, 0, 0) {
            return "Check in - Check out".to_string();
        }

        // Ensure dates are in correct order
        let (start_date, end_date) = if self.start != (0, 0, 0) && self.end != (0, 0, 0) {
            if self.start > self.end {
                (self.end, self.start)
            } else {
                (self.start, self.end)
            }
        } else {
            (self.start, self.end)
        };

        let format_date = |(y, m, d): (u32, u32, u32)| -> String {
            if (y, m, d) == (0, 0, 0) {
                return "".to_string();
            }
            let suffix = match d {
                1 | 21 | 31 => "st",
                2 | 22 => "nd",
                3 | 23 => "rd",
                _ => "th",
            };
            let month = match m {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "",
            };
            format!("{d}{suffix} {month} {y}")
        };

        if start_date == (0, 0, 0) {
            return format!("Check in - {}", format_date(end_date));
        }
        if end_date == (0, 0, 0) {
            return format!("{} - Check out", format_date(start_date));
        }

        let (sy, sm, sd) = start_date;
        let (ey, em, ed) = end_date;

        // Case: same month & year → "14th-18th July 2025"
        if sy == ey && sm == em {
            let suffix_start = match sd {
                1 | 21 | 31 => "st",
                2 | 22 => "nd",
                3 | 23 => "rd",
                _ => "th",
            };
            let suffix_end = match ed {
                1 | 21 | 31 => "st",
                2 | 22 => "nd",
                3 | 23 => "rd",
                _ => "th",
            };
            let month = match sm {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "",
            };
            return format!("{sd}{suffix_start}-{ed}{suffix_end} {month} {sy}");
        }

        // Otherwise → fallback to full dates
        format!("{} - {}", format_date(start_date), format_date(end_date))
    }

    pub fn no_of_nights(&self) -> u32 {
        let (start_year, start_month, start_day) = self.start;
        let (end_year, end_month, end_day) = self.end;

        if self.start == (0, 0, 0) || self.end == (0, 0, 0) {
            return 0;
        }

        let start_date = chrono::NaiveDate::from_ymd_opt(start_year as i32, start_month, start_day);
        let end_date = chrono::NaiveDate::from_ymd_opt(end_year as i32, end_month, end_day);

        if let (Some(start), Some(end)) = (start_date, end_date) {
            if end > start {
                return (end - start).num_days() as u32;
            }
        }
        0
    }

    pub fn normalize(&self) -> Self {
        if self.start == (0, 0, 0) || self.end == (0, 0, 0) {
            return self.clone();
        }

        // If start date is after end date, swap them
        if self.start > self.end {
            return SelectedDateRange {
                start: self.end,
                end: self.start,
            };
        }

        self.clone()
    }

    pub fn format_date(date: (u32, u32, u32)) -> String {
        format!("{:02}-{:02}-{:04}", date.2, date.1, date.0)
    }
    pub fn format_as_human_readable_date(&self) -> String {
        let format_date = |(year, month, day): (u32, u32, u32)| {
            NaiveDate::from_ymd_opt(year as i32, month, day)
                .map(|d| d.format("%a, %b %d").to_string())
                .unwrap_or_default()
        };

        format!("{} - {}", format_date(self.start), format_date(self.end))
    }

    // <!-- Added: Returns date range in format 'Apr, 26 - Apr, 27' (MMM, DD - MMM, DD) -->
    pub fn format_mmm_dd(&self) -> String {
        use chrono::NaiveDate;
        let format_md = |(year, month, day): (u32, u32, u32)| {
            NaiveDate::from_ymd_opt(year as i32, month, day)
                .map(|d| d.format("%b, %d").to_string())
                .unwrap_or_default()
        };
        format!("{} - {}", format_md(self.start), format_md(self.end))
    }

    // <!-- Returns date in format '04 April 2025' given (year, month, day) -->
    fn dd_month_yyyy(date: (u32, u32, u32)) -> String {
        use chrono::NaiveDate;
        NaiveDate::from_ymd_opt(date.0 as i32, date.1, date.2)
            .map(|d| d.format("%d %B %Y").to_string())
            .unwrap_or("-".to_string())
    }

    pub fn dd_month_yyyy_start(&self) -> String {
        Self::dd_month_yyyy(self.start)
    }

    pub fn dd_month_yyyy_end(&self) -> String {
        Self::dd_month_yyyy(self.end)
    }

    pub fn format_dd_month_yyyy(&self) -> String {
        format!(
            "{} - {}",
            Self::dd_month_yyyy(self.start),
            Self::dd_month_yyyy(self.end)
        )
    }

    // <!-- Returns date range in format '04 Nov - 08 Nov' -->
    pub fn format_dd_month_short(&self) -> String {
        use chrono::NaiveDate;
        let format_dm = |(year, month, day): (u32, u32, u32)| {
            NaiveDate::from_ymd_opt(year as i32, month, day)
                .map(|d| d.format("%d %b").to_string())
                .unwrap_or_default()
        };
        format!("{} - {}", format_dm(self.start), format_dm(self.end))
    }

    // <!-- Returns formatted nights string, e.g. '2 Nights' or '-' if none -->
    pub fn formatted_nights(&self) -> String {
        let nights = self.no_of_nights();
        if nights > 0 {
            format!("{} Night{}", nights, if nights > 1 { "s" } else { "" })
        } else {
            "-".to_string()
        }
    }
}

#[component]
pub fn DateTimeRangePickerCustom(
    #[prop(optional, into)] h_class: MaybeSignal<String>,
) -> impl IntoView {
    let h_class = create_memo(move |_| {
        let class = h_class.get();
        if class.is_empty() {
            "h-full".to_string()
        } else {
            class
        }
    });
    let is_open = create_memo(move |_| InputGroupState::is_date_open());
    let search_ctx: UISearchCtx = expect_context();
    let selected_range = search_ctx.date_range;
    let (initial_date, set_initial_date) = create_signal((2024_u32, 1_u32));
    let calendar_ref = create_node_ref::<leptos::html::Div>();

    let next_month_date = Signal::derive(move || {
        let (current_year, current_month) = initial_date.get();
        next_date(current_year, current_month)
    });

    let date_range_display = create_memo(move |_prev| {
        let range = selected_range.get();
        range.display_string()
    });

    // Create scroll handler using use_scroll
    let UseScrollReturn { y, .. } = use_scroll(calendar_ref);

    // Track last scroll position
    let last_scroll_position = create_rw_signal(0.0);

    // Throttle scroll changes
    let throttled_scroll = use_throttle_fn(
        move || {
            let current_y = y.get();
            log!("[Scroll] position y = {:?}", current_y);
            let last_y = last_scroll_position.get();
            log!("[Scroll] last y = {:?}", last_y);
            let delta_y = current_y - last_y;
            log!("[Scroll] delta_y = {}", delta_y);
            // Only handle significant scroll (>50px)
            if delta_y.abs() > 50.0 {
                let (year, month) = initial_date.get_untracked();
                if delta_y < 0.0 {
                    log!("[Scroll] scrolling up: next month");
                    set_initial_date(next_date(year, month));
                } else {
                    log!("[Scroll] scrolling down: prev month");
                    set_initial_date(prev_date(year, month));
                }
                last_scroll_position.set(current_y);
            }
        },
        300.0,
    );

    create_effect(move |_| {
        // reactive dependency on scroll position
        let _ = y.get();
        throttled_scroll();
    });

    let UseTimestampReturn {
        timestamp,
        is_active,
        pause,
        resume,
    } = use_timestamp_with_controls();

    // pause.pause();
    log!("timestamp: {:?}", timestamp.get_untracked());

    let (year, month) = get_year_month(timestamp.get_untracked());
    set_initial_date((year, month));

    // --- Calendar Modal Scroll Lock Effect ---

    create_effect(move |_| {
        log!("[Dialog] open state = {:?}", is_open.get());
        use web_sys::window;
        let is_open = is_open.get();
        let document = window().and_then(|w| w.document());
        if let Some(body) = document.and_then(|d| d.body()) {
            if is_open {
                let _ = body.class_list().add_1("overflow-hidden md:overflow-auto");
            } else {
                let _ = body
                    .class_list()
                    .remove_1("overflow-hidden md:overflow-auto");
            }
        }
        // Clear calendar_ref when dialog is closed to allow re-binding
        // if !is_open {
        //     calendar_ref.set(None);
        // }
    });

    // Touch swipe handling for mobile
    let touch_start_y = create_rw_signal(0.0);
    let touch_throttled = move |delta: f64| {
        if delta.abs() > 50.0 {
            let (year, month) = initial_date.get_untracked();
            if delta < 0.0 {
                set_initial_date(next_date(year, month));
            } else {
                set_initial_date(prev_date(year, month));
            }
        }
    };

    // let touch_throttled = use_throttle_fn_with_arg(
    //     move |delta: f64| {
    //         if delta.abs() > 50.0 {
    //             let (year, month) = initial_date.get_untracked();
    //             if delta < 0.0 {
    //                 set_initial_date(next_date(year, month));
    //             } else {
    //                 set_initial_date(prev_date(year, month));
    //             }
    //         }
    //     },
    //     300.0,
    // );

    view! {
        <div class="relative w-full py-2">
            <div class="absolute inset-y-0 left-2 flex items-center text-xl">
                <Icon icon=icondata::AiCalendarOutlined class="text-blue-500 font-extralight"/>
            </div>

            <button
                class=move || {
                    format!(
                        "w-full {} h-full pl-14 pr-3 text-[15px] leading-[18px] text-gray-900 font-medium bg-transparent border-none rounded-md focus:outline-none text-left",
                        h_class(),
                    )
                }
                on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::DateComponent)
            >
                {{
                    move || {
                        view! { <span class="text-black font-medium truncate">{date_range_display()}</span> }
                    }
                }}
            </button>

            <Show when=move || is_open()>
                // --- MOBILE: full-screen modal ---
                <div
                    class="fixed inset-0 z-[99999] bg-white md:hidden flex flex-col"
                    style="touch-action: none; overscroll-behavior: contain; isolation: isolate;"
                >
                    // Header
                    <div class="flex items-center justify-between px-4 py-4 border-b border-gray-100">
                        <h2 class="text-lg font-semibold text-gray-900">"Select Dates"</h2>
                        <button
                            type="button"
                            class="p-2 rounded-full hover:bg-gray-100 transition-colors"
                            on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::None)
                        >
                            <svg class="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>

                    // Check-in / Nights / Check-out row
                    <div class="flex items-center justify-between px-4 py-4 border-b border-gray-100">
                        <div class="text-left">
                            <p class="text-xs text-gray-500">"Check-in"</p>
                            <p class="text-sm font-medium text-gray-900">
                                {move || {
                                    let range = selected_range.get();
                                    range.dd_month_yyyy_start()
                                }}
                            </p>
                        </div>
                        <div class="px-3 py-1 bg-blue-500 text-white text-xs font-medium rounded-full">
                            {move || selected_range.get().formatted_nights()}
                        </div>
                        <div class="text-right">
                            <p class="text-xs text-gray-500">"Check-out"</p>
                            <p class="text-sm font-medium text-gray-900">
                                {move || {
                                    let range = selected_range.get();
                                    range.dd_month_yyyy_end()
                                }}
                            </p>
                        </div>
                    </div>

                    // Calendar content (scrollable)
                    <div
                        _ref=calendar_ref
                        class="flex-1 overflow-y-auto px-4 py-4"
                        style="-webkit-overflow-scrolling: touch;"
                    >
                        {date_picker_mobile_calendar_content(initial_date, selected_range)}
                    </div>

                    // Apply button
                    <div class="px-4 py-4 border-t border-gray-100 bg-white">
                        <button
                            type="button"
                            class=move || {
                                let range = selected_range.get();
                                let has_both_dates = range.start != (0, 0, 0) && range.end != (0, 0, 0);
                                if has_both_dates {
                                    "w-full bg-blue-500 text-white py-3 rounded-full font-medium hover:bg-blue-600 transition-colors"
                                } else {
                                    "w-full bg-gray-300 text-gray-500 py-3 rounded-full font-medium cursor-not-allowed"
                                }
                            }
                            disabled=move || {
                                let range = selected_range.get();
                                !(range.start != (0, 0, 0) && range.end != (0, 0, 0))
                            }
                            on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::None)
                        >
                            "Apply"
                        </button>
                    </div>
                </div>

                // --- DESKTOP: dropdown positioned relative to button ---
                <div
                    class="absolute top-full left-1/2 -translate-x-1/2 mt-2 z-[10000] hidden md:block"
                    on:click=|e| e.stop_propagation()
                >
                    <div class="bg-white rounded-lg shadow-lg border border-gray-200 w-[600px]">
                        {date_picker_inner_content(initial_date, next_month_date, selected_range, calendar_ref)}
                    </div>
                </div>
            </Show>

        </div>
    }
}

// Mobile calendar content - shows current month + next months in scrollable view
fn date_picker_mobile_calendar_content(
    initial_date: ReadSignal<(u32, u32)>,
    selected_range: RwSignal<SelectedDateRange>,
) -> impl IntoView {
    // Generate 12 months worth of calendar starting from initial_date
    let months_to_show = create_memo(move |_| {
        let (start_year, start_month) = initial_date.get();
        let mut months = Vec::new();
        let mut year = start_year;
        let mut month = start_month;

        for _ in 0..12 {
            months.push((year, month));
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
        months
    });

    view! {
        <div class="space-y-8">
            <For
                each=move || months_to_show.get()
                key=|(y, m)| (*y, *m)
                let:year_month
            >
                <DateCells
                    year_month=Signal::derive(move || year_month)
                    selected_range=selected_range
                />
            </For>
        </div>
    }
}

fn date_picker_inner_content(
    initial_date: ReadSignal<(u32, u32)>,
    next_month_date: Signal<(u32, u32)>,
    selected_range: RwSignal<SelectedDateRange>,
    calendar_ref: NodeRef<html::Div>,
) -> impl IntoView {
    view! {
        <div
            _ref=calendar_ref
            class="flex flex-col md:flex-row gap-8 p-4 overflow-y-auto"
            style="max-height: 70vh; -webkit-overflow-scrolling: touch;"
        >
            <div class="flex-1">
                <DateCells year_month=initial_date.into() selected_range=selected_range />
            </div>
            <div class="flex-1">
                <DateCells year_month=next_month_date.into() selected_range=selected_range />
            </div>
        </div>

        <div class="bg-white px-4 py-3 flex justify-center border-t">
            <button
                type="button"
                class=move || {
                    let range = selected_range.get();
                    let has_both_dates = range.start != (0, 0, 0)
                        && range.end != (0, 0, 0);
                    if has_both_dates {
                        "w-full md:w-48 bg-blue-500 text-white py-2 rounded-full hover:bg-blue-600 transition-colors"
                    } else {
                        "w-full md:w-48 bg-gray-300 text-gray-500 py-2 rounded-full cursor-not-allowed"
                    }
                }
                disabled=move || {
                    let range = selected_range.get();
                    !(range.start != (0, 0, 0) && range.end != (0, 0, 0))
                }
                on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::None)
            >
                "Apply"
            </button>
        </div>
    }
}

#[component]
fn DateCells(
    year_month: Signal<(u32, u32)>,
    selected_range: RwSignal<SelectedDateRange>,
) -> impl IntoView {
    let year_signal: Signal<u32> = Signal::derive(move || year_month.get().0);

    let month_signal: Signal<u32> = Signal::derive(move || year_month.get().1);

    let start_month_day = create_rw_signal(0_u32);

    let days_in_month = create_memo(move |_| match month_signal() {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year_signal()) {
                29
            } else {
                28
            }
        }
        _ => 0,
    });

    let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    view! {
        <div>
            // !<-- Month Title -->
            <div class="text-center font-medium text-lg mb-4">
                {move || {
                    let month_name = month_names[(month_signal() - 1) as usize];
                    format!("{} {}", month_name, year_signal())
                }}
            </div>

            // !<-- Calendar Grid -->
            <div class="grid grid-cols-7">
                // !<-- Weekday Headers -->
                {weekdays
                    .iter()
                    .map(|&day| view! { <div class="text-xs text-gray-500 font-medium text-center mb-2">{day}</div> })
                    .collect::<Vec<_>>()}

                // !<-- Empty Cells -->
                {move || {
                    (0..calculate_starting_day_of_month(year_month, start_month_day))
                        .map(|_| view! { <div class="w-9 h-9 flex items-center justify-center"></div> })
                        .collect::<Vec<_>>()
                }}

                // !<-- Date Cells -->
                {move || {
                    (1..=days_in_month())
                        .map(|day_num| {
                            let on_click = move |_val| {
                                if is_date_in_past(year_signal(), month_signal(), day_num) {
                                    return;
                                }

                                let date_tuple = (year_signal(), month_signal(), day_num);
                                let range = selected_range.get();
                                let new_range = if range.start == date_tuple {
                                    SelectedDateRange {
                                        start: (0, 0, 0),
                                        end: range.end,
                                    }
                                } else if range.end == date_tuple {
                                    SelectedDateRange {
                                        start: range.start,
                                        end: (0, 0, 0),
                                    }
                                } else if range.start == (0, 0, 0) {
                                    SelectedDateRange {
                                        start: date_tuple,
                                        end: range.end,
                                    }
                                } else if range.end == (0, 0, 0) {
                                    // If we're selecting the end date, ensure it's after the start date
                                    if date_tuple > range.start {
                                        SelectedDateRange {
                                            start: range.start,
                                            end: date_tuple,
                                        }
                                    } else {
                                        // If the selected date is before the start date,
                                        // make it the new start date and make the old start date the end date
                                        SelectedDateRange {
                                            start: date_tuple,
                                            end: range.start,
                                        }
                                    }
                                } else {
                                    // If both dates are already set and we're selecting a new date,
                                    // start a new selection with this date as the start date
                                    SelectedDateRange {
                                        start: date_tuple,
                                        end: (0, 0, 0),
                                    }
                                };
                                selected_range.set(new_range.normalize());
                                let updated_range = selected_range.get();
                            };
                            view! {
                                <button
                                    class=move || class_signal(
                                        selected_range.into(),
                                        day_num,
                                        year_signal(),
                                        month_signal(),
                                    )
                                    on:click=on_click
                                >
                                    <span class=move || inner_span_class(
                                        selected_range.into(),
                                        day_num,
                                        year_signal(),
                                        month_signal(),
                                    )>
                                        {day_num}
                                    </span>
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

pub fn class_signal(
    selected_range: Signal<SelectedDateRange>,
    day_num: u32,
    year: u32,
    month: u32,
) -> String {
    let range = selected_range.get();
    let date_tuple = (year, month, day_num);
    // Use full width cell with centered content
    let base_classes = "w-full h-10 flex items-center justify-center text-sm transition-colors";

    // !<-- Past dates -->
    if is_date_in_past(year, month, day_num) {
        return format!("{} text-gray-300 cursor-not-allowed", base_classes);
    }

    // Normalize range to ensure start < end
    let (start, end) = if range.start != (0, 0, 0) && range.end != (0, 0, 0) {
        if range.start > range.end {
            (range.end, range.start)
        } else {
            (range.start, range.end)
        }
    } else {
        (range.start, range.end)
    };

    let has_range = start != (0, 0, 0) && end != (0, 0, 0);

    // !<-- Start date: full circle with light background extending right -->
    if date_tuple == start && start != (0, 0, 0) {
        if has_range {
            // Has range: blue square + light blue background on right half, clipped with rounded-l-md
            return format!(
                "{} bg-gradient-to-r from-transparent from-50% to-blue-100 to-50% overflow-hidden rounded-l-md",
                base_classes
            );
        } else {
            // Only start selected, no end yet
            return format!("{}", base_classes);
        }
    }

    // !<-- End date: full circle with light background extending left -->
    if date_tuple == end && end != (0, 0, 0) {
        if has_range {
            // Has range: blue square + light blue background on left half, clipped with rounded-r-md
            return format!(
                "{} bg-gradient-to-l from-transparent from-50% to-blue-100 to-50% overflow-hidden rounded-r-md",
                base_classes
            );
        } else {
            return format!("{}", base_classes);
        }
    }

    // !<-- Dates in range (full light background) -->
    if has_range && is_date_between(date_tuple, start, end) {
        return format!("{} bg-blue-100 text-blue-900", base_classes);
    }

    // !<-- Default state -->
    format!("{} hover:bg-gray-100 rounded-full", base_classes)
}

/// Returns the class for the inner span (date number) - shows circular highlight for selected dates
pub fn inner_span_class(
    selected_range: Signal<SelectedDateRange>,
    day_num: u32,
    year: u32,
    month: u32,
) -> String {
    let range = selected_range.get();
    let date_tuple = (year, month, day_num);
    let base_inner = "w-9 h-9 flex items-center justify-center text-sm rounded-full";

    // Normalize range
    let (start, end) = if range.start != (0, 0, 0) && range.end != (0, 0, 0) {
        if range.start > range.end {
            (range.end, range.start)
        } else {
            (range.start, range.end)
        }
    } else {
        (range.start, range.end)
    };

    // Start or end date: show blue circle with rounded-md
    if (date_tuple == start && start != (0, 0, 0)) || (date_tuple == end && end != (0, 0, 0)) {
        return "w-9 h-9 flex items-center justify-center text-sm rounded-md bg-blue-500 text-white font-medium".to_string();
    }

    // Date in range: just text, background is on parent
    if start != (0, 0, 0) && end != (0, 0, 0) && is_date_between(date_tuple, start, end) {
        return format!("{} text-blue-900", base_inner);
    }

    // Past dates
    if is_date_in_past(year, month, day_num) {
        return format!("{} text-gray-300", base_inner);
    }

    // Default
    base_inner.to_string()
}

/// Checks if a date is in the past (before today)
fn is_date_in_past(year: u32, month: u32, day: u32) -> bool {
    let today = Local::now().date_naive();

    if let Some(check_date) = NaiveDate::from_ymd_opt(year as i32, month, day) {
        return check_date < today;
    }

    false
}

/// Checks if a date is between two other dates (inclusive)
fn is_date_between(date: (u32, u32, u32), start: (u32, u32, u32), end: (u32, u32, u32)) -> bool {
    // Safely convert to NaiveDate, returning false if any date is invalid
    let date_opt = NaiveDate::from_ymd_opt(date.0 as i32, date.1, date.2);
    let start_opt = NaiveDate::from_ymd_opt(start.0 as i32, start.1, start.2);
    let end_opt = NaiveDate::from_ymd_opt(end.0 as i32, end.1, end.2);

    // Only proceed if all dates are valid
    if let (Some(date), Some(start), Some(end)) = (date_opt, start_opt, end_opt) {
        return date >= start && date <= end;
    }

    false
}

fn calculate_starting_day_of_month(year_month: Signal<(u32, u32)>, result: RwSignal<u32>) -> u32 {
    let day = 1;
    let (year, month) = year_month.get();
    // log!("year: {}, month: {}, day: {}",year, month, day);
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 12 } else { month };
    let k = y % 100;
    let j = y / 100;
    let h = (day + 13 * (m + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    let ans = (h + 6) % 7;

    result.set(ans);
    ans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_of_nights() {
        let date_range = SelectedDateRange {
            start: (2023, 10, 1),
            end: (2023, 10, 5),
        };
        assert_eq!(date_range.no_of_nights(), 4);

        let date_range_same_day = SelectedDateRange {
            start: (2023, 10, 1),
            end: (2023, 10, 1),
        };
        assert_eq!(date_range_same_day.no_of_nights(), 0);

        let date_range_invalid = SelectedDateRange {
            start: (0, 0, 0),
            end: (2023, 10, 5),
        };
        assert_eq!(date_range_invalid.no_of_nights(), 0);

        let date_range_end_before_start = SelectedDateRange {
            start: (2023, 10, 5),
            end: (2023, 10, 1),
        };
        assert_eq!(date_range_end_before_start.no_of_nights(), 0);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_calculate_weekday() {
//         let test_cases = vec![
//             ((2023, 10, 1), 0), // Sunday
//             ((2023, 10, 2), 1), // Monday
//             ((2023, 10, 3), 2), // Tuesday
//             ((2023, 10, 4), 3), // Wednesday
//             ((2023, 10, 5), 4), // Thursday
//             ((2023, 10, 6), 5), // Friday
//             ((2023, 10, 7), 6), // Saturday
//             ((2024, 2, 29), 4), // Thursday (leap year)
//             ((2023, 2, 28), 2), // Tuesday (non-leap year)
//         ];

//         for ((year, month, day), expected) in test_cases.iter() {
//             assert_eq!(calculate_weekday(*year, *month, *day), *expected);
//         }
//     }
// }
