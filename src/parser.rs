use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use anyhow::{Context, Result};

use crate::types::UsageEntry;

/// Parse a single JSONL line into a UsageEntry
pub fn parse_line(line: &str) -> Option<UsageEntry> {
    // Skip empty lines
    if line.trim().is_empty() {
        return None;
    }
    
    // Try to parse the JSON
    match serde_json::from_str::<UsageEntry>(line) {
        Ok(entry) => {
            // Only return entries that have usage data
            if entry.message.usage.is_some() {
                Some(entry)
            } else {
                None
            }
        }
        Err(_) => {
            // Silently skip malformed lines
            None
        }
    }
}

/// Parse a JSONL file and return all valid usage entries
pub fn parse_file(path: &Path) -> Result<Vec<UsageEntry>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        match line {
            Ok(line_content) => {
                if let Some(entry) = parse_line(&line_content) {
                    entries.push(entry);
                }
                // Silently skip lines without usage data or malformed lines
            }
            Err(e) => {
                // Log error but continue processing
                eprintln!("Error reading line {} in {}: {}", 
                    line_num + 1, path.display(), e);
            }
        }
    }
    
    Ok(entries)
}

/// Parse a JSONL file starting from a specific position
pub fn parse_file_from_position(path: &Path, start_position: u64) -> Result<(Vec<UsageEntry>, u64)> {
    let mut file = File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    
    // Get current file size
    let file_size = file.metadata()?.len();
    
    // If start position is beyond file size, file was likely replaced
    if start_position > file_size {
        // Read entire file from beginning
        return parse_file_with_position(path);
    }
    
    // Seek to the start position
    file.seek(SeekFrom::Start(start_position))?;
    
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut current_position = start_position;
    
    // Read lines from the current position
    for line in reader.lines() {
        match line {
            Ok(line_content) => {
                current_position += line_content.len() as u64 + 1; // +1 for newline
                
                if let Some(entry) = parse_line(&line_content) {
                    entries.push(entry);
                }
            }
            Err(e) => {
                eprintln!("Error reading line in {}: {}", path.display(), e);
                break;
            }
        }
    }
    
    Ok((entries, current_position))
}

/// Parse entire file and return entries with final position
pub fn parse_file_with_position(path: &Path) -> Result<(Vec<UsageEntry>, u64)> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;
    
    let file_size = file.metadata()?.len();
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    
    for line in reader.lines() {
        match line {
            Ok(line_content) => {
                if let Some(entry) = parse_line(&line_content) {
                    entries.push(entry);
                }
            }
            Err(e) => {
                eprintln!("Error reading line in {}: {}", path.display(), e);
                break;
            }
        }
    }
    
    Ok((entries, file_size))
}

/// Parse multiple JSONL files and return all entries
pub fn parse_files(paths: &[&Path]) -> Result<Vec<UsageEntry>> {
    let mut all_entries = Vec::new();
    
    for path in paths {
        match parse_file(path) {
            Ok(mut entries) => all_entries.append(&mut entries),
            Err(e) => {
                // Log error but continue with other files
                eprintln!("Error parsing file {}: {}", path.display(), e);
            }
        }
    }
    
    Ok(all_entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_parse_valid_line() {
        let json_line = r#"{
            "timestamp": "2025-01-12T16:03:28.593Z",
            "message": {
                "id": "msg_01QB3q4aPG1gsE54YVH185S9",
                "type": "message",
                "role": "assistant",
                "model": "claude-opus-4-20250514",
                "usage": {
                    "input_tokens": 10,
                    "output_tokens": 7,
                    "cache_creation_input_tokens": 5174,
                    "cache_read_input_tokens": 13568
                }
            },
            "costUSD": 0.0125,
            "requestId": "req_011CR3QAZByoJd2TpJFRxWLf",
            "version": "1.0.51"
        }"#;
        
        let entry = parse_line(json_line).expect("Should parse valid JSON");
        assert_eq!(entry.message.model, "claude-opus-4-20250514");
        assert!(entry.message.usage.is_some());
        
        let usage = entry.message.usage.unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 7);
    }
    
    #[test]
    fn test_parse_line_without_usage() {
        let json_line = r#"{
            "timestamp": "2025-01-12T16:03:28.593Z",
            "message": {
                "id": "msg_01QB3q4aPG1gsE54YVH185S9",
                "type": "message",
                "role": "user",
                "model": "claude-opus-4-20250514"
            },
            "requestId": "req_011CR3QAZByoJd2TpJFRxWLf",
            "version": "1.0.51"
        }"#;
        
        let entry = parse_line(json_line);
        assert!(entry.is_none(), "Should skip entries without usage data");
    }
    
    #[test]
    fn test_parse_malformed_line() {
        let malformed_lines = vec![
            "not json at all",
            "{invalid json",
            "",
            "   ",
            r#"{"partial": "json"#,
        ];
        
        for line in malformed_lines {
            let entry = parse_line(line);
            assert!(entry.is_none(), "Should skip malformed line: {}", line);
        }
    }
    
    #[test]
    fn test_parse_line_with_minimal_usage() {
        // Test that default values work for cache tokens
        let json_line = r#"{
            "timestamp": "2025-01-12T16:03:28.593Z",
            "message": {
                "id": "msg_01QB3q4aPG1gsE54YVH185S9",
                "type": "message",
                "role": "assistant",
                "model": "claude-opus-4-20250514",
                "usage": {
                    "input_tokens": 100,
                    "output_tokens": 50
                }
            },
            "requestId": "req_011CR3QAZByoJd2TpJFRxWLf",
            "version": "1.0.51"
        }"#;
        
        let entry = parse_line(json_line).expect("Should parse JSON with minimal usage");
        let usage = entry.message.usage.unwrap();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cache_creation_input_tokens, 0); // Default value
        assert_eq!(usage.cache_read_input_tokens, 0); // Default value
    }
    
    #[test]
    fn test_parse_file() {
        let test_file = PathBuf::from("test_data/sample.jsonl");
        if test_file.exists() {
            let entries = parse_file(&test_file).expect("Should parse test file");
            
            // Should have 4 valid entries (skipping the user message and malformed line)
            assert_eq!(entries.len(), 4);
            
            // Verify first entry
            assert_eq!(entries[0].message.id, "msg_001");
            assert_eq!(entries[0].message.usage.as_ref().unwrap().input_tokens, 100);
            
            // Verify last entry
            assert_eq!(entries[3].message.id, "msg_005");
            assert_eq!(entries[3].message.usage.as_ref().unwrap().input_tokens, 300);
            
            // Verify different models
            assert_eq!(entries[2].message.model, "claude-sonnet-4-20250514");
        }
    }
}