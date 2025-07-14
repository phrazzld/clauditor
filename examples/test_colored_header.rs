use chrono::{Duration, Utc};
use clauditor::display::display_active_window;
use clauditor::types::{SessionBlock, ProjectUsage, TokenCounts};

fn main() {
    let now = Utc::now();
    
    // Test with single window
    println!("=== Testing single window display ===");
    let single_window = SessionBlock {
        start_time: now - Duration::hours(1),
        end_time: now + Duration::hours(4),
        last_activity: now,
        projects: vec![ProjectUsage {
            name: "test-project".to_string(),
            token_counts: TokenCounts {
                input_tokens: 10000,
                output_tokens: 5000,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
            },
            entry_count: 50,
        }],
        token_counts: TokenCounts {
            input_tokens: 10000,
            output_tokens: 5000,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
        },
        is_active: true,
    };
    display_active_window(Some(&single_window));
    
    println!("\n\n=== Testing window with multiple projects ===");
    let window_with_projects = SessionBlock {
            start_time: now - Duration::hours(1),
            end_time: now + Duration::hours(4),
            last_activity: now,
            projects: vec![
                ProjectUsage {
                    name: "project-alpha".to_string(),
                    token_counts: TokenCounts {
                        input_tokens: 25000,
                        output_tokens: 12000,
                        cache_creation_tokens: 1000,
                        cache_read_tokens: 500,
                    },
                    entry_count: 100,
                },
                ProjectUsage {
                    name: "project-beta".to_string(),
                    token_counts: TokenCounts {
                        input_tokens: 5000,
                        output_tokens: 2500,
                        cache_creation_tokens: 0,
                        cache_read_tokens: 0,
                    },
                    entry_count: 25,
                },
                ProjectUsage {
                    name: "project-gamma".to_string(),
                    token_counts: TokenCounts {
                        input_tokens: 8000,
                        output_tokens: 4000,
                        cache_creation_tokens: 500,
                        cache_read_tokens: 200,
                    },
                    entry_count: 40,
                },
            ],
            token_counts: TokenCounts {
                input_tokens: 13000,
                output_tokens: 6500,
                cache_creation_tokens: 500,
                cache_read_tokens: 200,
            },
            is_active: true,
        };
    display_active_window(Some(&window_with_projects));
    
    println!("\n\n=== Testing no active window display ===");
    display_active_window(None);
}