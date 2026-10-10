use std::ops::Sub;

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    julian: u32,
    year: u32,
    month: u32,
    day: u32,
}

impl Date {
    /// Creates a Gregorian date and computes its Julian day number.
    ///
    /// # Errors
    ///
    /// Returns an error if the date is invalid, the year is zero, or the Julian day
    /// number does not fit in a `u32`.
    pub fn new(year: u32, month: u32, day: u32) -> Result<Self, DateError> {
        if year == 0 {
            return Err(DateError::InvalidYear);
        }
        if !(1..=12).contains(&month) {
            return Err(DateError::InvalidMonth);
        }

        let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days_in_month = match month {
            2 if leap_year => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        if !(1..=days_in_month).contains(&day) {
            return Err(DateError::InvalidDay);
        }

        // Use wider intermediates so large years cannot overflow silently.
        let a = (14 - u64::from(month)) / 12;
        let y = u64::from(year) + 4800 - a;
        let m = u64::from(month) + 12 * a - 3;
        let julian =
            u64::from(day) + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
        let julian = u32::try_from(julian).map_err(|_| DateError::JulianDayOverflow)?;

        Ok(Self {
            julian,
            year,
            month,
            day,
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl Sub for Date {
    type Output = i32;
    fn sub(self, rhs: Self) -> Self::Output {
        i32::try_from(self.julian).unwrap() - i32::try_from(rhs.julian).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{Date, DateError};

    #[test]
    fn computes_known_julian_day_numbers() {
        for (year, month, day, expected) in [
            (1, 1, 1, 1_721_426),
            (1970, 1, 1, 2_440_588),
            (2000, 1, 1, 2_451_545),
            (2000, 2, 29, 2_451_604),
            (2000, 3, 1, 2_451_605),
            (1900, 2, 28, 2_415_079),
            (1900, 3, 1, 2_415_080),
        ] {
            let date = Date::new(year, month, day).unwrap();
            assert_eq!(date.julian, expected);
            assert_eq!((date.year, date.month, date.day), (year, month, day));
        }
    }

    #[test]
    fn rejects_invalid_dates() {
        for (year, month, day, expected) in [
            (0, 1, 1, DateError::InvalidYear),
            (2024, 0, 1, DateError::InvalidMonth),
            (2024, 13, 1, DateError::InvalidMonth),
            (2024, 1, 0, DateError::InvalidDay),
            (2024, 1, 32, DateError::InvalidDay),
            (2024, 4, 31, DateError::InvalidDay),
            (2023, 2, 29, DateError::InvalidDay),
            (1900, 2, 29, DateError::InvalidDay),
            (2024, 2, 30, DateError::InvalidDay),
        ] {
            assert_eq!(Date::new(year, month, day).unwrap_err(), expected);
        }
    }

    #[test]
    fn rejects_julian_day_overflow() {
        assert_eq!(
            Date::new(u32::MAX, 1, 1).unwrap_err(),
            DateError::JulianDayOverflow
        );
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateError {
    InvalidYear,
    InvalidMonth,
    InvalidDay,
    JulianDayOverflow,
}

impl std::fmt::Display for DateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidYear => "year must be greater than zero",
            Self::InvalidMonth => "month must be between 1 and 12",
            Self::InvalidDay => "day is out of range for the given month and year",
            Self::JulianDayOverflow => "Julian day number exceeds u32 range",
        };
        f.write_str(message)
    }
}

impl std::error::Error for DateError {}
