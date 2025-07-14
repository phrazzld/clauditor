mod types;
mod parser;
mod window;
mod scanner;
mod coordinator;
mod display;
mod watcher;
mod position_tracker;

use anyhow::Result;
use clap::Parser;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;

/// Multi-session Claude Code usage tracker
#[derive(Parser)]
#[command(name = "clauditor")]
#[command(version)]
#[command(about = "Track active Claude Code billing windows across multiple sessions", long_about = None)]
struct Cli {}

fn main() -> Result<()> {
    // Parse command line arguments
    let _cli = Cli::parse();
    
    // Create persistent scanner with position tracking
    let mut scanner = scanner::SessionScanner::new();
    let mut current_window: Option<types::SessionBlock> = None;
    
    // Set up file watcher
    let file_watcher = match watcher::SessionWatcher::with_default_paths() {
        Ok(w) => Some(w),
        Err(e) => {
            eprintln!("Warning: Could not set up file watching: {}", e);
            eprintln!("Will rely on periodic refresh only");
            None
        }
    };
    
    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // Track if we need immediate refresh
    let mut needs_refresh = true;
    let mut needs_full_reload = true;
    
    // Main loop
    while running.load(Ordering::SeqCst) {
        // Check for file events
        if let Some(ref watcher) = file_watcher {
            let events = watcher.poll_events();
            if !events.is_empty() {
                // Files changed, use incremental loading
                needs_refresh = true;
                
                // Load data incrementally (or full reload if active window detected)
                match coordinator::load_and_group_sessions_incremental(&mut scanner) {
                    Ok(new_window_opt) => {
                        // The coordinator now returns complete window data when active
                        // No need for complex merging - just replace
                        current_window = new_window_opt;
                    }
                    Err(e) => {
                        eprintln!("Error loading incremental sessions: {}", e);
                    }
                }
            }
        }
        
        if needs_refresh || needs_full_reload {
            // Clear screen
            print!("\x1B[2J\x1B[1;1H");
            
            if needs_full_reload {
                // Full reload on first run or periodic refresh
                match coordinator::get_active_billing_window() {
                    Ok(window) => {
                        current_window = window;
                        display::display_active_window(current_window.as_ref());
                    }
                    Err(e) => {
                        eprintln!("Error loading sessions: {}", e);
                    }
                }
                needs_full_reload = false;
            } else {
                // Just display current window if active
                if let Some(ref mut window) = current_window {
                    window.is_active = types::is_block_active(window, chrono::Utc::now());
                }
                display::display_active_window(current_window.as_ref().filter(|w| w.is_active));
            }
            
            needs_refresh = false;
        }
        
        // Sleep briefly to avoid busy waiting
        thread::sleep(Duration::from_millis(100));
        
        // Force full reload every 5 seconds
        static LAST_REFRESH: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last = LAST_REFRESH.load(Ordering::Relaxed);
        if now - last >= 5 {
            needs_full_reload = true;
            LAST_REFRESH.store(now, Ordering::Relaxed);
        }
    }
    
    println!("\nShutting down...");
    Ok(())
}