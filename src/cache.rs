//! # Cache Module
//!
//! Provides scan result caching functionality to improve performance
//! when re-scanning the same directories.
//!
//! ## Features
//!
//! - Caches scan results for 5 minutes
//! - Validates cache against scan parameters
//! - Filters out non-existent items
//! - Stores cache in user's home directory

use crate::types::{DeletableItem, ScanCache};
use crate::utils::format_size;
use colored::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cache validity duration in seconds (5 minutes)
const CACHE_VALIDITY_SECONDS: u64 = 300;

/// Returns the path to the cache file in the user's home directory.
///
/// The cache file is stored at `~/.gigabroom-cache.json` on Unix-like systems
/// or `%USERPROFILE%\.gigabroom-cache.json` on Windows.
///
/// # Returns
///
/// A `PathBuf` pointing to the cache file location. Falls back to current
/// directory if home directory cannot be determined.
///
/// # Examples
///
/// ```
/// use gigabroom::cache::get_cache_path;
///
/// let cache_path = get_cache_path();
/// assert!(cache_path.ends_with(".gigabroom-cache.json"));
/// ```
#[inline]
pub fn get_cache_path() -> PathBuf {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".gigabroom-cache.json")
}

/// Loads cached scan results if valid and matching scan parameters.
///
/// The cache is considered valid if:
/// - It exists and is readable
/// - Scan path and max depth match
/// - Cache is less than 5 minutes old
/// - Cached items still exist on filesystem
///
/// # Arguments
///
/// * `scan_path` - The directory path that was scanned
/// * `max_depth` - The maximum recursion depth used
///
/// # Returns
///
/// `Some(Vec<DeletableItem>)` if cache is valid, `None` otherwise
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use gigabroom::cache::load_cache;
///
/// let items = load_cache(Path::new("/home/user/projects"), 10);
/// if let Some(cached) = items {
///     println!("Loaded {} items from cache", cached.len());
/// }
/// ```
pub fn load_cache(scan_path: &Path, max_depth: usize) -> Option<Vec<DeletableItem>> {
    let cache_path = get_cache_path();

    if !cache_path.exists() {
        return None;
    }

    let cache_data = fs::read_to_string(&cache_path).ok()?;
    let cache: ScanCache = serde_json::from_str(&cache_data).ok()?;

    // Validate cache matches current scan parameters
    if cache.scan_path != scan_path || cache.max_depth != max_depth {
        return None;
    }

    // Check if cache is still valid (< 5 minutes old)
    if let Ok(elapsed) = cache.scan_time.elapsed() {
        if elapsed.as_secs() > CACHE_VALIDITY_SECONDS {
            return None;
        }
    }

    // Filter out items that no longer exist
    let valid_items: Vec<DeletableItem> = cache
        .items
        .into_iter()
        .filter(|item| item.path.exists())
        .collect();

    Some(valid_items)
}

/// Saves scan results to cache for future use.
///
/// Creates a JSON cache file in the user's home directory containing
/// the scan path, max depth, timestamp, and found items.
///
/// # Arguments
///
/// * `scan_path` - The directory path that was scanned
/// * `max_depth` - The maximum recursion depth used
/// * `items` - The scan results to cache
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use gigabroom::cache::save_cache;
///
/// let items = vec![]; // Your scan results
/// save_cache(Path::new("/home/user/projects"), 10, &items);
/// ```
pub fn save_cache(scan_path: &Path, max_depth: usize, items: &[DeletableItem]) {
    let cache = ScanCache {
        scan_path: scan_path.to_path_buf(),
        scan_time: SystemTime::now(),
        items: items.to_vec(),
        max_depth,
    };

    if let Ok(json) = serde_json::to_string(&cache) {
        let cache_path = get_cache_path();
        let _ = fs::write(cache_path, json);
    }
}

/// Deletes the cache file from the filesystem.
///
/// Silently succeeds even if the cache file doesn't exist.
///
/// # Examples
///
/// ```no_run
/// use gigabroom::cache::clear_cache;
///
/// clear_cache();
/// println!("Cache cleared");
/// ```
pub fn clear_cache() {
    let cache_path = get_cache_path();
    let _ = fs::remove_file(cache_path);
}

/// Displays detailed information about the cache.
///
/// Shows cache location, size, age, scan parameters, and validity status.
/// Outputs formatted information to stdout using colored text.
///
/// # Examples
///
/// ```no_run
/// use gigabroom::cache::show_cache_info();
///
/// show_cache_info();
/// // Prints cache details to stdout
/// ```
pub fn show_cache_info() {
    let cache_path = get_cache_path();

    if !cache_path.exists() {
        println!("{}", "No cache file found.".yellow());
        return;
    }

    match fs::metadata(&cache_path) {
        Ok(metadata) => {
            println!("{}", "Cache Information:".bright_cyan().bold());
            println!("  {}: {}", "Location".bright_white(), cache_path.display());
            println!(
                "  {}: {}",
                "Size".bright_white(),
                format_size(metadata.len())
            );

            if let Ok(cache_data) = fs::read_to_string(&cache_path) {
                if let Ok(cache) = serde_json::from_str::<ScanCache>(&cache_data) {
                    println!(
                        "  {}: {}",
                        "Scan path".bright_white(),
                        cache.scan_path.display()
                    );
                    println!("  {}: {}", "Max depth".bright_white(), cache.max_depth);
                    println!("  {}: {}", "Items cached".bright_white(), cache.items.len());

                    if let Ok(elapsed) = cache.scan_time.elapsed() {
                        let secs = elapsed.as_secs();
                        let age = match secs {
                            s if s < 60 => format!("{} seconds", s),
                            s if s < 3600 => format!("{} minutes", s / 60),
                            s if s < 86400 => format!("{} hours", s / 3600),
                            s => format!("{} days", s / 86400),
                        };
                        println!("  {}: {} ago", "Cache age".bright_white(), age);

                        if secs > CACHE_VALIDITY_SECONDS {
                            println!(
                                "  {}",
                                format!("Cache is stale (>{} minutes old)", CACHE_VALIDITY_SECONDS / 60)
                                    .yellow()
                            );
                        } else {
                            println!("  {}", "Cache is fresh".green());
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{} {}", "Error reading cache:".red(), e);
        }
    }
}
