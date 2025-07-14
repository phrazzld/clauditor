use chrono::{Duration, Utc};
use clauditor::display::display_active_window;
use clauditor::types::{SessionBlock, ProjectUsage, TokenCounts};

fn main() {
    let now = Utc::now();
    
    // Test with various burn rates
    
    println!("=== Testing burn rate color coding ===");
    println!("Notice the color coding:");
    println!("- Red: > 1M tokens/min (extremely high!)");
    println!("- Yellow: > 500K tokens/min (high)");
    println!("- No color: Normal burn rate");
    println!();
    
    // Normal burn rate
    println!("\n--- Normal burn rate ---");
    let window_normal = SessionBlock {
            start_time: now - Duration::hours(3),
            end_time: now + Duration::hours(2),
            last_activity: now,
            projects: vec![ProjectUsage {
                name: "low-usage-project".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 30000,
                    output_tokens: 15000,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 100,
            }],
            token_counts: TokenCounts {
                input_tokens: 30000,
                output_tokens: 15000,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
            },
            is_active: true,
        };
    display_active_window(Some(&window_normal));
    
    // High burn rate (>500K/min - yellow)
    println!("\n--- High burn rate (>500K/min - should be YELLOW) ---");
    let window_high = SessionBlock {
            start_time: now - Duration::minutes(5),
            end_time: now + Duration::hours(4) + Duration::minutes(55),
            last_activity: now,
            projects: vec![ProjectUsage {
                name: "high-usage-project".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 2000000,
                    output_tokens: 1000000,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 50,
            }],
            token_counts: TokenCounts {
                input_tokens: 2000000,
                output_tokens: 1000000,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
            },
            is_active: true,
        };
    display_active_window(Some(&window_high));
    
    // Extremely high burn rate (>1M/min - red)
    println!("\n--- Extremely high burn rate (>1M/min - should be RED) ---");
    let window_extreme = SessionBlock {
            start_time: now - Duration::minutes(2),
            end_time: now + Duration::hours(4) + Duration::minutes(58),
            last_activity: now,
            projects: vec![ProjectUsage {
                name: "extreme-usage-project".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 1500000,
                    output_tokens: 750000,
                    cache_creation_tokens: 250000,
                    cache_read_tokens: 100000,
                },
                entry_count: 20,
            }],
            token_counts: TokenCounts {
                input_tokens: 1500000,
                output_tokens: 750000,
                cache_creation_tokens: 250000,
                cache_read_tokens: 100000,
            },
            is_active: true,
        };
    display_active_window(Some(&window_extreme));
}