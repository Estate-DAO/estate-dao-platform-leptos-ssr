// use console_log::log;
use leptos::*;
use leptos_icons::*;
// use leptos_use::use_timestamp;
use leptos::logging::log;

/// year,  month, day
#[derive(Clone, Debug)]
struct SelectedDateRange {
    start: (u32, u32, u32),
    // start: RwSignal<(u32, u32, u32)>,
    end: (u32, u32, u32),
}

impl SelectedDateRange {
    fn to_string(&self) -> String {
        let start_str = format!(
            "{:04}-{:02}-{:02}",
            self.start.0, self.start.1, self.start.2
        );
        let end_str = format!("{:04}-{:02}-{:02}", self.end.0, self.end.1, self.end.2);
        format!("{} - {}", start_str, end_str)
    }
}

#[component]
pub fn DateTimeRangePickerCustom() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let selected_range: RwSignal<SelectedDateRange> = create_rw_signal(SelectedDateRange {
        start: (0, 0, 0),
        end: (0, 0, 0),
    });
    let (initial_date, set_initial_date) = create_signal((2024_u32, 1_u32));

    let next_month_date = Signal::derive(move || {
        let (current_year, current_month) = initial_date.get();
        next_date(current_year, current_month)
    });

    let date_range_display = create_memo(move |_prev| {
        let range = selected_range.get();
        if range.start == (0, 0, 0) && range.end == (0, 0, 0) {
            "Check in - Check out".to_string()
        } else {
            range.to_string()
        }
    });

    view! {
        <div class="">
            <div class="absolute inset-y-0 left-2 flex items-center text-2xl">
                <Icon icon=icondata::AiCalendarOutlined class="text-black font-light" />
            </div>

            <button
                class="w-full ml-2 py-2 pl-8 text-black bg-transparent border-none focus:outline-none text-sm text-left"
                on:click=move |_| set_is_open.update(|open| *open = !*open)
            >
                {{ move || date_range_display() }}
            </button>

            <Show when=move || is_open()>
                <div class="absolute mt-6  min-w-[40rem] bg-white border border-gray-200 rounded-xl shadow-lg p-4  z-50">
                   <div id="date-prev-next" class="flex justify-between">

                <button on:click={move |_| {
                    let (current_year, current_month) = initial_date.get_untracked();
                    set_initial_date(prev_date(current_year, current_month))
                }} class="hover:bg-gray-200 p-2 rounded-md">
                    <Icon icon=icondata::BiChevronLeftRegular class="text-black" />
                </button>

                    <button on:click={move |_| {
                        let (current_year, current_month) = initial_date.get_untracked();
                        set_initial_date(next_date(current_year, current_month))
                    }} class="hover:bg-gray-200 p-2 rounded-md">
                        <Icon icon=icondata::BiChevronRightRegular class="text-black" />
                    </button>
                </div>
                    <div class="flex space-x-8">
                        <DateCells
                            year_month=initial_date.into()
                            selected_range=selected_range
                        />
                        <DateCells
                            year_month=next_month_date.into()
                            selected_range=selected_range
                        />
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn prev_date(year: u32, month: u32) -> (u32, u32) {
    let value = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    value
}

fn next_date(year: u32, month: u32) -> (u32, u32) {
    let value = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    value
}

#[component]
fn DateCells(
    year_month: Signal<(u32, u32)>,
    selected_range: RwSignal<SelectedDateRange>,
) -> impl IntoView {
    let year_signal: Signal<u32> =  Signal::derive(move || {
        year_month.get().0
    });

    let month_signal: Signal<u32> =  Signal::derive( move || {
        year_month.get().1 
    });
    
    let start_month_day = create_rw_signal(0_u32);

    let days_in_month =  match month_signal() {
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
    };

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
        <div class="relative z-50">
            <div class="text-center font-bold mb-2">
                {move || format!("{} {}", month_names[(month_signal() - 1) as usize], year_signal())}
            </div>
            <div class="grid grid-cols-7 gap-x-2 gap-y-2.5 justify-items-center">
                
                {weekdays
                    .iter()
                    .map(|&day| view! { <div class="font-bold">{day}</div> })
                    .collect::<Vec<_>>()}

                {move || (0..calculate_starting_day_of_month(year_month, start_month_day)).map(|_| view! { <div></div> }).collect::<Vec<_>>()}

                
                {move || (1..=days_in_month)
                    .map(|day_num| {
                        let on_click = move |_val| {
                            let date_tuple = (year_signal(), month_signal(), day_num);
                            let range = selected_range.get();
                            // log!("Before update: start={:?}, end={:?}", range.start, range.end);
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
                                    start:  date_tuple,
                                    end: range.end,
                                }
                            } else if range.end == (0, 0, 0) {
                                if (year_signal(), month_signal(), day_num) > range.start {
                                    SelectedDateRange {
                                        start: range.start,
                                        end: date_tuple,
                                    }
                                } else {
                                    SelectedDateRange {
                                        start: date_tuple,
                                        end: range.start,
                                    }
                                }
                            } else {
                                SelectedDateRange {
                                    start: date_tuple,
                                    end: (0, 0, 0),
                                }
                            };
                            selected_range.set(new_range);
                            let updated_range = selected_range.get();
                            // log!(
                            //     "After update: start={:?}, end={:?}", updated_range.start, updated_range.end
                            // );
                        };
                        view! {
                            <button
                            class=move || class_signal(
                                selected_range.into(),
                                day_num,
                                year_signal(),
                                month_signal()
                            )
                            on:click=on_click
                        >
                            {day_num}
                        </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}

fn class_signal(
    selected_range: Signal<SelectedDateRange>,
    day_num: u32,
    year: u32,
    month: u32,
) -> String {
    let range = selected_range.get();
    format!(
        "border p-2 cursor-pointer w-10 rounded-md {} {}",
        if (year, month, day_num) == range.start || (year, month, day_num) == range.end {
            "bg-blue-500 text-white hover:bg-blue-600"
        } else {
            " hover:bg-gray-100"
        },
        if is_date_in_range(range.start, range.end, year, month, day_num) {
            "bg-gray-200"
        } else {
            ""
        },
    )
}

fn is_date_in_range(
    start: (u32, u32, u32),
    end: (u32, u32, u32),
    year: u32,
    month: u32,
    day_num: u32,
) -> bool {
    if start == (0, 0, 0) || end == (0, 0, 0) {
        false
    } else {
        let current_date = (year, month, day_num);
        current_date > start && current_date < end
    }
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
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
    let ans  = (h + 6) % 7;

    result.set(ans);
    ans
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