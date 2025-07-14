use chrono::{DateTime, Duration, Local, Utc};
use crate::types::SessionBlock;
use std::path::{Path, PathBuf};

/// ANSI color constants for terminal output
pub mod colors {
    pub const CYAN: &str = "\x1B[36m";
    pub const GREEN: &str = "\x1B[32m";
    pub const YELLOW: &str = "\x1B[33m";
    pub const ORANGE: &str = "\x1B[38;5;208m"; // Orange using 256-color mode
    pub const RED: &str = "\x1B[31m";
    pub const DIM: &str = "\x1B[2m";
    pub const RESET: &str = "\x1B[0m";
}

/// Get the terminal width in columns, defaulting to 80 if detection fails
pub fn get_terminal_width() -> u16 {
    #[cfg(unix)]
    {
        use libc::{ioctl, isatty, winsize, STDOUT_FILENO, TIOCGWINSZ};
        use std::mem;
        
        // Check if stdout is a terminal
        if unsafe { isatty(STDOUT_FILENO) } == 0 {
            return 80;
        }
        
        let mut size: winsize = unsafe { mem::zeroed() };
        
        // Try to get terminal size
        if unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) } == 0 && size.ws_col > 0 {
            size.ws_col
        } else {
            80
        }
    }
    
    #[cfg(not(unix))]
    {
        // Default to 80 columns on non-Unix platforms
        80
    }
}

/// Clean project paths by removing common prefix and handling home directory
/// 
/// Takes a slice of project paths and:
/// 1. Finds the longest common prefix among all paths
/// 2. Removes the common prefix from each path
/// 3. Replaces home directory with ~ when appropriate
/// 4. Cleans up double slashes
/// 
/// # Examples
/// ```
/// let paths = vec![
///     "/Users/phaedrus/Development/foo".to_string(),
///     "/Users/phaedrus/Development/bar".to_string(),
/// ];
/// let cleaned = clean_project_paths(&paths);
/// assert_eq!(cleaned, vec!["foo", "bar"]);
/// ```
#[allow(dead_code)]
pub fn clean_project_paths(paths: &[String]) -> Vec<String> {
    use std::env;
    
    if paths.is_empty() {
        return vec![];
    }
    
    // Get home directory
    let home_dir = env::var("HOME").ok()
        .or_else(|| env::var("USERPROFILE").ok())
        .map(PathBuf::from);
    
    // Clean double slashes and normalize paths
    let normalized_paths: Vec<PathBuf> = paths.iter()
        .map(|p| {
            // Replace double slashes with single
            let cleaned = p.replace("//", "/");
            PathBuf::from(cleaned)
        })
        .collect();
    
    if normalized_paths.len() == 1 {
        // Single path - just clean it up
        let path = &normalized_paths[0];
        return vec![clean_single_path(path, home_dir.as_ref())];
    }
    
    // Find common prefix for multiple paths
    let common_prefix = find_common_prefix(&normalized_paths);
    
    // Check if common prefix is under home
    let show_home_relative = if let (Some(home), Some(prefix)) = (&home_dir, &common_prefix) {
        prefix.starts_with(home) && prefix != home
    } else {
        false
    };
    
    // Process each path
    normalized_paths.iter()
        .map(|path| {
            if show_home_relative {
                // Show paths relative to home
                if let Some(home) = &home_dir {
                    if let Ok(stripped) = path.strip_prefix(home) {
                        if stripped.components().count() == 0 {
                            return "~".to_string();
                        }
                        return format!("~/{}", stripped.display());
                    }
                }
            }
            
            // Try to remove common prefix
            if let Some(prefix) = &common_prefix {
                if let Ok(relative) = path.strip_prefix(prefix) {
                    let relative_str = relative.display().to_string();
                    if !relative_str.is_empty() {
                        // If prefix is exactly home, prepend ~
                        if let Some(home) = &home_dir {
                            if prefix == home {
                                return format!("~/{}", relative_str);
                            }
                        }
                        return relative_str;
                    }
                }
            }
            
            // If no common prefix, check if under home
            if let Some(home) = &home_dir {
                if let Ok(stripped) = path.strip_prefix(home) {
                    if stripped.components().count() == 0 {
                        return "~".to_string();
                    }
                    return format!("~/{}", stripped.display());
                }
            }
            
            // Fallback to full path
            path.display().to_string()
        })
        .collect()
}

/// Find the longest common prefix among paths
#[allow(dead_code)]
fn find_common_prefix(paths: &[PathBuf]) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }
    
    let first = &paths[0];
    let mut common = PathBuf::new();
    
    'outer: for component in first.components() {
        // Check if this component exists in all paths
        for path in &paths[1..] {
            let mut path_components = path.components();
            let mut found = false;
            
            // Try to match components up to this point
            let mut temp_common = PathBuf::new();
            for common_comp in common.components() {
                if let Some(path_comp) = path_components.next() {
                    if common_comp == path_comp {
                        temp_common.push(path_comp);
                    } else {
                        break 'outer;
                    }
                } else {
                    break 'outer;
                }
            }
            
            // Check the current component
            if let Some(path_comp) = path_components.next() {
                if component == path_comp {
                    found = true;
                }
            }
            
            if !found {
                break 'outer;
            }
        }
        
        common.push(component);
    }
    
    // Only return prefix if it's meaningful (at least one component)
    if common.components().count() > 0 {
        Some(common)
    } else {
        None
    }
}

/// Clean a single path
#[allow(dead_code)]
fn clean_single_path(path: &Path, home_dir: Option<&PathBuf>) -> String {
    // Just return the last component (project name) for single paths
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Only if there's no filename (e.g., path is "/"), check for home
            if let Some(home) = home_dir {
                if path == home {
                    return "~".to_string();
                }
            }
            path.display().to_string()
        })
}



/// Format a duration as "Xh Ym" or "Xm" for durations under an hour with color coding
pub fn format_duration(duration: Duration) -> String {
    let total_minutes = duration.num_minutes();
    
    if total_minutes <= 0 {
        return "0m".to_string();
    }
    
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    
    let time_str = if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    };
    
    // Apply color coding based on time remaining
    if total_minutes <= 30 {
        // Red for <30m (urgent)
        format!("{}{}{}", colors::RED, time_str, colors::RESET)
    } else if total_minutes <= 60 {
        // Yellow for <=1h (warning)
        format!("{}{}{}", colors::YELLOW, time_str, colors::RESET)
    } else if total_minutes > 120 {
        // Green for >2h (plenty of time)
        format!("{}{}{}", colors::GREEN, time_str, colors::RESET)
    } else {
        // No color for 1h-2h range
        time_str
    }
}

/// Format a number with comma separators (e.g., 12345 -> "12,345")
pub fn format_number(num: u64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let mut count = 0;
    
    for ch in num_str.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
        count += 1;
    }
    
    result.chars().rev().collect()
}

/// Format burn rate with color coding based on value
pub fn format_burn_rate(burn_rate: f64) -> String {
    let rate_str = format!("{} tokens/min", format_number(burn_rate as u64));
    
    if burn_rate > 1_000_000.0 {
        // Red for >1M/min (very high)
        format!("{}{}{}", colors::RED, rate_str, colors::RESET)
    } else if burn_rate > 500_000.0 {
        // Orange for 500K-1M/min (high)
        format!("{}{}{}", colors::ORANGE, rate_str, colors::RESET)
    } else if burn_rate > 100_000.0 {
        // Yellow for 100K-500K/min (moderate)
        format!("{}{}{}", colors::YELLOW, rate_str, colors::RESET)
    } else if burn_rate < 50_000.0 {
        // Green for <50K/min (sustainable)
        format!("{}{}{}", colors::GREEN, rate_str, colors::RESET)
    } else {
        // No color for 50K-100K range (normal)
        rate_str
    }
}

/// Format a timestamp as time only in local timezone (e.g., "2:00 PM")
pub fn format_time(timestamp: DateTime<Utc>) -> String {
    let local_time: DateTime<Local> = timestamp.with_timezone(&Local);
    local_time.format("%-I:%M %p").to_string()
}

/// Extract a meaningful display name from a project path
/// Handles cases like:
/// - /Users/name/Development/project -> project
/// - /Users/name/Development/adminifi-web/feature-a-120 -> adminifi-web/feature-a-120
/// - Simple names -> unchanged
fn extract_display_name(project_path: &str) -> String {
    // If it's not a path, return as-is
    if !project_path.contains('/') {
        return project_path.to_string();
    }
    
    let parts: Vec<&str> = project_path.split('/').collect();
    let len = parts.len();
    
    // If path has 'Development' in it, show everything after it
    if let Some(dev_pos) = parts.iter().position(|&p| p == "Development") {
        if dev_pos + 1 < len {
            let result = parts[(dev_pos + 1)..]
                .iter()
                .filter(|&&p| !p.is_empty()) // Skip empty parts from double slashes
                .cloned()
                .collect::<Vec<_>>()
                .join("/");
            return result;
        }
    }
    
    // For other paths, show the last 2 components if available (org/project style)
    if len >= 2 {
        // Check if second-to-last component looks like an org/parent directory
        let parent = parts[len - 2];
        let name = parts[len - 1];
        
        // If parent looks meaningful (not generic like 'src', 'projects', etc)
        if parent.len() > 2 && !["src", "projects", "repos", "code", "git"].contains(&parent) {
            return format!("{}/{}", parent, name);
        }
    }
    
    // Otherwise just return the last component
    parts.last().unwrap_or(&project_path).to_string()
}

/// Display the billing window
pub fn display_window(window: &SessionBlock, now: DateTime<Utc>) {
    let time_remaining = window.time_remaining(now);
    let time_remaining_str = if time_remaining > Duration::zero() {
        format!("ends in {}", format_duration(time_remaining))
    } else {
        "ended".to_string()
    };
    
    println!("Started {}, {}",
        format_time(window.start_time),
        time_remaining_str
    );
    
    println!("Total: {} tokens ({})",
        format_number(window.token_counts.total()),
        format_burn_rate(window.burn_rate())
    );
    
    println!();
    
    // Display projects sorted by token count (highest first)
    let mut projects = window.projects.clone();
    projects.sort_by(|a, b| b.token_counts.total().cmp(&a.token_counts.total()));
    
    // Get terminal width for dynamic alignment
    let terminal_width = get_terminal_width() as usize;
    let tokens_suffix = " tokens";
    let total_tokens = window.token_counts.total();
    
    for project in &projects {
        let project_tokens = project.token_counts.total();
        let token_count_str = format_number(project_tokens);
        let token_display = format!("{} {}", token_count_str, tokens_suffix.trim());
        
        // Calculate percentage
        let percentage = if total_tokens > 0 {
            (project_tokens as f64 / total_tokens as f64 * 100.0) as u32
        } else {
            0
        };
        let percentage_str = format!("{}%", percentage);
        
        // Calculate available space
        let token_display_len = token_display.len();
        let percentage_len = percentage_str.len();
        
        // Calculate max project name length (accounting for percentage + spacing)
        let min_spacing = 2; // Minimum spaces between components
        let max_name_len = terminal_width.saturating_sub(token_display_len + percentage_len + min_spacing * 2);
        
        // Extract a meaningful project name from the full path
        let display_name = extract_display_name(&project.name);
        
        // Truncate project name if necessary
        let project_name = if display_name.len() > max_name_len && max_name_len > 3 {
            format!("{}...", &display_name[..max_name_len - 3])
        } else {
            display_name
        };
        
        // Calculate padding for alignment
        let used_len = project_name.len() + percentage_len + token_display_len + min_spacing * 2;
        let padding_len = terminal_width.saturating_sub(used_len);
        let padding = " ".repeat(padding_len);
        
        // Print with percentage right-aligned before token count
        println!("{}{}{:>4}  {}", project_name, padding, percentage_str, token_display);
    }
    
    println!();
}

/// Display the active billing window
pub fn display_active_window(window: Option<&SessionBlock>) {
    let now = Utc::now();
    
    match window {
        None => {
            println!("No active billing window");
        }
        Some(w) => {
            // Display header with color
            println!("{}Active billing window{}", colors::CYAN, colors::RESET);
            
            // Display separator line
            let terminal_width = get_terminal_width() as usize;
            let separator = "─".repeat(terminal_width.min(80)); // Cap at 80 chars to avoid overly long lines
            println!("{}{}{}", colors::DIM, separator, colors::RESET);
            println!();
            
            display_window(w, now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TokenCounts;
    
    #[test]
    fn test_extract_display_name() {
        // Simple cases
        assert_eq!(extract_display_name("project"), "project");
        assert_eq!(extract_display_name("/Users/name/Development/project"), "project");
        
        // Multi-component after Development
        assert_eq!(
            extract_display_name("/Users/name/Development/adminifi-web/feature-a-120"),
            "adminifi-web/feature-a-120"
        );
        
        // What happens with double slashes - they get cleaned up
        assert_eq!(
            extract_display_name("/Users/phaedrus/Development/adminifi/web//feature/a/120"),
            "adminifi/web/feature/a/120"  // Double slashes removed
        );
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::minutes(0)), "0m");
        assert_eq!(format_duration(Duration::minutes(30)), "\x1B[31m30m\x1B[0m"); // Red
        assert_eq!(format_duration(Duration::minutes(45)), "\x1B[33m45m\x1B[0m"); // Yellow
        assert_eq!(format_duration(Duration::minutes(60)), "\x1B[33m1h 0m\x1B[0m"); // Yellow (1h exactly)
        assert_eq!(format_duration(Duration::minutes(90)), "1h 30m"); // No color
        assert_eq!(format_duration(Duration::minutes(135)), "\x1B[32m2h 15m\x1B[0m"); // Green
        assert_eq!(format_duration(Duration::minutes(180)), "\x1B[32m3h 0m\x1B[0m"); // Green
    }
    
    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(12345), "12,345");
        assert_eq!(format_number(1234567), "1,234,567");
    }
    
    #[test]
    fn test_format_burn_rate() {
        // Normal rate (no color)
        assert_eq!(format_burn_rate(100.0), "100 tokens/min");
        assert_eq!(format_burn_rate(50000.0), "50000 tokens/min");
        assert_eq!(format_burn_rate(499999.0), "499999 tokens/min");
        
        // High rate (yellow)
        assert_eq!(format_burn_rate(500001.0), "\x1B[33m500001 tokens/min\x1B[0m");
        assert_eq!(format_burn_rate(750000.0), "\x1B[33m750000 tokens/min\x1B[0m");
        assert_eq!(format_burn_rate(999999.0), "\x1B[33m999999 tokens/min\x1B[0m");
        
        // Extremely high rate (red)
        assert_eq!(format_burn_rate(1000001.0), "\x1B[31m1000001 tokens/min\x1B[0m");
        assert_eq!(format_burn_rate(2000000.0), "\x1B[31m2000000 tokens/min\x1B[0m");
        assert_eq!(format_burn_rate(5000000.0), "\x1B[31m5000000 tokens/min\x1B[0m");
    }
    
    #[test]
    fn test_format_time() {
        let time = DateTime::parse_from_rfc3339("2024-01-15T14:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted = format_time(time);
        // Check that it doesn't contain UTC and has the expected format
        assert!(!formatted.contains("UTC"));
        assert!(formatted.contains(":"));
        assert!(formatted.contains("M")); // AM or PM
        
        let morning_time = DateTime::parse_from_rfc3339("2024-01-15T09:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted_morning = format_time(morning_time);
        assert!(!formatted_morning.contains("UTC"));
        assert!(formatted_morning.contains(":"));
        assert!(formatted_morning.contains("M")); // AM or PM
    }
    
    #[test]
    fn test_get_terminal_width() {
        let width = get_terminal_width();
        // Should return a reasonable width (at least 40 columns)
        assert!(width >= 40);
        // Should not exceed typical maximum (300 columns)
        assert!(width <= 300);
        // In CI or non-terminal environments, should return default 80
        if std::env::var("CI").is_ok() {
            assert_eq!(width, 80);
        }
    }
    
    #[test]
    fn test_clean_project_paths_common_prefix() {
        let paths = vec![
            "/Users/phaedrus/Development/foo".to_string(),
            "/Users/phaedrus/Development/bar".to_string(),
            "/Users/phaedrus/Development/baz/nested".to_string(),
        ];
        let cleaned = clean_project_paths(&paths);
        assert_eq!(cleaned, vec!["foo", "bar", "baz/nested"]);
    }
    
    #[test]
    fn test_clean_project_paths_home_directory() {
        use std::env;
        
        // Set HOME for test
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", "/Users/phaedrus");
        
        let paths = vec![
            "/Users/phaedrus/claude/project1".to_string(),
            "/Users/phaedrus/claude/project2".to_string(),
        ];
        let cleaned = clean_project_paths(&paths);
        assert_eq!(cleaned, vec!["~/claude/project1", "~/claude/project2"]);
        
        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        }
    }
    
    #[test]
    fn test_clean_project_paths_double_slashes() {
        let paths = vec![
            "/Users/phaedrus//Development//foo".to_string(),
            "/Users/phaedrus//Development/bar".to_string(),
        ];
        let cleaned = clean_project_paths(&paths);
        assert_eq!(cleaned, vec!["foo", "bar"]);
    }
    
    #[test]
    fn test_clean_project_paths_single_path() {
        let paths = vec![
            "/Users/phaedrus/Development/very/long/path/to/project".to_string(),
        ];
        let cleaned = clean_project_paths(&paths);
        // Single path should show just the project name
        assert_eq!(cleaned, vec!["project"]);
    }
    
    #[test]
    fn test_clean_project_paths_empty() {
        let paths: Vec<String> = vec![];
        let cleaned = clean_project_paths(&paths);
        assert_eq!(cleaned, Vec::<String>::new());
    }
    
    #[test]
    fn test_clean_project_paths_no_common_prefix() {
        let paths = vec![
            "/Users/alice/projects/foo".to_string(),
            "/home/bob/code/bar".to_string(),
            "/var/www/baz".to_string(),
        ];
        let cleaned = clean_project_paths(&paths);
        // When no common prefix, paths should be returned relatively unchanged
        assert_eq!(cleaned.len(), 3);
        assert!(cleaned[0].contains("foo") || cleaned[0].contains("alice"));
        assert!(cleaned[1].contains("bar") || cleaned[1].contains("bob"));
        assert!(cleaned[2].contains("baz") || cleaned[2].contains("www"));
    }
    
    #[test]
    fn test_clean_project_paths_home_only() {
        use std::env;
        
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", "/Users/phaedrus");
        
        let paths = vec![
            "/Users/phaedrus".to_string(),
        ];
        let cleaned = clean_project_paths(&paths);
        assert_eq!(cleaned, vec!["~"]);
        
        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        }
    }
    
    #[test]
    fn test_display_window_with_number() {
        use crate::types::{SessionBlock, ProjectUsage};
        use std::io::{self, Write};
        
        // Create a test window
        let now = Utc::now();
        let window = SessionBlock {
            start_time: now - Duration::hours(1),
            end_time: now + Duration::hours(4),
            last_activity: now,
            projects: vec![ProjectUsage {
                name: "test-project".to_string(),
                token_counts: TokenCounts {
                    input_tokens: 1000,
                    output_tokens: 500,
                    cache_creation_tokens: 0,
                    cache_read_tokens: 0,
                },
                entry_count: 10,
            }],
            token_counts: TokenCounts {
                input_tokens: 1000,
                output_tokens: 500,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
            },
            is_active: true,
        };
        
        // Test window display
        println!("=== Window display ===");
        display_window(&window, now);
        
        // Visual verification - test passes if it compiles and runs
        assert!(true);
    }
}