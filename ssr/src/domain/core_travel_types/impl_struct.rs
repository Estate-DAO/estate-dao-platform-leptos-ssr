use std::collections::HashMap;

use crate::domain::{
    core_travel_types::DomainSelectedDateRange, DomainHotelAfterSearch, DomainHotelListAfterSearch,
    DomainHotelSearchCriteria,
};

impl DomainSelectedDateRange {
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
            return DomainSelectedDateRange {
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
            chrono::NaiveDate::from_ymd_opt(year as i32, month, day)
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

impl DomainHotelListAfterSearch {
    pub fn get_results_token_map(&self) -> HashMap<String, String> {
        let mut hotel_map = HashMap::new();

        for hotel in &self.hotel_results {
            hotel_map.insert(hotel.hotel_code.clone(), hotel.result_token.clone());
        }

        hotel_map
    }

    pub fn hotel_list(&self) -> Vec<DomainHotelAfterSearch> {
        self.hotel_results.clone()
    }
}
