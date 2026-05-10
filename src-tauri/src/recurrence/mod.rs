use chrono::{Duration, NaiveDateTime};

#[derive(Debug, Clone, PartialEq)]
pub enum Recurrence {
    None,
    Daily,
    Weekly,
    Monthly,
}

impl Recurrence {
    pub fn from_str(s: &str) -> Self {
        match s {
            "daily" => Recurrence::Daily,
            "weekly" => Recurrence::Weekly,
            "monthly" => Recurrence::Monthly,
            _ => Recurrence::None,
        }
    }
}

/// 기준 날짜 이후의 다음 반복 날짜를 반환한다.
/// 기준 날짜보다 미래인 가장 가까운 날짜를 찾는다.
pub fn next_occurrence(
    from: NaiveDateTime,
    recurrence: &Recurrence,
    now: NaiveDateTime,
) -> Option<NaiveDateTime> {
    match recurrence {
        Recurrence::None => None,
        Recurrence::Daily => {
            let mut next = from;
            while next <= now {
                next += Duration::days(1);
            }
            Some(next)
        }
        Recurrence::Weekly => {
            let mut next = from;
            while next <= now {
                next += Duration::weeks(1);
            }
            Some(next)
        }
        Recurrence::Monthly => {
            let mut next = from;
            while next <= now {
                next = next
                    .checked_add_months(chrono::Months::new(1))
                    .unwrap_or(next + Duration::days(30));
            }
            Some(next)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn dt(y: i32, m: u32, d: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap()
    }

    #[test]
    fn test_none_returns_none() {
        let result = next_occurrence(dt(2026, 5, 1), &Recurrence::None, dt(2026, 5, 10));
        assert_eq!(result, None);
    }

    #[test]
    fn test_daily_missed_two_days() {
        // 5월 1일 일정을 5월 10일에 확인 → 다음은 5월 11일
        let result = next_occurrence(dt(2026, 5, 1), &Recurrence::Daily, dt(2026, 5, 10));
        assert_eq!(result, Some(dt(2026, 5, 11)));
    }

    #[test]
    fn test_weekly_missed() {
        // 5월 1일 주간 일정을 5월 10일에 확인 → 다음은 5월 15일
        let result = next_occurrence(dt(2026, 5, 1), &Recurrence::Weekly, dt(2026, 5, 10));
        assert_eq!(result, Some(dt(2026, 5, 15)));
    }

    #[test]
    fn test_monthly_missed() {
        // 4월 10일 월간 일정을 5월 10일에 확인 → 다음은 6월 10일
        let result = next_occurrence(dt(2026, 4, 10), &Recurrence::Monthly, dt(2026, 5, 10));
        assert_eq!(result, Some(dt(2026, 6, 10)));
    }

    #[test]
    fn test_future_occurrence_not_changed() {
        // 5월 15일 일정을 5월 10일에 확인 → 아직 미래, 5월 15일 그대로
        let result = next_occurrence(dt(2026, 5, 15), &Recurrence::Daily, dt(2026, 5, 10));
        assert_eq!(result, Some(dt(2026, 5, 15)));
    }
}
