// use console_log::log;
use crate::state::input_group_state::OpenDialogComponent;
use leptos::*;
use leptos_icons::*;
// use leptos_use::use_timestamp;
use crate::state::{input_group_state::InputGroupState, search_state::SearchCtx};
use crate::utils::date::*;
use chrono::{Local, NaiveDate, TimeZone, Utc};
// use leptos::logging::log;
use crate::log;
use leptos_use::{use_timestamp_with_controls, UseTimestampReturn};

/// year,  month, day
/// Struct is stored in the global search state - SearchCtx and accessed from there
#[derive(Clone, Debug, Default, serde::Serialize)]
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

    pub fn display_string(&self) -> String {
        if self.start == (0, 0, 0) && self.end == (0, 0, 0) {
            return "Check in - Check out".to_string();
        }

        // Ensure we're displaying dates in the correct order
        let (start_date, end_date) = if self.start != (0, 0, 0) && self.end != (0, 0, 0) {
            if self.start > self.end {
                // If dates are in wrong order, swap them in the display
                (self.end, self.start)
            } else {
                (self.start, self.end)
            }
        } else {
            (self.start, self.end)
        };

        let start_str = if start_date == (0, 0, 0) {
            "Check in".to_string()
        } else {
            format!(
                "{:04}-{:02}-{:02}",
                start_date.0, start_date.1, start_date.2
            )
        };

        let end_str = if end_date == (0, 0, 0) {
            "Check out".to_string()
        } else {
            format!("{:04}-{:02}-{:02}", end_date.0, end_date.1, end_date.2)
        };

        format!("{} - {}", start_str, end_str)
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
}

#[component]
pub fn DateTimeRangePickerCustom() -> impl IntoView {
    // let (is_open, set_is_open) = create_signal(false);
    let is_open = create_memo(move |_| {
        // log!("is_open called");
        InputGroupState::is_date_open()
    });

    let search_ctx: SearchCtx = expect_context();

    let selected_range = search_ctx.date_range;

    let (initial_date, set_initial_date) = create_signal((2024_u32, 1_u32));

    let next_month_date = Signal::derive(move || {
        let (current_year, current_month) = initial_date.get();
        next_date(current_year, current_month)
    });

    let date_range_display = create_memo(move |_prev| {
        let range = selected_range.get();
        range.display_string()
    });

    // todo: find a way to not set signal from effect
    create_effect(move |_| {
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
    });

    view! {
        <div class="bg-white">
            <div class="absolute inset-y-0 left-2 flex items-center text-2xl">
                <Icon icon=icondata::AiCalendarOutlined class="text-black font-light" />
            </div>

            <button
                class="w-full ml-2 py-2 pl-8 text-black bg-transparent border-none focus:outline-none text-sm text-left"
                on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::DateComponent)
            >
                {{ move || date_range_display() }}
            </button>

            <Show when=move || is_open()>
                <div class="fixed inset-0 bg-white z-[100] md:bg-black/30 md:backdrop-blur-sm">
                    <div class="fixed bottom-0 left-0 right-0 top-auto md:top-1/2 md:left-1/2 md:-translate-x-1/2 md:-translate-y-1/2 md:fixed md:max-w-[600px] md:w-[600px] md:rounded-2xl md:shadow-xl md:border md:border-gray-200 md:p-0 md:mx-auto md:my-auto md:z-50">
                        <div class="p-4 md:p-8 rounded-t-2xl md:rounded-2xl h-full w-full">
                            {/* Calendar Header */}
                            <div class="flex justify-between items-center mb-8">
                                <button
                                    on:click=move |_| {
                                        let (current_year, current_month) = initial_date.get_untracked();
                                        set_initial_date(prev_date(current_year, current_month))
                                    }
                                    class="hover:bg-gray-100 p-1.5 rounded-full"
                                >
                                    <Icon icon=icondata::BiChevronLeftRegular class="text-black text-lg" />
                                </button>

                                <button
                                    on:click=move |_| {
                                        let (current_year, current_month) = initial_date.get_untracked();
                                        set_initial_date(next_date(current_year, current_month))
                                    }
                                    class="hover:bg-gray-100 p-1.5 rounded-full"
                                >
                                    <Icon icon=icondata::BiChevronRightRegular class="text-black text-lg" />
                                </button>
                            </div>

                            {/* Calendar Grid */}
                            <div class="flex flex-col md:flex-row md:gap-16 space-y-8 md:space-y-0">
                                <div class="flex-1">
                                    <DateCells year_month=initial_date.into() selected_range=selected_range />
                                </div>
                                <div class="flex-1">
                                    <DateCells
                                        year_month=next_month_date.into()
                                        selected_range=selected_range
                                    />
                                </div>
                            </div>

                            {/* Action Button */}
                            <Show
                                when=move || {
                                    let range = selected_range.get();
                                    range.start != (0, 0, 0) && range.end != (0, 0, 0)
                                }
                            >
                                <button
                                    type="button"
                                    class="w-full mt-8 mb-2 bg-blue-500 md:bg-white text-white md:text-black md:border md:border-black-2 py-3 md:py-2 rounded-full"
                                    on:click=move |_| InputGroupState::toggle_dialog(OpenDialogComponent::None)
                                >
                                    "Apply"
                                </button>
                            </Show>
                        </div>
                    </div>
                </div>
            </Show>
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
            {/* Month Title */}
            <div class="text-left font-semibold text-base mb-6">
                {move || {
                    format!("{} {}", month_names[(month_signal() - 1) as usize], year_signal())
                }}
            </div>

            {/* Calendar Grid */}
            <div class="grid grid-cols-7 gap-y-2">
                {/* Weekday Headers */}
                {weekdays
                    .iter()
                    .map(|&day| view! { <div class="text-[11px] text-gray-500 font-medium text-center mb-1">{day}</div> })
                    .collect::<Vec<_>>()}

                {/* Empty cells for month start alignment */}
                {move || {
                    (0..calculate_starting_day_of_month(year_month, start_month_day))
                        .map(|_| view! { <div class="w-8 h-8"></div> })
                        .collect::<Vec<_>>()
                }}

                {/* Date Cells */}
                {move || {
                    (1..=days_in_month())
                        .map(|day_num| {
                            let on_click = move |_val| {
                                // Check if date is in the past
                                if is_date_in_past(year_signal(), month_signal(), day_num) {
                                    return; // Don't allow selecting past dates
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
                                    <span class="text-xs font-normal">{day_num}</span>
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
    let base_classes = "w-8 h-8 rounded-full flex items-center justify-center";

    // Check if date is in the past
    let is_past_date = is_date_in_past(year, month, day_num);

    if is_past_date {
        return format!("{} {} {}", base_classes, "text-gray-300 cursor-not-allowed", "bg-white");
    }

    if range.start == date_tuple {
        return format!("{} {} {}", base_classes, "bg-black text-white", "hover:bg-gray-900");
    }

    if range.end == date_tuple {
        return format!("{} {} {}", base_classes, "bg-black text-white", "hover:bg-gray-900");
    }

    if range.start != (0, 0, 0)
        && range.end != (0, 0, 0)
        && is_date_between(date_tuple, range.start, range.end)
    {
        return format!("{} {} {}", base_classes, "bg-gray-100", "hover:bg-gray-200");
    }

    format!("{} {} {}", base_classes, "bg-white", "hover:bg-gray-100")
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
