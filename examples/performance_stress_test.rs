use anyhow::Result;
use chrono::{Duration, Utc};
use clauditor::scanner::SessionScanner;
use clauditor::coordinator::load_and_group_sessions;
use std::fs;
use std::time::Instant;
use tempfile::TempDir;
use std::env;

fn main() -> Result<()> {
    println!("=== Clauditor Performance Stress Test ===\n");
    println!("Testing with larger session files (1000+ entries each)...\n");
    
    // Create temporary directory structure
    let temp_dir = TempDir::new()?;
    let claude_dir = temp_dir.path().join(".claude");
    let projects_dir = claude_dir.join("projects");
    fs::create_dir_all(&projects_dir)?;
    
    println!("Generating stress test data: 50 sessions with 1000+ entries each...");
    
    // Generate 50 sessions with much more data
    let mut total_entries = 0;
    for project_idx in 0..10 {
        let project_name = format!("-Users-phaedrus-Development-bigproject-{}", project_idx);
        let project_dir = projects_dir.join(&project_name);
        fs::create_dir(&project_dir)?;
        
        // 5 sessions per project
        for session_idx in 0..5 {
            let session_id = format!("b{:06x}-{:04x}-{:04x}-{:04x}-{:012x}", 
                project_idx * 100000 + session_idx,
                4000 + project_idx,
                5000 + session_idx,
                6000,
                project_idx * 1000000 + session_idx);
            let session_file = project_dir.join(format!("{}.jsonl", session_id));
            
            // Generate MANY entries for this session (1000-2000)
            let num_entries = 1000 + (project_idx * 100) + (session_idx * 200);
            let entries = generate_large_session(project_idx, session_idx, num_entries);
            total_entries += entries.len();
            
            // Write entries to file
            let mut content = String::new();
            for entry in entries {
                content.push_str(&serde_json::to_string(&entry)?);
                content.push('\n');
            }
            fs::write(&session_file, content)?;
        }
    }
    
    println!("Generated {} total entries across 50 sessions\n", total_entries);
    
    // Temporarily set HOME to our temp directory
    let original_home = env::var("HOME").unwrap_or_default();
    env::set_var("HOME", temp_dir.path());
    
    // Measure initial scan performance with large files
    println!("Measuring initial scan performance with large files...");
    
    let start = Instant::now();
    
    // Create scanner and perform initial scan
    let mut scanner = SessionScanner::new();
    let sessions = scanner.load_sessions()?;
    
    let scan_duration = start.elapsed();
    
    // Count total entries
    let total_loaded_entries: usize = sessions.iter().map(|s| s.entries.len()).sum();
    
    println!("Stress test scan results:");
    println!("  - Found {} session files", sessions.len());
    println!("  - Loaded {} entries (avg {} per session)", total_loaded_entries, total_loaded_entries / sessions.len());
    println!("  - Scan time: {:.2}ms", scan_duration.as_millis());
    
    // Check if we meet the performance target even with large files
    if scan_duration.as_millis() < 100 {
        println!("  ✓ PASS: Scan completed in under 100ms even with large files!");
    } else {
        println!("  ⚠ WARNING: Scan took {}ms (target: <100ms)", scan_duration.as_millis());
        println!("    Note: This is with unusually large session files ({} entries avg)", total_loaded_entries / sessions.len());
    }
    
    // Test full pipeline performance
    println!("\nMeasuring full pipeline performance with large data...");
    
    let start_full = Instant::now();
    let window = load_and_group_sessions()?;
    let full_duration = start_full.elapsed();
    
    let window_count = if window.is_some() { 1 } else { 0 };
    println!("  - Created {} billing window", window_count);
    println!("  - Full pipeline time: {:.2}ms", full_duration.as_millis());
    
    // Calculate actual memory usage more accurately
    use std::mem::size_of;
    use clauditor::types::{UsageEntry};
    
    let entry_size = size_of::<UsageEntry>();
    
    // Include string allocations (rough estimate)
    let avg_string_overhead = 100; // bytes per entry for string data
    let total_entry_memory = total_loaded_entries * (entry_size + avg_string_overhead);
    let total_mb = total_entry_memory as f64 / 1_048_576.0;
    
    println!("\nMemory usage with large files:");
    println!("  - Total entries: {}", total_loaded_entries);
    println!("  - Entry size + string overhead: ~{} bytes", entry_size + avg_string_overhead);
    println!("  - Estimated total memory: {:.2} MB", total_mb);
    
    if total_mb < 50.0 {
        println!("  ✓ PASS: Memory usage under 50MB even with large files");
    } else {
        println!("  ⚠ WARNING: Memory usage {:.2}MB exceeds 50MB target", total_mb);
        println!("    Note: This test uses exceptionally large session files");
    }
    
    // Test performance with release build suggestion
    if scan_duration.as_millis() > 100 || total_mb > 50.0 {
        println!("\n💡 TIP: For better performance, compile with --release:");
        println!("   cargo build --release --example performance_stress_test");
        println!("   ./target/release/examples/performance_stress_test");
    }
    
    // Restore original HOME
    env::set_var("HOME", original_home);
    
    Ok(())
}

fn generate_large_session(project_idx: usize, session_idx: usize, num_entries: usize) -> Vec<serde_json::Value> {
    let mut entries = Vec::new();
    let now = Utc::now();
    
    // All stress test sessions are recent/active
    let session_start = now - Duration::hours(2);
    
    for i in 0..num_entries {
        let timestamp = session_start + Duration::seconds((i * 5) as i64);
        
        let entry = serde_json::json!({
            "timestamp": timestamp.to_rfc3339(),
            "message": {
                "id": format!("msg_{}_{}_{}", project_idx, session_idx, i),
                "type": "message", 
                "role": "assistant",
                "model": "claude-opus-4-20250514",
                "usage": {
                    "input_tokens": 500 + (i % 500),
                    "output_tokens": 1000 + (i % 1000),
                    "cache_creation_input_tokens": if i % 20 == 0 { 5000 } else { 0 },
                    "cache_read_input_tokens": if i % 10 == 0 { 2500 } else { 0 }
                }
            },
            "requestId": format!("req_{}_{}_{}", project_idx, session_idx, i),
            "version": "1.0.51"
        });
        
        entries.push(entry);
    }
    
    entries
}