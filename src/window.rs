use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

use crate::types::{
    UsageEntry, SessionBlock, ProjectUsage, TokenCounts, 
    floor_to_hour, is_block_active, EntryWithProject
};

const SESSION_DURATION_HOURS: i64 = 5;

/// Group usage entries into a single account-wide billing window
/// 
/// This implements the core billing window algorithm based on Claude Code's actual model:
/// - ONE active window at a time for entire account
/// - Window starts when you first use Claude Code (any project)
/// - Window lasts exactly 5 hours from start time
/// - ALL usage across ALL projects during those 5 hours counts
/// - New activity after window ends starts a new window
/// 
/// Algorithm:
/// 1. Find the earliest entry across all projects
/// 2. Create a single 5-hour window starting from floor_to_hour of that entry
/// 3. Include ALL entries within that 5-hour period
/// 4. Mark window as active if last activity was within 5 hours
#[allow(dead_code)]
pub fn group_into_single_window(entries: Vec<UsageEntry>) -> Option<SessionBlock> {
    if entries.is_empty() {
        return None;
    }
    
    // Sort entries by timestamp to find the earliest one
    let mut sorted_entries = entries;
    sorted_entries.sort_by_key(|e| e.timestamp);
    
    // Find the earliest timestamp - this starts our single window
    let earliest_entry = sorted_entries.first()?;
    let window_start = floor_to_hour(earliest_entry.timestamp);
    let window_end = window_start + Duration::hours(SESSION_DURATION_HOURS);
    
    // Filter entries that fall within this 5-hour window
    let window_entries: Vec<UsageEntry> = sorted_entries
        .into_iter()
        .filter(|e| e.timestamp >= window_start && e.timestamp < window_end)
        .collect();
    
    // Create the single window with all entries
    let mut window = create_window(window_start, &window_entries)?;
    
    // Update active status
    let now = Utc::now();
    window.is_active = is_block_active(&window, now);
    
    Some(window)
}

/// Create a SessionBlock from a group of entries
/// 
/// Builds a complete billing window with:
/// - Start time: The floored hour when the window began
/// - End time: Exactly 5 hours after start time
/// - Last activity: Timestamp of the most recent entry
/// - Token counts: Aggregated from all entries in the window
/// - Projects: Usage broken down by project
#[allow(dead_code)]
fn create_window(start_time: DateTime<Utc>, entries: &[UsageEntry]) -> Option<SessionBlock> {
    if entries.is_empty() {
        return None;
    }
    
    let end_time = start_time + Duration::hours(SESSION_DURATION_HOURS);
    let last_activity = entries.last()?.timestamp;
    
    // Group entries by project
    let mut project_map: HashMap<String, ProjectUsage> = HashMap::new();
    let mut total_tokens = TokenCounts::default();
    
    for entry in entries {
        // Extract project name from request ID or use "unknown"
        // In a real implementation, this would parse from file path
        let project_name = extract_project_name(entry);
        
        if let Some(usage) = &entry.message.usage {
            total_tokens.add_usage(usage);
            
            let project = project_map.entry(project_name.clone())
                .or_insert_with(|| ProjectUsage {
                    name: project_name,
                    token_counts: TokenCounts::default(),
                    entry_count: 0,
                });
            
            project.token_counts.add_usage(usage);
            project.entry_count += 1;
        }
    }
    
    let projects: Vec<ProjectUsage> = project_map.into_values().collect();
    
    Some(SessionBlock {
        start_time,
        end_time,
        last_activity,
        projects,
        token_counts: total_tokens,
        is_active: false, // Will be updated by caller
    })
}

/// Extract project name from entry (placeholder implementation)
#[allow(dead_code)]
fn extract_project_name(entry: &UsageEntry) -> String {
    // In real implementation, this would be parsed from the file path
    // For now, use model name as a placeholder to differentiate
    match entry.message.model.as_str() {
        "claude-opus-4-20250514" => "project-opus".to_string(),
        "claude-sonnet-4-20250514" => "project-sonnet".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Check if the single window is currently active
pub fn is_window_active(window: &SessionBlock) -> bool {
    window.is_active
}

/// Find the currently active billing window period based on recent activity
/// 
/// Returns Some((start_time, end_time)) if there's an active window, None otherwise.
/// This uses chronological processing to correctly identify which window entries belong to.
pub fn find_active_window_period(entries: &[EntryWithProject], now: DateTime<Utc>) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    if entries.is_empty() {
        return None;
    }
    
    // Look back 15 hours to catch windows that might have started earlier
    let fifteen_hours_ago = now - Duration::hours(SESSION_DURATION_HOURS * 3);
    
    // Get entries from the last 15 hours and sort chronologically (oldest first)
    let mut recent_entries: Vec<&EntryWithProject> = entries
        .iter()
        .filter(|e| e.entry.timestamp >= fifteen_hours_ago)
        .collect();
    
    if recent_entries.is_empty() {
        return None;
    }
    
    // Sort by timestamp (oldest first) - this is KEY for correct window assignment
    recent_entries.sort_by_key(|e| e.entry.timestamp);
    
    
    // Process entries chronologically to find windows
    let mut windows: Vec<(DateTime<Utc>, DateTime<Utc>, DateTime<Utc>)> = Vec::new();
    let mut current_window_start: Option<DateTime<Utc>> = None;
    let mut last_activity: Option<DateTime<Utc>> = None;
    
    for entry in recent_entries {
        let entry_time = entry.entry.timestamp;
        
        if let Some(window_start) = current_window_start {
            let time_since_window_start = entry_time - window_start;
            
            // Check if this entry belongs to the current window
            if time_since_window_start < Duration::hours(SESSION_DURATION_HOURS) {
                // Update last activity in current window
                last_activity = Some(entry_time);
            } else {
                // Entry is beyond current window - save current window and start new one
                if let Some(last_act) = last_activity {
                    let window_end = window_start + Duration::hours(SESSION_DURATION_HOURS);
                    windows.push((window_start, window_end, last_act));
                }
                
                // Start new window
                current_window_start = Some(floor_to_hour(entry_time));
                last_activity = Some(entry_time);
            }
        } else {
            // First entry - start new window
            current_window_start = Some(floor_to_hour(entry_time));
            last_activity = Some(entry_time);
        }
    }
    
    // Don't forget the last window
    if let (Some(window_start), Some(last_act)) = (current_window_start, last_activity) {
        let window_end = window_start + Duration::hours(SESSION_DURATION_HOURS);
        windows.push((window_start, window_end, last_act));
    }
    
    
    // Find the active window: has recent activity and hasn't ended
    let five_hours_ago = now - Duration::hours(SESSION_DURATION_HOURS);
    
    for (start, end, last_activity) in windows.iter().rev() {
        // Window is active if:
        // 1. Last activity was within 5 hours
        // 2. Window end time hasn't passed
        
        if *last_activity >= five_hours_ago && now < *end {
            return Some((*start, *end));
        }
    }
    
    None
}

/// Group usage entries with project info into a single account-wide billing window
/// 
/// This is the production version that preserves project information from file paths.
/// It implements the single account-wide window model where:
/// - Only ONE billing window exists at a time across the entire account
/// - Multiple projects can contribute usage within the same window
/// - Token usage is correctly attributed to each project
/// - The window is based on RECENT activity (within last 5 hours)
pub fn group_into_single_window_with_projects(entries: Vec<EntryWithProject>) -> Option<SessionBlock> {
    group_into_single_window_with_projects_at_time(entries, Utc::now())
}

/// Group usage entries with project info into a single account-wide billing window at a specific time
/// 
/// This version accepts a "now" parameter for testing with historical data.
pub fn group_into_single_window_with_projects_at_time(
    entries: Vec<EntryWithProject>,
    now: DateTime<Utc>
) -> Option<SessionBlock> {
    if entries.is_empty() {
        return None;
    }
    
    // Find the active window period based on recent activity
    let window_period = find_active_window_period(&entries, now);
    
    match window_period {
        None => {
            return None;
        }
        Some((window_start, window_end)) => {
            // Filter entries that fall within the active window
            let window_entries: Vec<EntryWithProject> = entries
                .into_iter()
                .filter(|e| e.entry.timestamp >= window_start && e.entry.timestamp < window_end)
                .collect();
            
            // Create the single window with all entries in the active period
            let mut window = create_window_with_projects(window_start, &window_entries)?;
            
            // Update active status
            window.is_active = is_block_active(&window, now);
            
            Some(window)
        }
    }
}

/// Create a SessionBlock from entries with project info
fn create_window_with_projects(start_time: DateTime<Utc>, entries: &[EntryWithProject]) -> Option<SessionBlock> {
    if entries.is_empty() {
        return None;
    }
    
    let end_time = start_time + Duration::hours(SESSION_DURATION_HOURS);
    let last_activity = entries.last()?.entry.timestamp;
    
    // Group entries by project
    let mut project_map: HashMap<String, ProjectUsage> = HashMap::new();
    let mut total_tokens = TokenCounts::default();
    
    for entry_with_project in entries {
        let project_name = &entry_with_project.project;
        
        if let Some(usage) = &entry_with_project.entry.message.usage {
            total_tokens.add_usage(usage);
            
            let project = project_map.entry(project_name.clone())
                .or_insert_with(|| ProjectUsage {
                    name: project_name.clone(),
                    token_counts: TokenCounts::default(),
                    entry_count: 0,
                });
            
            project.token_counts.add_usage(usage);
            project.entry_count += 1;
        }
    }
    
    let projects: Vec<ProjectUsage> = project_map.into_values().collect();
    
    Some(SessionBlock {
        start_time,
        end_time,
        last_activity,
        projects,
        token_counts: total_tokens,
        is_active: false, // Will be updated by caller
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, TokenUsage};
    
    fn create_test_entry(timestamp: &str, input_tokens: u64, output_tokens: u64) -> UsageEntry {
        UsageEntry {
            timestamp: timestamp.parse().unwrap(),
            message: Message {
                id: format!("msg_{}", timestamp),
                msg_type: "message".to_string(),
                role: "assistant".to_string(),
                model: "claude-opus-4-20250514".to_string(),
                usage: Some(TokenUsage {
                    input_tokens,
                    output_tokens,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 0,
                }),
            },
            cost_usd: None,
            request_id: format!("req_{}", timestamp),
            version: "1.0.51".to_string(),
        }
    }
    
    #[test]
    fn test_single_window() {
        let entries = vec![
            create_test_entry("2025-01-12T14:00:00Z", 100, 50),
            create_test_entry("2025-01-12T14:30:00Z", 200, 100),
            create_test_entry("2025-01-12T15:00:00Z", 150, 75),
        ];
        
        let window = group_into_single_window(entries).unwrap();
        assert_eq!(window.start_time, "2025-01-12T14:00:00Z".parse::<DateTime<Utc>>().unwrap());
        assert_eq!(window.end_time, "2025-01-12T19:00:00Z".parse::<DateTime<Utc>>().unwrap());
        assert_eq!(window.token_counts.total(), 675); // 450 input + 225 output
    }
    
    #[test]
    fn test_single_window_with_gap() {
        let entries = vec![
            // All within same 5-hour window: 14:00 - 19:00
            create_test_entry("2025-01-12T14:00:00Z", 100, 50),
            create_test_entry("2025-01-12T14:30:00Z", 200, 100),
            // Gap > 5 hours (but this entry is outside window, so excluded)
            create_test_entry("2025-01-12T20:00:00Z", 150, 75),
            create_test_entry("2025-01-12T20:30:00Z", 100, 50),
        ];
        
        let window = group_into_single_window(entries).unwrap();
        
        // Single window starting from earliest activity
        assert_eq!(window.start_time, "2025-01-12T14:00:00Z".parse::<DateTime<Utc>>().unwrap());
        // Only includes entries within the 5-hour window
        assert_eq!(window.token_counts.total(), 450); // Only first two entries
    }
    
    #[test]
    fn test_window_boundary() {
        let entries = vec![
            // Window: 14:00 - 19:00
            create_test_entry("2025-01-12T14:00:00Z", 100, 50),
            create_test_entry("2025-01-12T18:59:00Z", 200, 100), // Still in window
            // This entry is outside the 5-hour window, so excluded
            create_test_entry("2025-01-12T19:01:00Z", 150, 75),
        ];
        
        let window = group_into_single_window(entries).unwrap();
        
        assert_eq!(window.last_activity, "2025-01-12T18:59:00Z".parse::<DateTime<Utc>>().unwrap());
        assert_eq!(window.start_time, "2025-01-12T14:00:00Z".parse::<DateTime<Utc>>().unwrap());
        // Only includes entries within the window boundary
        assert_eq!(window.token_counts.total(), 450);
    }
    
    #[test]
    fn test_floor_to_hour_behavior() {
        let entries = vec![
            create_test_entry("2025-01-12T14:23:45Z", 100, 50),
            create_test_entry("2025-01-12T14:45:00Z", 200, 100),
        ];
        
        let window = group_into_single_window(entries).unwrap();
        
        // Window should start at 14:00, not 14:23
        assert_eq!(window.start_time, "2025-01-12T14:00:00Z".parse::<DateTime<Utc>>().unwrap());
    }
    
    #[test]
    fn test_empty_entries() {
        let entries = vec![];
        let window = group_into_single_window(entries);
        assert!(window.is_none());
    }
    
    #[test]
    fn test_midnight_utc_crossing() {
        let entries = vec![
            // Window spans midnight UTC
            create_test_entry("2025-01-12T22:00:00Z", 100, 50),
            create_test_entry("2025-01-12T23:30:00Z", 200, 100),
            create_test_entry("2025-01-13T00:30:00Z", 150, 75),
            create_test_entry("2025-01-13T01:00:00Z", 100, 50),
        ];
        
        let window = group_into_single_window(entries).unwrap();
        
        assert_eq!(window.start_time, "2025-01-12T22:00:00Z".parse::<DateTime<Utc>>().unwrap());
        assert_eq!(window.end_time, "2025-01-13T03:00:00Z".parse::<DateTime<Utc>>().unwrap());
        
        // Calculate expected total: (100+200+150+100) input + (50+100+75+50) output
        let expected_total = 550 + 275; // 825
        assert_eq!(window.token_counts.total(), expected_total);
    }
    
    #[test]
    fn test_different_models_create_projects() {
        let mut entries = vec![];
        
        // Opus model entries
        entries.push(UsageEntry {
            timestamp: "2025-01-12T14:00:00Z".parse().unwrap(),
            message: Message {
                id: "msg_1".to_string(),
                msg_type: "message".to_string(),
                role: "assistant".to_string(),
                model: "claude-opus-4-20250514".to_string(),
                usage: Some(TokenUsage {
                    input_tokens: 100,
                    output_tokens: 50,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 0,
                }),
            },
            cost_usd: None,
            request_id: "req_1".to_string(),
            version: "1.0.51".to_string(),
        });
        
        // Sonnet model entries
        entries.push(UsageEntry {
            timestamp: "2025-01-12T14:15:00Z".parse().unwrap(),
            message: Message {
                id: "msg_2".to_string(),
                msg_type: "message".to_string(),
                role: "assistant".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                usage: Some(TokenUsage {
                    input_tokens: 200,
                    output_tokens: 100,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 0,
                }),
            },
            cost_usd: None,
            request_id: "req_2".to_string(),
            version: "1.0.51".to_string(),
        });
        
        let window = group_into_single_window(entries).unwrap();
        assert_eq!(window.projects.len(), 2);
        
        // Check that projects are correctly grouped
        let opus_project = window.projects.iter().find(|p| p.name == "project-opus");
        let sonnet_project = window.projects.iter().find(|p| p.name == "project-sonnet");
        
        assert!(opus_project.is_some());
        assert!(sonnet_project.is_some());
        
        assert_eq!(opus_project.unwrap().token_counts.total(), 150);
        assert_eq!(sonnet_project.unwrap().token_counts.total(), 300);
    }
    
    #[test]
    fn test_cache_tokens_included() {
        let mut entry = create_test_entry("2025-01-12T14:00:00Z", 100, 50);
        
        // Add cache tokens
        if let Some(usage) = &mut entry.message.usage {
            usage.cache_creation_input_tokens = 1000;
            usage.cache_read_input_tokens = 500;
        }
        
        let window = group_into_single_window(vec![entry]).unwrap();
        
        assert_eq!(window.token_counts.input_tokens, 100);
        assert_eq!(window.token_counts.output_tokens, 50);
        assert_eq!(window.token_counts.cache_creation_tokens, 1000);
        assert_eq!(window.token_counts.cache_read_tokens, 500);
        assert_eq!(window.token_counts.total(), 1650);
    }
    
    #[test]
    fn test_exact_5_hour_boundary() {
        let entries = vec![
            create_test_entry("2025-01-12T14:00:00Z", 100, 50),
            create_test_entry("2025-01-12T18:59:59Z", 200, 100), // Just under 5 hours
            create_test_entry("2025-01-12T19:00:01Z", 150, 75), // Just over 5 hours
        ];
        
        let window = group_into_single_window(entries).unwrap();
        
        // Only includes entries within the 5-hour window
        assert_eq!(window.token_counts.total(), 450);
        assert_eq!(window.end_time, "2025-01-12T19:00:00Z".parse::<DateTime<Utc>>().unwrap());
    }
    
    #[test]
    fn test_window_assignment_chronological() {
        // Test the specific bug: activity at 7:15 PM should belong to 6:00 PM window
        let now = "2025-01-14T19:30:00Z".parse::<DateTime<Utc>>().unwrap(); // 7:30 PM
        
        let entries = vec![
            EntryWithProject {
                entry: create_test_entry("2025-01-14T18:00:00Z", 100, 50), // 6:00 PM
                project: "test-project".to_string(),
            },
            EntryWithProject {
                entry: create_test_entry("2025-01-14T19:15:00Z", 200, 100), // 7:15 PM
                project: "test-project".to_string(),
            },
        ];
        
        let window_period = find_active_window_period(&entries, now);
        assert!(window_period.is_some());
        
        let (start, end) = window_period.unwrap();
        // Window should start at 6:00 PM (18:00), not 7:00 PM (19:00)
        assert_eq!(start, "2025-01-14T18:00:00Z".parse::<DateTime<Utc>>().unwrap());
        assert_eq!(end, "2025-01-14T23:00:00Z".parse::<DateTime<Utc>>().unwrap());
    }
    
    #[test]
    fn test_multiple_windows_chronological() {
        // Test that multiple windows are correctly identified
        let now = "2025-01-14T20:30:00Z".parse::<DateTime<Utc>>().unwrap(); // 8:30 PM
        
        let entries = vec![
            // First window: 10:00 AM - 3:00 PM
            EntryWithProject {
                entry: create_test_entry("2025-01-14T10:30:00Z", 100, 50),
                project: "project1".to_string(),
            },
            EntryWithProject {
                entry: create_test_entry("2025-01-14T14:00:00Z", 200, 100),
                project: "project1".to_string(),
            },
            // Gap > 5 hours
            // Second window: 8:00 PM - 1:00 AM (active)
            EntryWithProject {
                entry: create_test_entry("2025-01-14T20:15:00Z", 300, 150),
                project: "project2".to_string(),
            },
        ];
        
        let window_period = find_active_window_period(&entries, now);
        assert!(window_period.is_some());
        
        let (start, end) = window_period.unwrap();
        // Should return the second (active) window
        assert_eq!(start, "2025-01-14T20:00:00Z".parse::<DateTime<Utc>>().unwrap());
        assert_eq!(end, "2025-01-15T01:00:00Z".parse::<DateTime<Utc>>().unwrap());
    }
    
    #[test]
    fn test_active_status_calculation() {
        // Create an entry that would make an active window
        let now = Utc::now();
        let recent_time = now - Duration::hours(2);
        
        let mut entry = create_test_entry("2025-01-12T14:00:00Z", 100, 50);
        entry.timestamp = floor_to_hour(recent_time);
        
        // Add another entry within the last hour
        let mut entry2 = create_test_entry("2025-01-12T14:30:00Z", 200, 100);
        entry2.timestamp = now - Duration::minutes(30);
        
        let window = group_into_single_window(vec![entry, entry2]).unwrap();
        
        // Window should be active
        assert!(window.is_active, "Window should be active with recent activity");
    }
    
    #[test]
    fn test_entries_without_usage_skipped() {
        let mut entry1 = create_test_entry("2025-01-12T14:00:00Z", 100, 50);
        let mut entry2 = create_test_entry("2025-01-12T14:30:00Z", 0, 0);
        entry2.message.usage = None; // No usage data
        let entry3 = create_test_entry("2025-01-12T15:00:00Z", 200, 100);
        
        let window = group_into_single_window(vec![entry1, entry2, entry3]).unwrap();
        
        // Only entries with usage should contribute to totals
        assert_eq!(window.token_counts.total(), 450);
    }
}