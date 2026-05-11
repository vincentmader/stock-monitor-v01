use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Tz;

use crate::models::AssetClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exchange {
    /// NYSE / NASDAQ — Mon–Fri 09:30–16:00 ET, closed on US federal market holidays.
    Nyse,
    /// Crypto — always open, 24/7.
    Crypto,
}

impl Exchange {
    pub fn for_asset_class(asset_class: AssetClass) -> Self {
        match asset_class {
            AssetClass::Crypto => Exchange::Crypto,
            _ => Exchange::Nyse,
        }
    }
}

/// Returns `true` when the given exchange is open for trading on `date`.
pub fn is_trading_day(date: NaiveDate, exchange: Exchange) -> bool {
    match exchange {
        Exchange::Crypto => true,
        Exchange::Nyse => !is_weekend(date) && !is_nyse_holiday(date),
    }
}

/// Returns the UTC timestamp of session open for `date`, or `None` for 24/7 exchanges
/// or when the exchange is closed that day.
pub fn session_open(date: NaiveDate, exchange: Exchange) -> Option<DateTime<Utc>> {
    match exchange {
        Exchange::Crypto => None,
        Exchange::Nyse => {
            if !is_trading_day(date, Exchange::Nyse) {
                return None;
            }
            local_to_utc(date, NaiveTime::from_hms_opt(9, 30, 0)?, chrono_tz::US::Eastern)
        }
    }
}

/// Returns the UTC timestamp of session close for `date`, or `None` for 24/7 exchanges
/// or when the exchange is closed that day.
pub fn session_close(date: NaiveDate, exchange: Exchange) -> Option<DateTime<Utc>> {
    match exchange {
        Exchange::Crypto => None,
        Exchange::Nyse => {
            if !is_trading_day(date, Exchange::Nyse) {
                return None;
            }
            local_to_utc(date, NaiveTime::from_hms_opt(16, 0, 0)?, chrono_tz::US::Eastern)
        }
    }
}

/// Returns `true` if `now` falls within the opening blackout window
/// (first `blackout_mins` minutes after session open) for equity symbols.
/// Always returns `false` for crypto.
pub fn in_open_blackout(now: DateTime<Utc>, exchange: Exchange, blackout_mins: u32) -> bool {
    if exchange == Exchange::Crypto || blackout_mins == 0 {
        return false;
    }
    let date = now.date_naive();
    if let Some(open) = session_open(date, exchange) {
        now >= open && now < open + Duration::minutes(blackout_mins as i64)
    } else {
        false
    }
}

/// Returns `true` if `now` falls within the closing blackout window
/// (last `blackout_mins` minutes before session close).
pub fn in_close_blackout(now: DateTime<Utc>, exchange: Exchange, blackout_mins: u32) -> bool {
    if exchange == Exchange::Crypto || blackout_mins == 0 {
        return false;
    }
    let date = now.date_naive();
    if let Some(close) = session_close(date, exchange) {
        now > close - Duration::minutes(blackout_mins as i64) && now <= close
    } else {
        false
    }
}

// ── internals ────────────────────────────────────────────────────────────────

fn is_weekend(date: NaiveDate) -> bool {
    matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

fn is_nyse_holiday(date: NaiveDate) -> bool {
    let year = date.year();
    let month = date.month();
    let day = date.day();
    let weekday = date.weekday();

    // New Year's Day — Jan 1 (observed)
    // When Jan 1 falls on Saturday it is observed the prior Dec 31 (year - 1 for the observer).
    if is_observed(date, year, 1, 1) || is_observed(date, year + 1, 1, 1) {
        return true;
    }

    // MLK Day — 3rd Monday of January
    if month == 1 && weekday == Weekday::Mon && nth_weekday_of_month(date) == 3 {
        return true;
    }

    // Presidents' Day — 3rd Monday of February
    if month == 2 && weekday == Weekday::Mon && nth_weekday_of_month(date) == 3 {
        return true;
    }

    // Good Friday — computed per year
    if is_good_friday(date) {
        return true;
    }

    // Memorial Day — last Monday of May
    if month == 5 && weekday == Weekday::Mon {
        // Last Monday: the next Monday is already in June
        if (date + Duration::days(7)).month() == 6 {
            return true;
        }
    }

    // Juneteenth — Jun 19 (observed; added to NYSE calendar from 2022)
    if year >= 2022 && is_observed(date, year, 6, 19) {
        return true;
    }

    // Independence Day — Jul 4 (observed)
    if is_observed(date, year, 7, 4) {
        return true;
    }

    // Labor Day — 1st Monday of September
    if month == 9 && weekday == Weekday::Mon && day <= 7 {
        return true;
    }

    // Thanksgiving — 4th Thursday of November
    if month == 11 && weekday == Weekday::Thu && nth_weekday_of_month(date) == 4 {
        return true;
    }

    // Christmas — Dec 25 (observed)
    if is_observed(date, year, 12, 25) {
        return true;
    }

    false
}

/// Returns which occurrence (1st, 2nd, 3rd, …) of its weekday this date is within its month.
fn nth_weekday_of_month(date: NaiveDate) -> u32 {
    (date.day() - 1) / 7 + 1
}

/// Returns `true` if `date` is the observed NYSE holiday for a fixed date (month/day)
/// that may fall on a weekend.
/// — Saturday holiday observed on the prior Friday.
/// — Sunday holiday observed on the following Monday.
fn is_observed(date: NaiveDate, year: i32, month: u32, day: u32) -> bool {
    let Some(holiday) = NaiveDate::from_ymd_opt(year, month, day) else {
        return false;
    };
    match holiday.weekday() {
        Weekday::Sat => date == holiday - Duration::days(1),
        Weekday::Sun => date == holiday + Duration::days(1),
        _ => date == holiday,
    }
}

/// Good Friday dates through 2030, precomputed.
fn is_good_friday(date: NaiveDate) -> bool {
    const GOOD_FRIDAYS: &[(i32, u32, u32)] = &[
        (2024, 3, 29),
        (2025, 4, 18),
        (2026, 4, 3),
        (2027, 3, 26),
        (2028, 4, 14),
        (2029, 3, 30),
        (2030, 4, 19),
    ];
    GOOD_FRIDAYS
        .iter()
        .filter_map(|&(y, m, d)| NaiveDate::from_ymd_opt(y, m, d))
        .any(|gf| gf == date)
}

fn local_to_utc(date: NaiveDate, time: NaiveTime, tz: Tz) -> Option<DateTime<Utc>> {
    let naive = date.and_time(time);
    tz.from_local_datetime(&naive).single().map(|dt| dt.with_timezone(&Utc))
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use chrono::Timelike;
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn weekend_not_trading() {
        assert!(!is_trading_day(date(2024, 1, 6), Exchange::Nyse)); // Sat
        assert!(!is_trading_day(date(2024, 1, 7), Exchange::Nyse)); // Sun
    }

    #[test]
    fn weekday_is_trading() {
        assert!(is_trading_day(date(2024, 1, 8), Exchange::Nyse)); // Mon, not a holiday
    }

    #[test]
    fn christmas_not_trading() {
        assert!(!is_trading_day(date(2024, 12, 25), Exchange::Nyse)); // Wed
    }

    #[test]
    fn christmas_observed_saturday_is_friday() {
        // Christmas 2021 = Saturday → observed Friday Dec 24
        assert!(!is_trading_day(date(2021, 12, 24), Exchange::Nyse));
        assert!(!is_trading_day(date(2021, 12, 25), Exchange::Nyse)); // Saturday — not a trading day regardless
    }

    #[test]
    fn christmas_observed_sunday_is_monday() {
        // Christmas 2022 = Sunday → observed Monday Dec 26
        assert!(!is_trading_day(date(2022, 12, 26), Exchange::Nyse));
    }

    #[test]
    fn new_years_observed_saturday_is_friday() {
        // Jan 1 2022 = Saturday → observed Friday Dec 31 2021
        assert!(!is_trading_day(date(2021, 12, 31), Exchange::Nyse));
    }

    #[test]
    fn new_years_observed_sunday_is_monday() {
        // Jan 1 2023 = Sunday → observed Monday Jan 2 2023
        assert!(!is_trading_day(date(2023, 1, 2), Exchange::Nyse));
    }

    #[test]
    fn good_friday_not_trading() {
        assert!(!is_trading_day(date(2024, 3, 29), Exchange::Nyse));
        assert!(!is_trading_day(date(2025, 4, 18), Exchange::Nyse));
    }

    #[test]
    fn mlk_day_not_trading() {
        // MLK 2024 = Jan 15
        assert!(!is_trading_day(date(2024, 1, 15), Exchange::Nyse));
    }

    #[test]
    fn labor_day_not_trading() {
        // Labor Day 2024 = Sep 2
        assert!(!is_trading_day(date(2024, 9, 2), Exchange::Nyse));
    }

    #[test]
    fn thanksgiving_not_trading() {
        // Thanksgiving 2024 = Nov 28
        assert!(!is_trading_day(date(2024, 11, 28), Exchange::Nyse));
    }

    #[test]
    fn independence_day_not_trading() {
        assert!(!is_trading_day(date(2024, 7, 4), Exchange::Nyse)); // Thu
    }

    #[test]
    fn crypto_always_trades() {
        assert!(is_trading_day(date(2024, 1, 6), Exchange::Crypto)); // Sat
        assert!(is_trading_day(date(2024, 12, 25), Exchange::Crypto)); // Christmas
        assert!(is_trading_day(date(2024, 3, 29), Exchange::Crypto)); // Good Friday
    }

    #[test]
    fn session_open_returns_utc() {
        let open = session_open(date(2024, 1, 8), Exchange::Nyse).unwrap();
        // 09:30 ET = 14:30 UTC in winter (EST = UTC-5)
        assert_eq!(open.hour(), 14);
        assert_eq!(open.minute(), 30);
    }

    #[test]
    fn session_open_none_on_holiday() {
        assert!(session_open(date(2024, 12, 25), Exchange::Nyse).is_none());
    }

    #[test]
    fn session_open_none_for_crypto() {
        assert!(session_open(date(2024, 1, 8), Exchange::Crypto).is_none());
    }

    #[test]
    fn blackout_detection() {
        // NYSE open on 2024-01-08 is 14:30 UTC
        let open_utc = session_open(date(2024, 1, 8), Exchange::Nyse).unwrap();
        let one_min_after = open_utc + Duration::minutes(1);
        let thirty_one_min_after = open_utc + Duration::minutes(31);

        assert!(in_open_blackout(one_min_after, Exchange::Nyse, 30));
        assert!(!in_open_blackout(thirty_one_min_after, Exchange::Nyse, 30));
        assert!(!in_open_blackout(one_min_after, Exchange::Crypto, 30));
    }
}
