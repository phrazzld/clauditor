use anyhow::Result;
use chrono::Utc;

use crate::scanner::SessionScanner;
use crate::window::{group_into_single_window_with_projects, is_window_active, find_active_window_period};
use crate::types::{SessionBlock, EntryWithProject};

/// Load all sessions and group them into a single account-wide billing window
pub fn load_and_group_sessions() -> Result<Option<SessionBlock>> {
    // eprintln!("[DEBUG] load_and_group_sessions: Starting");
    
    let mut scanner = SessionScanner::new();
    let sessions = scanner.load_sessions()?;
    
    // eprintln!("[DEBUG] load_and_group_sessions: Loaded {} sessions", sessions.len());
    
    // Convert sessions to entries with project info
    let mut entries_with_projects = Vec::new();
    for session in sessions {
        let entry_count = session.entries.len();
        // eprintln!("[DEBUG] load_and_group_sessions: Session {} has {} entries", session.project, entry_count);
        
        for entry in session.entries {
            entries_with_projects.push(EntryWithProject {
                entry,
                project: session.project.clone(),
            });
        }
    }
    
    // eprintln!("[DEBUG] load_and_group_sessions: Total {} entries with projects", entries_with_projects.len());
    
    // Group into single window
    let window = group_into_single_window_with_projects(entries_with_projects);
    
    match &window {
        Some(_w) => {}, // eprintln!("[DEBUG] load_and_group_sessions: Window created with {} projects", w.projects.len()),
        None => {}, // eprintln!("[DEBUG] load_and_group_sessions: No window created"),
    }
    
    Ok(window)
}

/// Load sessions incrementally and group them into a single account-wide billing window
/// 
/// This function now checks if there's an active window period and loads ALL data
/// for that window, not just incremental updates. This ensures all projects are included.
pub fn load_and_group_sessions_incremental(scanner: &mut SessionScanner) -> Result<Option<SessionBlock>> {
    // eprintln!("[DEBUG] load_and_group_sessions_incremental: Starting incremental load");
    
    // First, try incremental load to check for new activity
    let incremental_sessions = scanner.load_sessions_incremental()?;
    
    // eprintln!("[DEBUG] load_and_group_sessions_incremental: Found {} incremental sessions", incremental_sessions.len());
    
    // If no new data, check if we still have an active window from previous data
    if incremental_sessions.is_empty() {
        // eprintln!("[DEBUG] load_and_group_sessions_incremental: No new data, checking for active window via full load");
        // Load all sessions to check for active window
        return get_active_billing_window();
    }
    
    // We have new data - check if there's an active window period
    let mut all_entries = Vec::new();
    for session in &incremental_sessions {
        // eprintln!("[DEBUG] load_and_group_sessions_incremental: Session {} has {} new entries", session.project, session.entries.len());
        for entry in &session.entries {
            all_entries.push(EntryWithProject {
                entry: entry.clone(),
                project: session.project.clone(),
            });
        }
    }
    
    // eprintln!("[DEBUG] load_and_group_sessions_incremental: Total {} new entries across all sessions", all_entries.len());
    
    let now = Utc::now();
    // eprintln!("[DEBUG] load_and_group_sessions_incremental: Current time: {}", now);
    
    // Check if there's an active window based on the new entries
    if let Some(window_period) = find_active_window_period(&all_entries, now) {
        // eprintln!("[DEBUG] load_and_group_sessions_incremental: Active window detected: {} to {}, doing FULL reload", window_period.0, window_period.1);
        // Active window detected - do a FULL reload to get all projects
        return get_active_billing_window();
    }
    
    // eprintln!("[DEBUG] load_and_group_sessions_incremental: No active window found based on new entries");
    // No active window found
    Ok(None)
}

/// Get the currently active billing window (if any)
/// 
/// This loads ALL sessions and finds the currently active window.
/// Used for full reloads when we need complete window data.
pub fn get_active_billing_window() -> Result<Option<SessionBlock>> {
    // eprintln!("[DEBUG] get_active_billing_window: Starting");
    
    let window = load_and_group_sessions()?;
    
    match &window {
        Some(w) => {
            // eprintln!("[DEBUG] get_active_billing_window: Window exists, is_active: {}", w.is_active);
            let _active_check = is_window_active(w);
            // eprintln!("[DEBUG] get_active_billing_window: is_window_active() returned: {}", active_check);
        }
        None => {
            // eprintln!("[DEBUG] get_active_billing_window: No window returned from load_and_group_sessions");
        }
    }
    
    // Return the window only if it's active
    let result = window.filter(|w| is_window_active(w));
    
    match &result {
        Some(_) => {} ,// eprintln!("[DEBUG] get_active_billing_window: Returning active window"),
        None => {}, // eprintln!("[DEBUG] get_active_billing_window: Returning None (no active window)"),
    }
    
    Ok(result)
}

/// Get summary statistics for the active window
pub struct ActiveWindowSummary {
    pub has_active_window: bool,
    pub total_tokens: u64,
    pub burn_rate: f64,
}

impl ActiveWindowSummary {
    pub fn from_window(window: Option<&SessionBlock>) -> Self {
        match window {
            Some(w) => Self {
                has_active_window: true,
                total_tokens: w.token_counts.total(),
                burn_rate: w.burn_rate(),
            },
            None => Self {
                has_active_window: false,
                total_tokens: 0,
                burn_rate: 0.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_entry_with_project() {
        // This test ensures our wrapper type works correctly
        let entry = crate::types::UsageEntry {
            timestamp: Utc::now(),
            message: crate::types::Message {
                id: "test".to_string(),
                msg_type: "message".to_string(),
                role: "assistant".to_string(),
                model: "claude-opus-4-20250514".to_string(),
                usage: Some(crate::types::TokenUsage {
                    input_tokens: 100,
                    output_tokens: 50,
                    cache_creation_input_tokens: 0,
                    cache_read_input_tokens: 0,
                }),
            },
            cost_usd: None,
            request_id: "req_test".to_string(),
            version: "1.0.51".to_string(),
        };
        
        let entry_with_project = EntryWithProject {
            entry: entry.clone(),
            project: "test-project".to_string(),
        };
        
        assert_eq!(entry_with_project.project, "test-project");
        assert_eq!(entry_with_project.entry.message.id, "test");
    }
}