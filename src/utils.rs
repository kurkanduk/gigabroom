use std::path::{Path, PathBuf};
use std::env;

/// Macro for conditional printing based on quiet flag
#[macro_export]
macro_rules! println_unless_quiet {
    ($quiet:expr, $($arg:tt)*) => {
        if !$quiet {
            println!($($arg)*);
        }
    };
}

/// Macro for printing errors with consistent formatting
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        eprintln!("{} {}", "Error:".bright_red().bold(), format!($($arg)*));
    };
}

/// Macro for printing success messages
#[macro_export]
macro_rules! print_success {
    ($($arg:tt)*) => {
        println!("{}", format!($($arg)*).green().bold());
    };
}

/// Macro for printing warnings
#[macro_export]
macro_rules! print_warning {
    ($($arg:tt)*) => {
        println!("{}", format!($($arg)*).yellow());
    };
}

/// Macro for printing info messages
#[macro_export]
macro_rules! print_info {
    ($label:expr, $value:expr) => {
        println!("{}: {}", $label.bright_white(), $value.to_string().bright_yellow());
    };
}

/// Format bytes into human-readable size
#[inline]
pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    match size {
        s if s >= TB => format!("{:.2} TB", s as f64 / TB as f64),
        s if s >= GB => format!("{:.2} GB", s as f64 / GB as f64),
        s if s >= MB => format!("{:.2} MB", s as f64 / MB as f64),
        s if s >= KB => format!("{:.2} KB", s as f64 / KB as f64),
        s => format!("{} B", s),
    }
}

/// Parse size string (e.g., "100MB", "1GB") to bytes
pub fn parse_size(size_str: &str) -> Result<u64, String> {
    let size_str = size_str.trim().to_uppercase();

    let (num_str, multiplier) = if let Some(num) = size_str.strip_suffix("TB") {
        (num, 1024_u64.pow(4))
    } else if let Some(num) = size_str.strip_suffix("GB") {
        (num, 1024_u64.pow(3))
    } else if let Some(num) = size_str.strip_suffix("MB") {
        (num, 1024_u64.pow(2))
    } else if let Some(num) = size_str.strip_suffix("KB") {
        (num, 1024)
    } else if let Some(num) = size_str.strip_suffix('B') {
        (num, 1)
    } else {
        // Assume bytes if no suffix
        (size_str.as_str(), 1)
    };

    num_str
        .trim()
        .parse::<u64>()
        .map(|n| n.saturating_mul(multiplier))
        .map_err(|_| format!("Invalid size format: {}", size_str))
}

/// Get project name from path
#[inline]
pub fn get_project_name(path: &Path) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(String::from)
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Check if a path is a Cargo target directory
#[inline]
pub fn is_cargo_target(path: &Path) -> bool {
    path.parent()
        .map(|p| p.join("Cargo.toml").exists())
        .unwrap_or(false)
}

/// Expand tilde (~) in path to home directory and handle escaped spaces
pub fn expand_tilde(path: &str) -> PathBuf {
    // Remove escape characters (backslashes before spaces)
    let cleaned_path = path.replace("\\ ", " ");

    if cleaned_path.starts_with("~/") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(&cleaned_path[2..]);
        }
    } else if cleaned_path == "~" {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(cleaned_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("1TB").unwrap(), 1024_u64.pow(4));
        assert_eq!(parse_size("100mb").unwrap(), 100 * 1024 * 1024);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1536 * 1024 * 1024), "1.50 GB");
    }

    #[test]
    fn test_expand_tilde() {
        // Test tilde expansion
        let expanded = expand_tilde("~/test");
        assert!(expanded.to_string_lossy().contains("test"));
        assert!(!expanded.to_string_lossy().starts_with("~"));

        // Test tilde alone
        let expanded_home = expand_tilde("~");
        assert!(!expanded_home.to_string_lossy().is_empty());

        // Test non-tilde path
        let normal = expand_tilde("/usr/local/bin");
        assert_eq!(normal.to_string_lossy(), "/usr/local/bin");
    }
}
