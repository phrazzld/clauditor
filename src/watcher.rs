use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use anyhow::{Context, Result};

/// File system event that we care about
#[derive(Debug, Clone)]
pub enum FileEvent {
    Modified(PathBuf),
    Created(PathBuf),
}

/// Watch Claude session directories for JSONL file changes
pub struct SessionWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<FileEvent>,
}

impl SessionWatcher {
    /// Create a new session watcher for the given paths
    pub fn new(paths: Vec<PathBuf>) -> Result<Self> {
        let (tx, rx) = channel();
        
        // Create the file system watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // Filter for JSONL files and relevant events
                    if let Some(file_event) = filter_event(event) {
                        // Ignore send errors (receiver might be dropped)
                        let _ = tx.send(file_event);
                    }
                }
            },
            Config::default(),
        ).context("Failed to create file watcher")?;
        
        // Watch each projects directory
        for base_path in &paths {
            let projects_dir = base_path.join("projects");
            if projects_dir.exists() {
                watcher.watch(&projects_dir, RecursiveMode::Recursive)
                    .with_context(|| format!("Failed to watch directory: {}", projects_dir.display()))?;
            }
        }
        
        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }
    
    /// Check for file events (non-blocking)
    pub fn poll_events(&self) -> Vec<FileEvent> {
        let mut events = Vec::new();
        
        // Drain all pending events
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        
        events
    }
    
    /// Create a watcher with default Claude paths
    pub fn with_default_paths() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let paths = vec![
            PathBuf::from(&home).join(".claude"),
            PathBuf::from(&home).join(".config/claude"),
        ];
        
        Self::new(paths)
    }
}

/// Filter file system events to only JSONL file modifications and creations
fn filter_event(event: Event) -> Option<FileEvent> {
    match event.kind {
        EventKind::Modify(_) => {
            // File was modified
            for path in &event.paths {
                if is_jsonl_file(path) {
                    return Some(FileEvent::Modified(path.clone()));
                }
            }
        }
        EventKind::Create(_) => {
            // File was created
            for path in &event.paths {
                if is_jsonl_file(path) {
                    return Some(FileEvent::Created(path.clone()));
                }
            }
        }
        _ => {}
    }
    
    None
}

/// Check if a path is a JSONL file
fn is_jsonl_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "jsonl")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;
    
    #[test]
    fn test_is_jsonl_file() {
        assert!(is_jsonl_file(Path::new("session.jsonl")));
        assert!(is_jsonl_file(Path::new("/path/to/file.jsonl")));
        assert!(!is_jsonl_file(Path::new("file.json")));
        assert!(!is_jsonl_file(Path::new("file.txt")));
        assert!(!is_jsonl_file(Path::new("file")));
    }
    
    #[test]
    fn test_watcher_detects_changes() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let projects_dir = temp_dir.path().join("projects");
        fs::create_dir_all(&projects_dir)?;
        
        // Create watcher
        let watcher = SessionWatcher::new(vec![temp_dir.path().to_path_buf()])?;
        
        // Give watcher time to initialize
        thread::sleep(Duration::from_millis(100));
        
        // Create a JSONL file
        let test_file = projects_dir.join("test.jsonl");
        let mut file = File::create(&test_file)?;
        writeln!(file, r#"{{"test": "data"}}"#)?;
        file.sync_all()?;
        
        // Give watcher time to detect
        thread::sleep(Duration::from_millis(100));
        
        // Check for events
        let events = watcher.poll_events();
        assert!(!events.is_empty(), "Should have detected file creation");
        
        // Verify we got a creation event for our file
        let has_creation = events.iter().any(|e| {
            matches!(e, FileEvent::Created(p) if p.ends_with("test.jsonl"))
        });
        assert!(has_creation, "Should have creation event for test.jsonl");
        
        Ok(())
    }
}