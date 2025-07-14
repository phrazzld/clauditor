use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Tracks last read positions for JSONL files
#[derive(Debug, Serialize, Deserialize)]
pub struct FilePositionTracker {
    positions: HashMap<String, u64>,
    cache_file: PathBuf,
}

impl FilePositionTracker {
    /// Create a new position tracker with default cache location
    pub fn new() -> Self {
        let cache_dir = std::env::temp_dir();
        let cache_file = cache_dir.join("clauditor_positions.json");
        
        let mut tracker = Self {
            positions: HashMap::new(),
            cache_file,
        };
        
        // Load existing positions if available
        let _ = tracker.load();
        tracker
    }
    
    /// Get the last read position for a file
    pub fn get_position(&self, path: &Path) -> u64 {
        let path_str = path.to_string_lossy().to_string();
        self.positions.get(&path_str).copied().unwrap_or(0)
    }
    
    /// Update the position for a file
    pub fn set_position(&mut self, path: &Path, position: u64) {
        let path_str = path.to_string_lossy().to_string();
        self.positions.insert(path_str, position);
    }
    
    /// Check if file has been truncated or replaced
    #[allow(dead_code)]
    pub fn validate_position(&self, path: &Path, current_size: u64) -> u64 {
        let stored_position = self.get_position(path);
        
        // If stored position is beyond current file size, file was truncated/replaced
        if stored_position > current_size {
            0
        } else {
            stored_position
        }
    }
    
    /// Save positions to cache file
    pub fn save(&self) -> Result<()> {
        let file = File::create(&self.cache_file)
            .context("Failed to create position cache file")?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.positions)
            .context("Failed to write position cache")?;
        Ok(())
    }
    
    /// Load positions from cache file
    fn load(&mut self) -> Result<()> {
        if !self.cache_file.exists() {
            return Ok(());
        }
        
        let file = File::open(&self.cache_file)
            .context("Failed to open position cache file")?;
        let reader = BufReader::new(file);
        self.positions = serde_json::from_reader(reader)
            .context("Failed to read position cache")?;
        Ok(())
    }
    
    /// Clean up stale entries (files that no longer exist)
    pub fn cleanup(&mut self) {
        let paths_to_remove: Vec<String> = self.positions
            .keys()
            .filter(|path_str| !Path::new(path_str).exists())
            .cloned()
            .collect();
        
        for path in paths_to_remove {
            self.positions.remove(&path);
        }
    }
}

impl Drop for FilePositionTracker {
    fn drop(&mut self) {
        // Save positions when tracker is dropped
        let _ = self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    
    #[test]
    fn test_position_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.jsonl");
        
        let mut tracker = FilePositionTracker::new();
        
        // Initially no position
        assert_eq!(tracker.get_position(&test_file), 0);
        
        // Set position
        tracker.set_position(&test_file, 1024);
        assert_eq!(tracker.get_position(&test_file), 1024);
        
        // Update position
        tracker.set_position(&test_file, 2048);
        assert_eq!(tracker.get_position(&test_file), 2048);
    }
    
    #[test]
    fn test_validate_position() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.jsonl");
        
        let mut tracker = FilePositionTracker::new();
        tracker.set_position(&test_file, 1000);
        
        // File size is larger than position - valid
        assert_eq!(tracker.validate_position(&test_file, 2000), 1000);
        
        // File size equals position - valid
        assert_eq!(tracker.validate_position(&test_file, 1000), 1000);
        
        // File size is smaller than position - file was truncated
        assert_eq!(tracker.validate_position(&test_file, 500), 0);
    }
    
    #[test]
    fn test_persistence() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_file = temp_dir.path().join("test_cache.json");
        let test_path = PathBuf::from("/test/file.jsonl");
        
        // Create and save tracker
        {
            let mut tracker = FilePositionTracker {
                positions: HashMap::new(),
                cache_file: cache_file.clone(),
            };
            tracker.set_position(&test_path, 12345);
            tracker.save()?;
        }
        
        // Load tracker
        {
            let mut tracker = FilePositionTracker {
                positions: HashMap::new(),
                cache_file,
            };
            tracker.load()?;
            assert_eq!(tracker.get_position(&test_path), 12345);
        }
        
        Ok(())
    }
    
    #[test]
    fn test_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("exists.jsonl");
        let missing_file = PathBuf::from("/nonexistent/file.jsonl");
        
        // Create the existing file
        File::create(&existing_file).unwrap();
        
        let mut tracker = FilePositionTracker::new();
        tracker.set_position(&existing_file, 100);
        tracker.set_position(&missing_file, 200);
        
        assert_eq!(tracker.positions.len(), 2);
        
        tracker.cleanup();
        
        // Should only keep the existing file
        assert_eq!(tracker.positions.len(), 1);
        assert_eq!(tracker.get_position(&existing_file), 100);
        assert_eq!(tracker.get_position(&missing_file), 0);
    }
}