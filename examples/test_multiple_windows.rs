use std::path::Path;
use clauditor::{parser, window, display, types::EntryWithProject};

fn main() {
    println!("Testing display of single active window with multiple projects\n");
    
    // Load test data
    let path = Path::new("test_data/overlapping_active_windows.jsonl");
    let raw_entries = parser::parse_file(path).expect("Failed to parse test file");
    
    // Convert to EntryWithProject (assuming project name from test data)
    let entries: Vec<EntryWithProject> = raw_entries.into_iter()
        .map(|e| EntryWithProject {
            entry: e,
            project: "test-project".to_string(),
        })
        .collect();
    
    let window = window::group_into_single_window_with_projects(entries);
    
    if let Some(window) = &window {
        println!("Window created with {} projects\n", window.projects.len());
        println!("=== Active Window ===");
        display::display_active_window(Some(window));
        
        // Show time details
        println!("\nWindow Details:");
        println!("- Start: {}", window.start_time.format("%H:%M UTC"));
        println!("- End: {}", window.end_time.format("%H:%M UTC"));
        println!("- Last Activity: {}", window.last_activity.format("%H:%M UTC"));
        println!("- Total Tokens: {}", window.token_counts.total());
        println!("- Projects: {}", window.projects.len());
    } else {
        println!("No active window found in test data.");
    }
}