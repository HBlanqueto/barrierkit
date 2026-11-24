use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CivilDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatePart {
    Day,
    Month,
    Year,
    Hour,
    Minute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CivilDateTime {
    pub date: CivilDate,
    pub hour: u32,
    pub minute: u32,
}

impl DatePart {
    pub fn digits(self) -> usize {
        match self {
            Self::Year => 4,
            _ => 2,
        }
    }
}

impl CivilDateTime {
    pub fn new(date: CivilDate, hour: u32, minute: u32) -> Self {
        Self {
            date,
            hour: hour.min(23),
            minute: minute.min(59),
        }
    }

    pub fn shifted(self, part: DatePart, delta: i32) -> Self {
        let wrap = |value: u32, len: i32| (value as i32 + delta).rem_euclid(len) as u32;
        match part {
            DatePart::Hour => Self {
                hour: wrap(self.hour, 24),
                ..self
            },
            DatePart::Minute => Self {
                minute: wrap(self.minute, 60),
                ..self
            },
            _ => Self {
                date: self.date.shifted(part, delta),
                ..self
            },
        }
    }

    pub fn typed(self, part: DatePart, value: u32) -> Self {
        let date = self.date;
        match part {
            DatePart::Day => Self {
                date: CivilDate::new(date.year, date.month, value),
                ..self
            },
            DatePart::Month => Self {
                date: CivilDate::new(date.year, value, date.day),
                ..self
            },
            DatePart::Year => Self {
                date: CivilDate::new(value as i32, date.month, date.day),
                ..self
            },
            DatePart::Hour => Self::new(date, value, self.minute),
            DatePart::Minute => Self::new(date, self.hour, value),
        }
    }
}

pub fn typing_is_done(part: DatePart, typed: &str) -> bool {
    let largest = match part {
        DatePart::Day => 31,
        DatePart::Month => 12,
        DatePart::Hour => 23,
        DatePart::Minute => 59,
        DatePart::Year => 9999,
    };
    typed.len() >= part.digits() || typed.parse::<u32>().is_ok_and(|value| value * 10 > largest)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn utc_days_since_epoch() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (now.as_secs() / 86_400) as i64
}

impl CivilDate {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        let month = month.clamp(1, 12);
        Self {
            year,
            month,
            day: day.clamp(1, days_in_month(year, month)),
        }
    }

    // Howard Hinnant's days_from_civil: https://howardhinnant.github.io/date_algorithms.html
    fn days(self) -> i64 {
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let month = i64::from(self.month);
        let day_of_year =
            (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    fn from_days(days: i64) -> Self {
        let days = days + 719_468;
        let era = days.div_euclid(146_097);
        let day_of_era = days - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;
        let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
        let month = if shifted_month < 10 {
            shifted_month + 3
        } else {
            shifted_month - 9
        } as u32;
        let year = (year_of_era + era * 400 + i64::from(month <= 2)) as i32;
        Self { year, month, day }
    }

    pub fn today() -> Self {
        ctappearance::local_date().map_or_else(
            || Self::from_days(utc_days_since_epoch()),
            |(year, month, day)| Self::new(year, month, day),
        )
    }

    pub fn weekday_from_monday(self) -> u32 {
        (self.days() + 3).rem_euclid(7) as u32
    }

    pub fn add_days(self, days: i64) -> Self {
        Self::from_days(self.days() + days)
    }

    pub fn add_months(self, months: i32) -> Self {
        let index = self.year * 12 + self.month as i32 - 1 + months;
        Self::new(
            index.div_euclid(12),
            index.rem_euclid(12) as u32 + 1,
            self.day,
        )
    }

    pub fn shifted(self, part: DatePart, delta: i32) -> Self {
        match part {
            DatePart::Day => self.add_days(i64::from(delta)),
            DatePart::Month => self.add_months(delta),
            DatePart::Year => self.add_months(delta * 12),
            DatePart::Hour | DatePart::Minute => self,
        }
    }

    pub fn month_grid(self, week_starts_on_monday: bool) -> [CivilDate; 42] {
        let first = Self::new(self.year, self.month, 1);
        let offset = if week_starts_on_monday {
            first.weekday_from_monday()
        } else {
            (first.weekday_from_monday() + 1) % 7
        };
        std::array::from_fn(|cell| first.add_days(cell as i64 - i64::from(offset)))
    }
}
