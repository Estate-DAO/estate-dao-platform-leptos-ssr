use chrono::Datelike;
use chrono::NaiveDate;

pub fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn is_date_in_range(
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

pub fn days_in_month(month: u32, year: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => panic!("Invalid month"),
    }
}

pub fn prev_day(year: u32, month: u32, day: u32) -> (u32, u32, u32) {
    if day == 1 {
        if month == 1 {
            (year - 1, 12, 31)
        } else {
            (year, month - 1, days_in_month(month - 1, year))
        }
    } else {
        (year, month, day - 1)
    }
}

pub fn next_day(year: u32, month: u32, day: u32) -> (u32, u32, u32) {
    let days = days_in_month(month, year);
    if day == days {
        if month == 12 {
            (year + 1, 1, 1)
        } else {
            (year, month + 1, 1)
        }
    } else {
        (year, month, day + 1)
    }
}

pub fn prev_date(year: u32, month: u32) -> (u32, u32) {
    let value = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    value
}

pub fn next_date(year: u32, month: u32) -> (u32, u32) {
    let value = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    value
}

pub fn get_year_month_day(timestamp: f64) -> (u32, u32, u32) {
    let secs = (timestamp / 1000_f64).floor() as i64;
    let naive = chrono::NaiveDateTime::from_timestamp_opt(secs, 0).unwrap();
    let datetime: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(naive, chrono::Utc);
    (datetime.year() as u32, datetime.month(), datetime.day())
}

pub fn get_year_month(timestamp: f64) -> (u32, u32) {
    let secs = (timestamp / 1000_f64).floor() as i64;
    let naive = chrono::NaiveDateTime::from_timestamp_opt(secs, 0).unwrap();
    let datetime: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(naive, chrono::Utc);
    (datetime.year() as u32, datetime.month())
}
