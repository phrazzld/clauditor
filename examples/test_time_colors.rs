use chrono::{Duration, Utc};
use clauditor::display::display_active_window;
use clauditor::types::{SessionBlock, ProjectUsage, TokenCounts};

fn main() {
    let now = Utc::now();
    
    // Test with various time remaining scenarios
    
    println!("=== Testing time remaining color coding ===");
    println!("Notice the color coding:");
    println!("- Red: < 30 minutes remaining (urgent!)");
    println!("- Yellow: < 1 hour remaining (warning)");
    println!("- Green: > 2 hours remaining (plenty of time)");
    println!("- No color: 1-2 hours remaining");
    println!();
    
    // Window ending in 20 minutes (red)
    println!("\n--- Window ending in 20 minutes (should be RED) ---");
    let window_urgent = SessionBlock {
            start_time: now - Duration::hours(4) - Duration::minutes(40),
            end_time: now + Duration::minutes(20),
            last_activity: now - Duration::minutes(5),
            projects: vec![ProjectUsage {
                name: "urgent-project".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 450000,
                    output_tokens: 225000,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 100,
            }],
            token_counts: TokenCounts {
                input_tokens: 450000,
                output_tokens: 225000,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
            },
            is_active: true,
        };
    display_active_window(Some(&window_urgent));
    
    // Window ending in 45 minutes (yellow)
    println!("\n--- Window ending in 45 minutes (should be YELLOW) ---");
    let window_warning = SessionBlock {
            start_time: now - Duration::hours(4) - Duration::minutes(15),
            end_time: now + Duration::minutes(45),
            last_activity: now - Duration::minutes(10),
            projects: vec![ProjectUsage {
                name: "warning-project".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 300000,
                    output_tokens: 150000,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 75,
            }],
            token_counts: TokenCounts {
                input_tokens: 300000,
                output_tokens: 150000,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
            },
            is_active: true,
        };
    display_active_window(Some(&window_warning));
    
    // Window ending in 3h 30m (green)
    println!("\n--- Window ending in 3h 30m (should be GREEN) ---");
    let window_comfortable = SessionBlock {
            start_time: now - Duration::hours(1) - Duration::minutes(30),
            end_time: now + Duration::hours(3) + Duration::minutes(30),
            last_activity: now,
            projects: vec![ProjectUsage {
                name: "comfortable-project".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 100000,
                    output_tokens: 50000,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 25,
            }],
            token_counts: TokenCounts {
                input_tokens: 100000,
                output_tokens: 50000,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
            },
            is_active: true,
        };
    display_active_window(Some(&window_comfortable));
}