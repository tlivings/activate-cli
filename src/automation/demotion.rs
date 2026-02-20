use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use rusqlite::params;
use std::path::PathBuf;
use std::thread;

const INACTIVE_THRESHOLD_DAYS: i64 = 14;

/// Trigger a background demotion check (fire-and-forget)
/// Safe to call on every command - runs in background thread
pub fn trigger_demotion_check(db_path: PathBuf) {
    thread::spawn(move || {
        if let Err(e) = perform_demotion_check(&db_path) {
            // Log but don't fail - this is background work
            eprintln!("Warning: demotion check failed: {}", e);
        }
    });
}

/// Perform demotion check - demotes active projects older than threshold
pub fn perform_demotion_check(db_path: &std::path::Path) -> Result<usize> {
    use rusqlite::Connection;

    let conn = Connection::open(db_path)
        .context("Failed to open database for demotion check")?;

    let threshold = Utc::now() - Duration::days(INACTIVE_THRESHOLD_DAYS);
    let threshold_ts = threshold.timestamp();
    let now_ts = Utc::now().timestamp();

    let demoted = conn.execute(
        "UPDATE projects SET state = 'inactive', updated_at = ?1
         WHERE state = 'active' AND last_touched < ?2",
        params![now_ts, threshold_ts],
    ).context("Failed to demote stale projects")?;

    Ok(demoted)
}

/// Check a single project's filesystem modification time
pub fn get_fs_mtime(path: &std::path::Path) -> Result<chrono::DateTime<Utc>> {
    use std::time::SystemTime;

    let metadata = std::fs::metadata(path)
        .context("Failed to read directory metadata")?;

    let modified = metadata.modified()
        .context("Modification time not available")?;

    let duration = modified.duration_since(SystemTime::UNIX_EPOCH)
        .context("System time before UNIX epoch")?;

    chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_demotion_threshold() {
        // 14 days = 2 weeks
        assert_eq!(INACTIVE_THRESHOLD_DAYS, 14);
    }

    #[test]
    fn test_get_fs_mtime() {
        let temp = TempDir::new().unwrap();
        let result = get_fs_mtime(temp.path());
        assert!(result.is_ok());
        // Should be recent (within last minute)
        let mtime = result.unwrap();
        let now = Utc::now();
        let diff = now.signed_duration_since(mtime);
        assert!(diff.num_seconds() < 60);
    }
}
