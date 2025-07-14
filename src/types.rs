use chrono::{DateTime, Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};

/// Token usage information from Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

/// Message information from JSONL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub role: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

/// Single JSONL entry from Claude Code session file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub timestamp: DateTime<Utc>,
    pub message: Message,
    #[serde(rename = "costUSD")]
    pub cost_usd: Option<f64>,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub version: String,
}

/// Aggregated token counts
#[derive(Debug, Clone, Default)]
pub struct TokenCounts {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
}

impl TokenCounts {
    /// Total tokens (all types combined)
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }
    
    /// Add tokens from a usage entry
    pub fn add_usage(&mut self, usage: &TokenUsage) {
        self.input_tokens += usage.input_tokens;
        self.output_tokens += usage.output_tokens;
        self.cache_creation_tokens += usage.cache_creation_input_tokens;
        self.cache_read_tokens += usage.cache_read_input_tokens;
    }
}

/// Information about a single session file
#[derive(Debug, Clone)]
pub struct SessionFile {
    pub path: String,
    pub project: String,
    pub session_id: String,
    pub last_read_position: u64,
    pub entries: Vec<UsageEntry>,
}

/// A 5-hour billing window containing usage data
#[derive(Debug, Clone)]
pub struct SessionBlock {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub projects: Vec<ProjectUsage>,
    pub token_counts: TokenCounts,
    pub is_active: bool,
}

impl SessionBlock {
    /// Calculate burn rate in tokens per minute
    pub fn burn_rate(&self) -> f64 {
        let duration = self.last_activity - self.start_time;
        let minutes = duration.num_seconds() as f64 / 60.0;
        if minutes > 0.0 {
            self.token_counts.total() as f64 / minutes
        } else {
            0.0
        }
    }
    
    /// Time remaining until window ends
    pub fn time_remaining(&self, now: DateTime<Utc>) -> Duration {
        self.end_time - now
    }
}

/// Usage data for a specific project within a session block
#[derive(Debug, Clone)]
pub struct ProjectUsage {
    pub name: String,
    pub token_counts: TokenCounts,
    pub entry_count: usize,
}

/// Floor a timestamp to the beginning of the hour (UTC)
/// 
/// This function is critical for billing window calculations. Claude Code bills
/// in 5-hour windows that start at the top of an hour. For example:
/// - 14:23:45 -> 14:00:00
/// - 14:59:59 -> 14:00:00
/// - 15:00:01 -> 15:00:00
/// 
/// This ensures consistent window boundaries regardless of when activity starts.
pub fn floor_to_hour(timestamp: DateTime<Utc>) -> DateTime<Utc> {
    timestamp
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap()
}

/// Check if a session block is currently active
/// 
/// A billing window is considered active if BOTH conditions are met:
/// 1. Last activity was less than 5 hours ago (session hasn't expired)
/// 2. Current time is before the window end time (5 hours from window start)
/// 
/// This matches Claude Code's billing model where:
/// - Sessions expire after 5 hours of inactivity
/// - Windows are fixed 5-hour periods from the floored start hour
/// 
/// Example: Window starts at 2:00 PM, ends at 7:00 PM
/// - At 6:30 PM with last activity at 6:00 PM: Active (both conditions met)
/// - At 6:30 PM with last activity at 1:00 PM: Inactive (>5 hours since activity)
/// - At 7:30 PM with recent activity: Inactive (past window end time)
pub fn is_block_active(block: &SessionBlock, now: DateTime<Utc>) -> bool {
    let five_hours = Duration::hours(5);
    let time_since_last = now - block.last_activity;
    let time_until_end = block.end_time - now;
    
    time_since_last < five_hours && time_until_end > Duration::zero()
}

/// Entry with its associated project information
#[derive(Debug, Clone)]
pub struct EntryWithProject {
    pub entry: UsageEntry,
    pub project: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_floor_to_hour() {
        // Test various timestamps
        let test_cases = vec![
            ("2025-01-13T14:00:00.000Z", "2025-01-13T14:00:00Z"),
            ("2025-01-13T14:00:01.000Z", "2025-01-13T14:00:00Z"),
            ("2025-01-13T14:59:59.999Z", "2025-01-13T14:00:00Z"),
            ("2025-01-13T14:30:45.123Z", "2025-01-13T14:00:00Z"),
            ("2025-01-13T00:00:00.000Z", "2025-01-13T00:00:00Z"), // Midnight UTC
            ("2025-01-13T23:59:59.999Z", "2025-01-13T23:00:00Z"), // Just before midnight
        ];
        
        for (input, expected) in test_cases {
            let timestamp: DateTime<Utc> = input.parse().unwrap();
            let floored = floor_to_hour(timestamp);
            let expected_time: DateTime<Utc> = expected.parse().unwrap();
            assert_eq!(floored, expected_time, "Failed for input: {}", input);
        }
    }
    
    #[test]
    fn test_is_block_active() {
        // Create a test block
        let start_time: DateTime<Utc> = "2025-01-13T14:00:00Z".parse().unwrap();
        let end_time = start_time + Duration::hours(5);
        let last_activity: DateTime<Utc> = "2025-01-13T16:30:00Z".parse().unwrap();
        
        let block = SessionBlock {
            start_time,
            end_time,
            last_activity,
            projects: vec![],
            token_counts: TokenCounts::default(),
            is_active: false,
        };
        
        // Test various "now" times
        let test_cases = vec![
            // (now_time, expected_active, description)
            ("2025-01-13T17:00:00Z", true, "30 min after last activity, well before end"),
            ("2025-01-13T18:59:59Z", true, "Just before window end, within 5h of last activity"),
            ("2025-01-13T19:00:00Z", false, "Exactly at window end"),
            ("2025-01-13T19:00:01Z", false, "Just after window end"),
            ("2025-01-13T21:29:59Z", false, "Just under 5h after last activity but past window end"),
            ("2025-01-13T21:30:01Z", false, "Just over 5h after last activity"),
            ("2025-01-13T22:00:00Z", false, "Way past both limits"),
        ];
        
        for (now_str, expected, desc) in test_cases {
            let now: DateTime<Utc> = now_str.parse().unwrap();
            let is_active = is_block_active(&block, now);
            assert_eq!(is_active, expected, "Failed for {}: {}", desc, now_str);
        }
    }
    
    #[test]
    fn test_is_block_active_edge_cases() {
        // Test when last activity is at the very end of the window
        let start_time: DateTime<Utc> = "2025-01-13T14:00:00Z".parse().unwrap();
        let end_time = start_time + Duration::hours(5);
        let last_activity = end_time - Duration::seconds(1); // 1 second before end
        
        let block = SessionBlock {
            start_time,
            end_time,
            last_activity,
            projects: vec![],
            token_counts: TokenCounts::default(),
            is_active: false,
        };
        
        // Even though within 5 hours of last activity, should not be active after window end
        let now = last_activity + Duration::minutes(1);
        assert!(!is_block_active(&block, now), "Should not be active after window end time");
        
        // Should be active just before window end
        let now = last_activity - Duration::seconds(1);
        assert!(is_block_active(&block, now), "Should be active before window end");
        
        let now = last_activity + Duration::hours(5) + Duration::seconds(1);
        assert!(!is_block_active(&block, now), "Should not be active 5h+ after last activity");
    }
}