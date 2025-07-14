use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Duration, Utc};
use anyhow::{Context, Result};

use crate::parser::{parse_file, parse_file_from_position};
use crate::types::{UsageEntry, SessionFile};
use crate::position_tracker::FilePositionTracker;

/// Scan for Claude Code session files
pub struct SessionScanner {
    claude_paths: Vec<PathBuf>,
    hours_back: i64,
    position_tracker: FilePositionTracker,
}

impl SessionScanner {
    /// Create a new scanner with default paths
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let claude_paths = vec![
            PathBuf::from(&home).join(".claude"),
            PathBuf::from(&home).join(".config/claude"),
        ];
        
        Self {
            claude_paths,
            hours_back: 10, // Default to 10 hours as per requirements
            position_tracker: FilePositionTracker::new(),
        }
    }
    
    /// Set how many hours back to scan
    pub fn with_hours_back(mut self, hours: i64) -> Self {
        self.hours_back = hours;
        self
    }
    
    /// Find all JSONL files modified within the time window
    pub fn find_session_files(&self) -> Result<Vec<PathBuf>> {
        let cutoff_time = Utc::now() - Duration::hours(self.hours_back);
        let mut all_files = Vec::new();
        
        for base_path in &self.claude_paths {
            let projects_dir = base_path.join("projects");
            
            // Skip if directory doesn't exist
            if !projects_dir.exists() {
                continue;
            }
            
            // Recursively find JSONL files
            let files = find_jsonl_files(&projects_dir, cutoff_time)?;
            all_files.extend(files);
        }
        
        Ok(all_files)
    }
    
    /// Load all session data from found files
    pub fn load_sessions(&mut self) -> Result<Vec<SessionFile>> {
        let files = self.find_session_files()?;
        let mut sessions = Vec::new();
        
        // Clean up stale entries from position tracker
        self.position_tracker.cleanup();
        
        for file_path in files {
            // eprintln!("[DEBUG] load_sessions: Processing file: {}", file_path.display());
            
            // Extract project name from path
            let project_name = extract_project_name(&file_path);
            let session_id = extract_session_id(&file_path);
            
            // Parse the file
            match parse_file(&file_path) {
                Ok(entries) => {
                    // eprintln!("[DEBUG] load_sessions: File {} has {} entries", file_path.display(), entries.len());
                    
                    if !entries.is_empty() {
                        // Log first and last entry timestamps
                        if let (Some(first), Some(last)) = (entries.first(), entries.last()) {
                            // eprintln!("[DEBUG] load_sessions: File {} entries span {} to {}", 
                            //     file_path.display(), first.timestamp, last.timestamp);
                        }
                        
                        sessions.push(SessionFile {
                            path: file_path.to_string_lossy().to_string(),
                            project: project_name,
                            session_id,
                            last_read_position: 0, // Will be used for incremental reading
                            entries,
                        });
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing {}: {}", file_path.display(), e);
                    // Continue with other files
                }
            }
        }
        
        // eprintln!("[DEBUG] load_sessions: Loaded {} sessions total", sessions.len());
        Ok(sessions)
    }
    
    /// Load sessions incrementally, only reading new data
    pub fn load_sessions_incremental(&mut self) -> Result<Vec<SessionFile>> {
        let files = self.find_session_files()?;
        let mut sessions = Vec::new();
        
        // Clean up stale entries from position tracker
        self.position_tracker.cleanup();
        
        for file_path in files {
            // Extract project name from path
            let project_name = extract_project_name(&file_path);
            let session_id = extract_session_id(&file_path);
            
            // Get last read position
            let last_position = self.position_tracker.get_position(&file_path);
            
            // Parse the file incrementally
            match parse_file_from_position(&file_path, last_position) {
                Ok((entries, new_position)) => {
                    // Update position tracker
                    self.position_tracker.set_position(&file_path, new_position);
                    
                    if !entries.is_empty() {
                        sessions.push(SessionFile {
                            path: file_path.to_string_lossy().to_string(),
                            project: project_name,
                            session_id,
                            last_read_position: new_position,
                            entries,
                        });
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing {}: {}", file_path.display(), e);
                    // Continue with other files
                }
            }
        }
        
        // Save position tracker state
        let _ = self.position_tracker.save();
        
        Ok(sessions)
    }
    
    /// Load all entries from all sessions (flattened)
    pub fn load_all_entries(&mut self) -> Result<Vec<UsageEntry>> {
        let sessions = self.load_sessions()?;
        let mut all_entries = Vec::new();
        
        for session in sessions {
            all_entries.extend(session.entries);
        }
        
        Ok(all_entries)
    }
}

/// Recursively find JSONL files modified after cutoff time
fn find_jsonl_files(dir: &Path, cutoff_time: DateTime<Utc>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recurse into subdirectories
            if let Ok(mut subdir_files) = find_jsonl_files(&path, cutoff_time) {
                files.append(&mut subdir_files);
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            // Check modification time
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_time: DateTime<Utc> = modified.into();
                    if modified_time > cutoff_time {
                        files.push(path);
                    }
                }
            }
        }
    }
    
    Ok(files)
}

/// Extract project name from file path
/// Path format: ~/.claude/projects/{project-name}/{session-uuid}.jsonl
fn extract_project_name(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .map(|s| decode_project_name(s))
        .unwrap_or_else(|| "unknown".to_string())
}

/// Extract session ID from file path
fn extract_session_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Decode project name from file system encoding
/// Example: -Users-phaedrus-Development-ccusage -> /Users/phaedrus/Development/ccusage
/// Note: Double hyphens (--) in encoded names represent path separators between 
/// components that themselves contain hyphens
fn decode_project_name(encoded: &str) -> String {
    if encoded.starts_with('-') {
        // Leading hyphen indicates absolute path
        // Simply replace all hyphens with slashes
        format!("/{}", encoded[1..].replace('-', "/"))
    } else {
        encoded.replace('-', "/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::SystemTime;
    use tempfile::TempDir;
    
    #[test]
    fn test_decode_project_name() {
        assert_eq!(
            decode_project_name("-Users-phaedrus-Development-ccusage"),
            "/Users/phaedrus/Development/ccusage"
        );
        
        assert_eq!(
            decode_project_name("relative-path-project"),
            "relative/path/project"
        );
        
        // Test that hyphens become slashes (including double hyphens)
        assert_eq!(
            decode_project_name("-Users-phaedrus-Development-adminifi-web--feature-a-120"),
            "/Users/phaedrus/Development/adminifi/web//feature/a/120"
        );
    }
    
    #[test]
    fn test_extract_project_name() {
        let path = PathBuf::from("/home/user/.claude/projects/-Users-phaedrus-Development-ccusage/session.jsonl");
        assert_eq!(
            extract_project_name(&path),
            "/Users/phaedrus/Development/ccusage"
        );
    }
    
    #[test]
    fn test_find_jsonl_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let projects_dir = temp_dir.path().join("projects");
        fs::create_dir_all(&projects_dir)?;
        
        // Create some test files
        let project_dir = projects_dir.join("test-project");
        fs::create_dir_all(&project_dir)?;
        
        // Recent file (should be found)
        let recent_file = project_dir.join("recent.jsonl");
        File::create(&recent_file)?;
        
        // Old file (should not be found)
        let old_file = project_dir.join("old.jsonl");
        File::create(&old_file)?;
        
        // Non-JSONL file (should not be found)
        let other_file = project_dir.join("other.txt");
        File::create(&other_file)?;
        
        // Set old file's modification time to 11 hours ago
        let eleven_hours_ago = SystemTime::now() - std::time::Duration::from_secs(11 * 3600);
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(eleven_hours_ago))?;
        
        // Find files modified in last 10 hours
        let cutoff = Utc::now() - Duration::hours(10);
        let files = find_jsonl_files(&projects_dir, cutoff)?;
        
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("recent.jsonl"));
        
        Ok(())
    }
}