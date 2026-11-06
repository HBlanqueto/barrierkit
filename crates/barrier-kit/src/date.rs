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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_round_trip_across_leap_years_and_the_epoch() {
        for date in [
            CivilDate::new(1970, 1, 1),
            CivilDate::new(2000, 2, 29),
            CivilDate::new(2026, 11, 6),
            CivilDate::new(1900, 3, 1),
        ] {
            assert_eq!(CivilDate::from_days(date.days()), date);
        }
        assert_eq!(CivilDate::new(1970, 1, 1).days(), 0);
    }

    #[test]
    fn the_sixth_of_november_2026_is_a_friday() {
        assert_eq!(CivilDate::new(2026, 11, 6).weekday_from_monday(), 4);
    }

    #[test]
    fn moving_a_month_keeps_the_day_inside_the_shorter_month() {
        assert_eq!(
            CivilDate::new(2026, 1, 31).add_months(1),
            CivilDate::new(2026, 2, 28)
        );
        assert_eq!(
            CivilDate::new(2024, 2, 29).shifted(DatePart::Year, 1),
            CivilDate::new(2025, 2, 28)
        );
        assert_eq!(
            CivilDate::new(2026, 12, 31).shifted(DatePart::Day, 1),
            CivilDate::new(2027, 1, 1)
        );
    }

    #[test]
    fn hours_and_minutes_wrap_without_changing_the_day() {
        let at = CivilDateTime::new(CivilDate::new(2026, 11, 6), 23, 59);
        assert_eq!(at.shifted(DatePart::Hour, 1).hour, 0);
        assert_eq!(at.shifted(DatePart::Minute, 1).minute, 0);
        assert_eq!(at.shifted(DatePart::Minute, 1).date, at.date);
    }

    #[test]
    fn typing_moves_on_when_the_part_is_full_or_no_digit_can_follow() {
        assert!(!typing_is_done(DatePart::Day, "1"));
        assert!(typing_is_done(DatePart::Day, "4"));
        assert!(typing_is_done(DatePart::Month, "2"));
        assert!(typing_is_done(DatePart::Year, "2026"));
        assert!(!typing_is_done(DatePart::Year, "202"));
        let at = CivilDateTime::new(CivilDate::new(2026, 2, 10), 9, 0);
        assert_eq!(
            at.typed(DatePart::Day, 31).date,
            CivilDate::new(2026, 2, 28)
        );
        assert_eq!(at.typed(DatePart::Hour, 30).hour, 23);
    }

    #[test]
    fn the_month_grid_starts_on_the_first_day_of_its_week() {
        let monday = CivilDate::new(2026, 11, 6).month_grid(true);
        assert_eq!(monday[0], CivilDate::new(2026, 10, 26));
        assert_eq!(monday[6], CivilDate::new(2026, 11, 1));
        let sunday = CivilDate::new(2026, 11, 6).month_grid(false);
        assert_eq!(sunday[0], CivilDate::new(2026, 11, 1));
    }
}
