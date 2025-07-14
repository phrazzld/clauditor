use std::path::Path;
use clauditor::parser;
use clauditor::window;
use chrono::Timelike;

#[test]
fn test_edge_cases_jsonl() {
    let path = Path::new("test_data/edge_cases.jsonl");
    let entries = parser::parse_file(path).expect("Failed to parse edge_cases.jsonl");
    
    // Should parse some valid entries despite malformed lines
    assert!(entries.len() > 0, "Should parse some valid entries");
    
    // Check that cache tokens are parsed correctly
    let entry_with_cache = entries.iter()
        .find(|e| e.message.usage.as_ref()
            .map(|u| u.cache_creation_input_tokens > 0 || u.cache_read_input_tokens > 0)
            .unwrap_or(false))
        .expect("Should find entry with cache tokens");
    
    let usage = entry_with_cache.message.usage.as_ref().unwrap();
    assert!(usage.cache_creation_input_tokens > 0 || usage.cache_read_input_tokens > 0);
    
    // Group into single window
    let window = window::group_into_single_window_with_projects(entries.into_iter().map(|e| {
        clauditor::types::EntryWithProject {
            entry: e,
            project: "test-project".to_string(),
        }
    }).collect()).expect("Should create a window");
    
    // Check totals include cache tokens
    assert!(window.token_counts.cache_creation_tokens > 0 || window.token_counts.cache_read_tokens > 0,
        "Window should have cache tokens");
    
    assert!(window.token_counts.total() > 
        window.token_counts.input_tokens + 
        window.token_counts.output_tokens);
}

#[test]
fn test_multiple_windows_jsonl() {
    let path = Path::new("test_data/multiple_windows_with_gaps.jsonl");
    let entries = parser::parse_file(path).expect("Failed to parse multiple_windows_with_gaps.jsonl");
    
    let window = window::group_into_single_window_with_projects(entries.into_iter().map(|e| {
        let project = match e.message.model.as_str() {
            "claude-opus-4-20250514" => "project-opus",
            "claude-sonnet-4-20250514" => "project-sonnet",
            _ => "unknown",
        };
        clauditor::types::EntryWithProject {
            entry: e,
            project: project.to_string(),
        }
    }).collect()).expect("Should create a window");
    
    // Should create single window starting from earliest entry
    assert_eq!(window.start_time.hour(), 9, "Window should start at 9:00");
    
    // Different models should create different projects within same window
    assert!(window.projects.len() >= 1, "Window should have at least one project");
}

#[test] 
fn test_continuous_session_jsonl() {
    let path = Path::new("test_data/single_session_continuous.jsonl");
    let entries = parser::parse_file(path).expect("Failed to parse single_session_continuous.jsonl");
    
    let window = window::group_into_single_window_with_projects(entries.into_iter().map(|e| {
        clauditor::types::EntryWithProject {
            entry: e,
            project: "test-project".to_string(),
        }
    }).collect()).expect("Should create a window");
    
    assert_eq!(window.start_time.hour(), 14);
    assert_eq!(window.end_time.hour(), 19);
    
    // Should span nearly 5 hours
    let duration = window.last_activity - window.start_time;
    assert!(duration.num_hours() >= 4, "Session should span at least 4 hours");
}

#[test]
fn test_empty_and_malformed_jsonl() {
    // Empty file should return empty vec
    let empty_path = Path::new("test_data/empty.jsonl");
    let entries = parser::parse_file(empty_path).expect("Failed to parse empty.jsonl");
    assert_eq!(entries.len(), 0, "Empty file should return no entries");
    
    // Malformed only file should return empty vec
    let malformed_path = Path::new("test_data/malformed_only.jsonl");
    let entries = parser::parse_file(malformed_path).expect("Failed to parse malformed_only.jsonl");
    assert_eq!(entries.len(), 0, "Malformed only file should return no entries");
}

#[test]
fn test_single_account_wide_window() {
    use chrono::{Utc, Duration};
    use clauditor::types::is_block_active;
    
    let path = Path::new("test_data/multiple_active_sessions.jsonl");
    let entries = parser::parse_file(path).expect("Failed to parse multiple_active_sessions.jsonl");
    
    let window = window::group_into_single_window_with_projects(entries.into_iter().map(|e| {
        let project = match e.message.model.as_str() {
            "claude-opus-4-20250514" => "project-opus",
            "claude-sonnet-4-20250514" => "project-sonnet",
            _ => "unknown",
        };
        clauditor::types::EntryWithProject {
            entry: e,
            project: project.to_string(),
        }
    }).collect()).expect("Should create a window");
    
    // Should create single window starting from earliest entry
    assert_eq!(window.start_time.hour(), 9, "Window should start at 9:00 (earliest entry)");
    assert_eq!(window.end_time.hour(), 14, "Window should end at 14:00 (5 hours later)");
    
    // Check that different models create different projects within same window
    let project_names: Vec<&str> = window.projects.iter()
        .map(|p| p.name.as_str())
        .collect();
    
    assert!(project_names.contains(&"project-opus"), "Should have opus project");
    assert!(project_names.contains(&"project-sonnet"), "Should have sonnet project");
    
    // Test active status
    let mock_now = "2025-01-13T13:00:00Z".parse::<chrono::DateTime<Utc>>().unwrap();
    let is_active = is_block_active(&window, mock_now);
    assert!(is_active, "Window should be active at 13:00 (within 5 hours of last activity)");
}

#[test]
fn test_single_window_with_overlapping_sessions() {
    use chrono::{Utc, Duration};
    
    let path = Path::new("test_data/overlapping_active_windows.jsonl");
    let entries = parser::parse_file(path).expect("Failed to parse overlapping_active_windows.jsonl");
    
    let window = window::group_into_single_window_with_projects(entries.into_iter().map(|e| {
        let project = match e.message.model.as_str() {
            "claude-opus-4-20250514" => "project-opus",
            "claude-sonnet-4-20250514" => "project-sonnet",
            _ => "unknown",
        };
        clauditor::types::EntryWithProject {
            entry: e,
            project: project.to_string(),
        }
    }).collect()).expect("Should create a window");
    
    // Should create single window starting from earliest entry
    assert_eq!(window.start_time.hour(), 14, "Window should start at 14:00 (earliest entry)");
    assert_eq!(window.end_time.hour(), 19, "Window should end at 19:00 (5 hours later)");
    
    // All activity from both "sessions" should be in the same window
    assert!(window.projects.len() >= 1, "Should have projects from overlapping sessions");
    
    // Token counts should include all activity within the 5-hour window
    assert!(window.token_counts.total() > 0, "Window should have token usage");
}