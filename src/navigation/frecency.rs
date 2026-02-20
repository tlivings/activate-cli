use chrono::{DateTime, Duration, Utc};

/// Calculate frecency score using zoxide's algorithm.
/// Higher scores = more relevant projects.
///
/// Time decay multipliers:
/// - Last hour: 4x
/// - Last day: 2x
/// - Last week: 0.5x
/// - Older: 0.25x
pub fn calculate_frecency(visit_count: u32, last_visit: DateTime<Utc>) -> f64 {
    let base_score = visit_count as f64;
    let now = Utc::now();
    let elapsed = now.signed_duration_since(last_visit);

    let multiplier = if elapsed < Duration::hours(1) {
        4.0
    } else if elapsed < Duration::hours(24) {
        2.0
    } else if elapsed < Duration::weeks(1) {
        0.5
    } else {
        0.25
    };

    base_score * multiplier
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_frecency_within_hour() {
        let now = Utc::now();
        let recent = now - Duration::minutes(30);
        let score = calculate_frecency(10, recent);
        // 10 visits * 4x (within hour) = 40.0
        assert!((score - 40.0).abs() < 0.01, "Expected 40.0, got {}", score);
    }

    #[test]
    fn test_frecency_within_day() {
        let now = Utc::now();
        let yesterday = now - Duration::hours(12);
        let score = calculate_frecency(10, yesterday);
        // 10 visits * 2x (within day) = 20.0
        assert!((score - 20.0).abs() < 0.01, "Expected 20.0, got {}", score);
    }

    #[test]
    fn test_frecency_within_week() {
        let now = Utc::now();
        let few_days_ago = now - Duration::days(3);
        let score = calculate_frecency(10, few_days_ago);
        // 10 visits * 0.5x (within week) = 5.0
        assert!((score - 5.0).abs() < 0.01, "Expected 5.0, got {}", score);
    }

    #[test]
    fn test_frecency_older_than_week() {
        let now = Utc::now();
        let weeks_ago = now - Duration::weeks(2);
        let score = calculate_frecency(10, weeks_ago);
        // 10 visits * 0.25x (older than week) = 2.5
        assert!((score - 2.5).abs() < 0.01, "Expected 2.5, got {}", score);
    }

    #[test]
    fn test_frecency_zero_visits() {
        let now = Utc::now();
        let score = calculate_frecency(0, now);
        // 0 visits * anything = 0.0
        assert!((score - 0.0).abs() < 0.01, "Expected 0.0, got {}", score);
    }

    #[test]
    fn test_frecency_high_visit_count() {
        let now = Utc::now();
        let recent = now - Duration::minutes(10);
        let score = calculate_frecency(100, recent);
        // 100 visits * 4x = 400.0
        assert!(
            (score - 400.0).abs() < 0.01,
            "Expected 400.0, got {}",
            score
        );
    }
}
