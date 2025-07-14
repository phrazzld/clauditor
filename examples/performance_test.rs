use anyhow::Result;
use chrono::{Duration, Utc};
use clauditor::scanner::SessionScanner;
use clauditor::coordinator::load_and_group_sessions;
use std::fs;
use std::time::Instant;
use tempfile::TempDir;
use std::env;

fn main() -> Result<()> {
    println!("=== Clauditor Performance Test ===\n");
    
    // Create temporary directory structure
    let temp_dir = TempDir::new()?;
    let claude_dir = temp_dir.path().join(".claude");
    let projects_dir = claude_dir.join("projects");
    fs::create_dir_all(&projects_dir)?;
    
    println!("Generating test data: 50 sessions across multiple projects...");
    
    // Generate 50 sessions across 10 projects
    let mut total_entries = 0;
    for project_idx in 0..10 {
        let project_name = format!("-Users-phaedrus-Development-project-{}", project_idx);
        let project_dir = projects_dir.join(&project_name);
        fs::create_dir(&project_dir)?;
        
        // 5 sessions per project
        for session_idx in 0..5 {
            let session_id = format!("a{:06x}-{:04x}-{:04x}-{:04x}-{:012x}", 
                project_idx * 100000 + session_idx,
                4000 + project_idx,
                5000 + session_idx,
                6000,
                project_idx * 1000000 + session_idx);
            let session_file = project_dir.join(format!("{}.jsonl", session_id));
            
            // Generate entries for this session
            let entries = generate_session_entries(project_idx, session_idx);
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
    
    // Measure initial scan performance
    println!("Measuring initial scan performance...");
    
    let start = Instant::now();
    
    // Create scanner and perform initial scan
    let mut scanner = SessionScanner::new();
    let sessions = scanner.load_sessions()?;
    
    let scan_duration = start.elapsed();
    
    // Count total entries
    let total_loaded_entries: usize = sessions.iter().map(|s| s.entries.len()).sum();
    
    println!("Initial scan results:");
    println!("  - Found {} session files", sessions.len());
    println!("  - Loaded {} entries", total_loaded_entries);
    println!("  - Scan time: {:.2}ms", scan_duration.as_millis());
    
    // Check if we meet the performance target
    if scan_duration.as_millis() < 100 {
        println!("  ✓ PASS: Scan completed in under 100ms");
    } else {
        println!("  ✗ FAIL: Scan took longer than 100ms target");
    }
    
    // Test full pipeline performance (including window grouping)
    println!("\nMeasuring full pipeline performance (scan + window grouping)...");
    
    let start_full = Instant::now();
    let window = load_and_group_sessions()?;
    let full_duration = start_full.elapsed();
    
    let window_count = if window.is_some() { 1 } else { 0 };
    println!("  - Created {} billing window", window_count);
    println!("  - Full pipeline time: {:.2}ms", full_duration.as_millis());
    
    // Estimate memory usage
    use std::mem::size_of;
    use clauditor::types::{UsageEntry, SessionFile, SessionBlock};
    
    let entry_size = size_of::<UsageEntry>();
    let session_size = size_of::<SessionFile>();
    let block_size = size_of::<SessionBlock>();
    
    // Rough estimate - actual usage will be higher due to heap allocations
    let entries_memory = total_loaded_entries * entry_size;
    let sessions_memory = sessions.len() * session_size;
    let blocks_memory = window_count * block_size;
    let estimated_memory = entries_memory + sessions_memory + blocks_memory;
    let estimated_mb = estimated_memory as f64 / 1_048_576.0;
    
    println!("\nMemory usage estimate:");
    println!("  - Entry size: {} bytes", entry_size);
    println!("  - Session size: {} bytes", session_size);
    println!("  - Block size: {} bytes", block_size);
    println!("  - Total entries: {}", total_loaded_entries);
    println!("  - Estimated memory: {:.2} MB", estimated_mb);
    
    if estimated_mb < 50.0 {
        println!("  ✓ PASS: Estimated memory usage under 50MB");
    } else {
        println!("  ✗ FAIL: Estimated memory usage exceeds 50MB");
    }
    
    // Test incremental scan performance
    println!("\nTesting incremental scan (should be minimal)...");
    let start_inc = Instant::now();
    let incremental_sessions = scanner.load_sessions_incremental()?;
    let inc_duration = start_inc.elapsed();
    println!("  - Incremental scan found {} new entries", incremental_sessions.len());
    println!("  - Incremental scan time: {:.2}ms", inc_duration.as_millis());
    
    // Restore original HOME
    env::set_var("HOME", original_home);
    
    Ok(())
}

fn generate_session_entries(project_idx: usize, session_idx: usize) -> Vec<serde_json::Value> {
    let mut entries = Vec::new();
    let now = Utc::now();
    
    // Determine if this session is active (30% chance)
    let is_active = (project_idx + session_idx) % 3 == 0;
    
    // Start time depends on whether session is active
    let start_offset = if is_active {
        // Active sessions started within last 5 hours
        Duration::hours((session_idx % 4) as i64)
    } else {
        // Inactive sessions started 6-24 hours ago
        Duration::hours(6 + (session_idx % 18) as i64)
    };
    
    let session_start = now - start_offset;
    
    // Generate 50-200 entries per session
    let num_entries = 50 + (project_idx * 10) + (session_idx * 20);
    
    for i in 0..num_entries {
        let timestamp = session_start + Duration::minutes(i as i64);
        
        let entry = serde_json::json!({
            "timestamp": timestamp.to_rfc3339(),
            "message": {
                "id": format!("msg_{}_{}_{}", project_idx, session_idx, i),
                "type": "message",
                "role": "assistant",
                "model": if project_idx % 2 == 0 { "claude-opus-4-20250514" } else { "claude-sonnet-4-20250514" },
                "usage": {
                    "input_tokens": 100 + (i % 50),
                    "output_tokens": 200 + (i % 100),
                    "cache_creation_input_tokens": if i % 10 == 0 { 1000 } else { 0 },
                    "cache_read_input_tokens": if i % 5 == 0 { 500 } else { 0 }
                }
            },
            "requestId": format!("req_{}_{}_{}", project_idx, session_idx, i),
            "version": "1.0.51"
        });
        
        entries.push(entry);
    }
    
    entries
}