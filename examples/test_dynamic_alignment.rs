use chrono::{Duration, Utc};
use clauditor::display::display_active_window;
use clauditor::types::{SessionBlock, ProjectUsage, TokenCounts};

fn main() {
    let now = Utc::now();
    
    // Test with various project name lengths
    let window = SessionBlock {
        start_time: now - Duration::hours(1),
        end_time: now + Duration::hours(4),
        last_activity: now,
        projects: vec![
            ProjectUsage {
                name: "short".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 1000000,
                    output_tokens: 500000,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 50,
            },
            ProjectUsage {
                name: "medium-length-project".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 250000,
                    output_tokens: 125000,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 25,
            },
            ProjectUsage {
                name: "very-long-project-name-that-might-need-truncation-in-narrow-terminals".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 50000,
                    output_tokens: 25000,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 10,
            },
            ProjectUsage {
                name: "another-project-with-moderate-length-name".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 10000,
                    output_tokens: 5000,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 5,
            },
        ],
        token_counts: TokenCounts {
            input_tokens: 1310000,
            output_tokens: 655000,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
        },
        is_active: true,
    };
    
    println!("=== Testing dynamic token alignment ===");
    println!("Terminal width detected: {} columns", clauditor::display::get_terminal_width());
    println!();
    
    display_active_window(Some(&window));
    
    println!("\n=== Testing edge cases ===");
    println!("Notice how:");
    println!("- Token counts are right-aligned to terminal edge");
    println!("- Long project names are truncated with '...' if needed");
    println!("- All token values align vertically");
}