// use console_log::log;
use leptos::*;
use leptos_icons::*;
use leptos_use::use_timestamp;

use chrono::*;
use leptos::logging::log;



/// year,  month, day
#[derive(Clone, Debug)]
struct SelectedDateRange {
    
    start: (u32, u32, u32),
    end: (u32, u32, u32),
}

impl SelectedDateRange {
    fn to_string(&self) -> String {
        let start_str = format!("{:04}-{:02}-{:02}", self.start.0, self.start.1, self.start.2);
        let end_str = format!("{:04}-{:02}-{:02}", self.end.0, self.end.1, self.end.2);
        format!("{} - {}", start_str, end_str)
    }
}


#[component]
pub fn DateTimeRangePickerCustom() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    // let selected_range: RwSignal<(String, String)> =
    //     create_rw_signal(("".to_string(), "".to_string()));
    let selected_range: RwSignal<SelectedDateRange> = create_rw_signal(SelectedDateRange {
        start: (0, 0, 0),
        end: (0, 0, 0),
    });
    let initial_month = 1;
    let initial_year = 2024;

    // // display range
    // let date_range_display = create_memo(move |_prev| {
    //     if selected_range.get().0.is_empty() && selected_range.get().1.is_empty() {
    //         "Check in - Check out".to_string()
    //     } else {
    //         format!("{} - {}", selected_range.get().0, selected_range.get().1)
    //     }
    // });

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
                <div class="absolute mt-6  min-w-[40rem] flex space-x-8 bg-white border border-gray-200 rounded-xl shadow-lg p-4 z-50">
                    <DateCells
                        month=initial_month
                        year=initial_year
                        selected_range=selected_range
                    />
                    // <div class="border border-red-800 h-full w-[1px]"></div>
                    <DateCells
                        month=if initial_month == 12 { 1 } else { initial_month + 1 }
                        year=if initial_month == 12 { initial_year + 1 } else { initial_year }
                        selected_range=selected_range
                    />
                </div>
            </Show>
        </div>
    }
}

#[component]
fn DateCells(month: u32, year: u32, selected_range: RwSignal<SelectedDateRange>) -> impl IntoView {
    let timestamp = use_timestamp();
    
    // 0 => no day is selected
    // when the given day is selected, then the corresponding day number will be set to this signal
    // let (day_selected, set_day_selected) = create_signal(0_u32);
    
    // let (is_selected, set_is_selected) = create_signal(false);
    let (in_range, set_in_range) = create_signal(false);

    let start_weekday = {
        let timestamp = timestamp.get_untracked();
        if let Some((current_year, _, _)) = timestamp_to_ymd(timestamp as u64) {
            calculate_weekday(current_year, month, 1)
        } else {
            0
        }
    };

    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
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

    // let class_signal = move || with!(|day_selected, in_range| 
    //     { 
    //     format!(
    //         "border p-2 cursor-pointer hover:bg-gray-100 hover:text-black w-10 rounded-md {} {}",
    //         if *is_selected { "bg-blue-500 text-white" } else { "" },
    //         if *in_range { "bg-gray-200" } else { "" },
    //     )});


    // let (start, end) = selected_range.get();


    view! {
        <div class="relative z-50">
            <div class="text-center font-bold mb-2">
                {format!("{} {}", month_names[(month - 1) as usize], year)}
            </div>
            <div class="grid grid-cols-7 gap-x-2 gap-y-2.5 justify-items-center">
                {weekdays
                    .iter()
                    .map(|&day| view! { <div class="font-bold">{day}</div> })
                    .collect::<Vec<_>>()}
                {(0..start_weekday).map(|_| view! { <div></div> }).collect::<Vec<_>>()}
                {(1..=days_in_month)
                    .map(|day_num| {
                        let on_click = move |_val| {
                            let date_str = (year, month, day_num);
                            let range = selected_range.get();
                            log!("Before update: start={:?}, end={:?}", range.start, range.end);
                            let new_range = if range.start == date_str {
                                SelectedDateRange {
                                    start: (0, 0, 0),
                                    end: range.end,
                                }
                            } else if range.end == date_str {
                                SelectedDateRange {
                                    start: range.start,
                                    end: (0, 0, 0),
                                }
                            } else if range.start == (0, 0, 0) {
                                SelectedDateRange {
                                    start: (year, month, day_num),
                                    end: range.end,
                                }
                            } else if range.end == (0, 0, 0) {
                                if (year, month, day_num) > range.start {
                                    SelectedDateRange {
                                        start: range.start,
                                        end: (year, month, day_num),
                                    }
                                } else {
                                    SelectedDateRange {
                                        start: (year, month, day_num),
                                        end: range.start,
                                    }
                                }
                            } else {
                                SelectedDateRange {
                                    start: (year, month, day_num),
                                    end: (0, 0, 0),
                                }
                            };
                            selected_range.set(new_range);
                            let updated_range = selected_range.get();
                            log!(
                                "After update: start={:?}, end={:?}", updated_range.start, updated_range.end
                            );
                            // set_in_range(
                            //     is_date_in_range(
                            //         &format!(
                            //             "{:04}-{:02}-{:02}",
                            //             updated_range.start.0,
                            //             updated_range.start.1,
                            //             updated_range.start.2,
                            //         ),
                            //         &format!(
                            //             "{:04}-{:02}-{:02}",
                            //             updated_range.end.0,
                            //             updated_range.end.1,
                            //             updated_range.end.2,
                            //         ),
                            //         year,
                            //         month,
                            //         day_num,
                            //     ),
                            // );
                        };
                        view! {
                            // onclick start

                            // set the css if the button is in range of start - end dates
                            // onclick end

                            <button
                            class=move || class_signal(
                                selected_range.into(),
                                day_num,
                                year,
                                month,
                                in_range.into(),
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

 
 

fn is_date_in_range(start: &str, end: &str, year: u32, month: u32, day_num: u32) -> bool {
    if start.is_empty() || end.is_empty() {
        log!("FALSE- in_range -  {:?}", format!("{:04}-{:02}-{:02}", year, month, day_num));
        false
    } else {
        let start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap();
        let end_date = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap();
        let current_date = chrono::NaiveDate::from_ymd_opt(year as i32, month, day_num).unwrap();
        log!("ELSE - in_range -  {:?}. Result: {:?}", format!("{:04}-{:02}-{:02}", year, month, day_num), current_date > start_date && current_date < end_date);
        current_date > start_date && current_date < end_date
    }
}


fn class_signal(selected_range: Signal<SelectedDateRange>, day_num: u32, year: u32, month: u32, in_range: Signal<bool>) -> String {
    let range = selected_range.get();
    format!(
        "border p-2 cursor-pointer w-10 rounded-md {} {}",
        if (year, month, day_num) == range.start || (year, month, day_num) == range.end {
            "bg-blue-500 text-white hover:bg-blue-600"
        } else {
            " hover:bg-gray-100"
        },
        if in_range.get() { "bg-gray-200" } else { "" },
    )
}



fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn timestamp_to_ymd(timestamp_ms: u64) -> Option<(u32, u32, u32)> {
    let days: u32 = (timestamp_ms / 86400000) as u32;
    let mut year = 1970;
    let mut days_remaining = days;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days_remaining >= days_in_year {
            days_remaining -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let months = [
        31,
        28 + is_leap_year(year) as u32,
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &dm in &months {
        if days_remaining >= dm {
            days_remaining -= dm;
            month += 1;
        } else {
            break;
        }
    }

    let day = days_remaining + 1;
    Some((year, month, day))
}

fn calculate_weekday(year: u32, month: u32, day: u32) -> u32 {
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 12 } else { month };
    let k = y % 100;
    let j = y / 100;
    let h = (day + 13 * (m + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    (h + 6) % 7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_weekday() {
        let test_cases = vec![
            ((2023, 10, 1), 0), // Sunday
            ((2023, 10, 2), 1), // Monday
            ((2023, 10, 3), 2), // Tuesday
            ((2023, 10, 4), 3), // Wednesday
            ((2023, 10, 5), 4), // Thursday
            ((2023, 10, 6), 5), // Friday
            ((2023, 10, 7), 6), // Saturday
            ((2024, 2, 29), 4), // Thursday (leap year)
            ((2023, 2, 28), 2), // Tuesday (non-leap year)
        ];

        for ((year, month, day), expected) in test_cases.iter() {
            assert_eq!(calculate_weekday(*year, *month, *day), *expected);
        }
    }
}
