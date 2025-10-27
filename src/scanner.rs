use crate::types::{Category, DeletableItem};
use crate::utils::{get_project_name, is_cargo_target};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use walkdir::WalkDir;

/// Determine if a path is deletable and return its category
#[inline]
pub fn is_deletable(path: &Path) -> Option<Category> {
    let file_name = path.file_name()?.to_str()?;

    // Check if it's a directory first
    let is_dir = path.is_dir();

    match file_name {
        // Rust
        "target" if is_dir && is_cargo_target(path) => Some(Category::RustTarget),

        // JavaScript/TypeScript
        "node_modules" if is_dir => Some(Category::NodeModules),

        // Python
        "__pycache__" if is_dir => Some(Category::PythonCache),
        ".pytest_cache" if is_dir => Some(Category::PythonCache),
        ".tox" if is_dir => Some(Category::PythonCache),
        "venv" if is_dir => Some(Category::PythonCache),
        ".venv" if is_dir => Some(Category::PythonCache),

        // Java - Maven
        "target" if is_dir && is_maven_target(path) => Some(Category::MavenTarget),

        // Java - Gradle
        "build" if is_dir && is_gradle_build(path) => Some(Category::GradleBuild),
        ".gradle" if is_dir => Some(Category::GradleBuild),

        // PHP - Composer (check BEFORE Go!)
        "vendor" if is_dir && is_composer_vendor(path) => Some(Category::PHPVendor),

        // Go (check AFTER PHP!)
        "vendor" if is_dir && is_go_vendor(path) => Some(Category::GoVendor),

        // C/C++
        "CMakeFiles" if is_dir => Some(Category::CCache),

        // .NET
        "bin" if is_dir && is_dotnet_build(path) => Some(Category::DotNetBuild),
        "obj" if is_dir && is_dotnet_build(path) => Some(Category::DotNetBuild),
        "packages" if is_dir && is_dotnet_packages(path) => Some(Category::DotNetBuild),

        // Swift
        ".build" if is_dir && is_swift_build(path) => Some(Category::SwiftBuild),
        "DerivedData" if is_dir => Some(Category::SwiftBuild),

        // IDE Caches
        ".idea" if is_dir => Some(Category::IDECache),
        ".vscode" if is_dir => Some(Category::IDECache),
        ".vs" if is_dir => Some(Category::IDECache),

        // Ruby - Bundler
        "vendor" if is_dir && is_ruby_bundler(path) => Some(Category::RubyGems),
        ".bundle" if is_dir => Some(Category::RubyGems),

        // OS Junk Files
        ".DS_Store" => Some(Category::OSJunk),
        "Thumbs.db" => Some(Category::OSJunk),
        "desktop.ini" => Some(Category::OSJunk),
        ".localized" => Some(Category::OSJunk),

        // Temp/Cache directories
        ".sass-cache" if is_dir => Some(Category::TempFiles),
        ".parcel-cache" if is_dir => Some(Category::TempFiles),
        ".cache" if is_dir => Some(Category::TempFiles),

        // General build
        "build" if is_dir => Some(Category::BuildCache),
        "dist" if is_dir => Some(Category::BuildCache),
        "out" if is_dir => Some(Category::BuildCache),

        _ => {
            // Python bytecode files
            if file_name.ends_with(".pyc") || file_name.ends_with(".pyo") {
                return Some(Category::PythonCache);
            }

            // C/C++ object files
            if file_name.ends_with(".o") || file_name.ends_with(".a") || file_name == "a.out" {
                return Some(Category::CCache);
            }

            // Log files
            if file_name.ends_with(".log") {
                return Some(Category::TempFiles);
            }

            // Temp files
            if file_name.ends_with(".tmp") || file_name.ends_with(".temp") {
                return Some(Category::TempFiles);
            }

            // Package manager global caches (DANGEROUS!)
            let path_str = path.to_str()?;
            if path_str.contains("/.npm/_cacache") {
                return Some(Category::PackageCache);
            }
            if path_str.contains("/.cache/pip") {
                return Some(Category::PackageCache);
            }
            if path_str.contains("/.cache/yarn") {
                return Some(Category::PackageCache);
            }
            if path_str.contains("/.m2/repository") {
                return Some(Category::PackageCache);
            }

            None
        }
    }
}

/// Check if a path is a Maven target directory
#[inline]
fn is_maven_target(path: &Path) -> bool {
    path.parent()
        .map(|p| p.join("pom.xml").exists())
        .unwrap_or(false)
}

/// Check if a path is a Gradle build directory
#[inline]
fn is_gradle_build(path: &Path) -> bool {
    path.parent()
        .map(|p| p.join("build.gradle").exists() || p.join("build.gradle.kts").exists())
        .unwrap_or(false)
}

/// Check if a path is a PHP Composer vendor directory
#[inline]
fn is_composer_vendor(path: &Path) -> bool {
    path.parent()
        .map(|p| p.join("composer.json").exists())
        .unwrap_or(false)
}

/// Check if a path is a Go vendor directory
#[inline]
fn is_go_vendor(path: &Path) -> bool {
    path.parent()
        .map(|p| p.join("go.mod").exists() || p.join("go.sum").exists())
        .unwrap_or(false)
}

/// Check if a path is a .NET build directory (bin/obj)
#[inline]
fn is_dotnet_build(path: &Path) -> bool {
    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    path.parent()
        .map(|p| {
            // First check: parent must have a .NET project file
            let has_project_file = p.read_dir()
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .any(|e| {
                            e.file_name()
                                .to_str()
                                .map(|s| {
                                    s.ends_with(".csproj")
                                        || s.ends_with(".vbproj")
                                        || s.ends_with(".fsproj")
                                })
                                .unwrap_or(false)
                        })
                })
                .unwrap_or(false);

            if !has_project_file {
                return false;
            }

            // Second check: BOTH bin and obj should exist (typical .NET structure)
            // This prevents false positives with system directories
            if dir_name == "bin" {
                p.join("obj").exists()
            } else if dir_name == "obj" {
                p.join("bin").exists()
            } else {
                false
            }
        })
        .unwrap_or(false)
}

/// Check if a path is a .NET packages directory
#[inline]
fn is_dotnet_packages(path: &Path) -> bool {
    // packages/ directory is typically in solution root
    path.parent()
        .and_then(|p| p.parent())
        .map(|p| {
            p.read_dir()
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .any(|e| {
                            e.file_name()
                                .to_str()
                                .map(|s| s.ends_with(".sln"))
                                .unwrap_or(false)
                        })
                })
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// Check if a path is a Swift build directory
#[inline]
fn is_swift_build(path: &Path) -> bool {
    path.parent()
        .map(|p| p.join("Package.swift").exists())
        .unwrap_or(false)
}

/// Check if a path is a Ruby Bundler vendor directory
#[inline]
fn is_ruby_bundler(path: &Path) -> bool {
    // Check if parent has Gemfile
    path.parent()
        .map(|p| p.join("Gemfile").exists() || p.join("Gemfile.lock").exists())
        .unwrap_or(false)
}

/// Calculate directory size in parallel using all available cores
#[inline]
pub fn calculate_dir_size_parallel(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .par_bridge() // Parallel bridge for iterator
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            if entry.file_type().is_file() {
                entry.metadata().ok().map(|m| m.len())
            } else {
                None
            }
        })
        .sum()
}

/// Scan directory for deletable items with parallel processing
pub fn scan_directory(path: &Path, max_depth: usize, quiet: bool) -> Vec<DeletableItem> {
    let pb = if quiet {
        ProgressBar::hidden()
    } else {
        ProgressBar::new_spinner()
    };

    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} Scanning... [{elapsed_precise}] {pos} items scanned | {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );

    let scanned_count = Arc::new(Mutex::new(0u64));
    let pending_deletables = Arc::new(Mutex::new(Vec::new()));
    let found_count = Arc::new(Mutex::new(0u64));

    // First pass: collect deletable entries with smart filtering
    // Use filter_entry to prevent descending into deletable directories
    for entry in WalkDir::new(path)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            // Don't descend into directories that are themselves deletable
            // (except the root path we're scanning)
            if e.path() == path {
                return true;
            }

            // Check if this entry is deletable
            if let Some(_category) = is_deletable(e.path()) {
                // It's deletable, so we don't want to descend into it
                return false;
            }

            true
        })
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();

        {
            let mut count = scanned_count.lock().unwrap();
            *count += 1;
            if *count % 100 == 0 {
                pb.set_position(*count);
                pb.tick();
            }
        }

        // Check if this entry itself is deletable
        if let Some(category) = is_deletable(entry_path) {
            let mut found = found_count.lock().unwrap();
            *found += 1;

            pending_deletables
                .lock()
                .unwrap()
                .push((entry_path.to_path_buf(), category));

            // Update message with latest find
            if !quiet {
                let file_name = entry_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                pb.set_message(format!("Found {} items | Latest: {} ({})", *found, file_name, category.name()));
            }
        }
    }

    let final_count = *scanned_count.lock().unwrap();
    let total_found = *found_count.lock().unwrap();
    pb.set_position(final_count);
    pb.finish_with_message(format!(
        "✓ Scanned {} items, found {} deletable directories. Calculating sizes...",
        final_count, total_found
    ));

    let pending = pending_deletables.lock().unwrap().clone();

    // Second pass: calculate sizes in parallel using all cores
    let size_pb = Arc::new(Mutex::new(if quiet {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(pending.len() as u64)
    }));

    {
        let pb = size_pb.lock().unwrap();
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
    }

    // Parallel processing with rayon - uses all available cores
    let items: Vec<DeletableItem> = pending
        .par_iter()
        .map(|(item_path, category)| {
            let metadata = fs::metadata(item_path).ok();
            let size = if item_path.is_dir() {
                calculate_dir_size_parallel(item_path)
            } else {
                metadata.as_ref().map(|m| m.len()).unwrap_or(0)
            };

            let last_modified = metadata
                .and_then(|m| m.modified().ok())
                .unwrap_or_else(SystemTime::now);

            let project_name = get_project_name(item_path);

            if !quiet {
                let file_name = item_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                size_pb.lock().unwrap().set_message(format!("Sizing: {}", file_name));
            }
            size_pb.lock().unwrap().inc(1);

            DeletableItem::new(item_path.clone(), size, *category, project_name, last_modified)
        })
        .collect();

    size_pb.lock().unwrap().finish_and_clear();

    items
}

// ============================================================================
// System Indexing Functions (macOS Spotlight)
// ============================================================================

#[cfg(target_os = "macos")]
fn find_with_mdfind(base_path: &Path, query: &str) -> Result<Vec<PathBuf>, String> {
    let output = Command::new("mdfind")
        .arg("-onlyin")
        .arg(base_path)
        .arg(query)
        .output()
        .map_err(|e| format!("Failed to execute mdfind: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "mdfind failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| p.exists())
        .collect())
}

#[cfg(target_os = "macos")]
fn scan_directory_macos(path: &Path, _max_depth: usize, quiet: bool) -> Result<Vec<DeletableItem>, String> {
    let pb = if quiet {
        ProgressBar::hidden()
    } else {
        ProgressBar::new_spinner()
    };

    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} Querying Spotlight index... [{elapsed_precise}] {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );

    // Parallel queries to Spotlight
    let queries = vec![
        ("target", Category::RustTarget),
        ("node_modules", Category::NodeModules),
        ("__pycache__", Category::PythonCache),
        ("build", Category::BuildCache),
        (".gradle", Category::GradleBuild),
        ("venv", Category::PythonCache),
        (".venv", Category::PythonCache),
        ("dist", Category::BuildCache),
        ("vendor", Category::GoVendor), // Will match both PHP and Go, filtered later
        ("CMakeFiles", Category::CCache),
        ("bin", Category::DotNetBuild),
        ("obj", Category::DotNetBuild),
        (".build", Category::SwiftBuild),
        ("DerivedData", Category::SwiftBuild),
        (".idea", Category::IDECache),
        (".vscode", Category::IDECache),
        (".vs", Category::IDECache),
        (".bundle", Category::RubyGems),
        (".DS_Store", Category::OSJunk),
        ("Thumbs.db", Category::OSJunk),
        (".sass-cache", Category::TempFiles),
        (".parcel-cache", Category::TempFiles),
    ];

    let base_path = path.to_path_buf();
    let canonical_base = base_path.canonicalize().unwrap_or_else(|_| base_path.clone());

    let pending_items: Vec<(PathBuf, Category)> = queries
        .par_iter()
        .flat_map(|(name, category)| {
            if !quiet {
                pb.set_message(format!("Finding {}...", name));
                pb.tick();
            }

            find_with_mdfind(&base_path, &format!("kMDItemFSName == '{}'", name))
                .unwrap_or_default()
                .into_iter()
                .filter_map(|p| {
                    // Must be within the search path
                    if !p.starts_with(&base_path) && !p.starts_with(&canonical_base) {
                        return None;
                    }

                    // Must be a directory
                    if !p.is_dir() {
                        return None;
                    }

                    // Get the actual category using is_deletable() for accurate detection
                    let detected_category = is_deletable(&p)?;

                    // Apply category-specific filters
                    match category {
                        Category::RustTarget => {
                            // Check if it's actually a Cargo target directory
                            if detected_category != Category::RustTarget {
                                return None;
                            }
                        }
                        Category::GoVendor => {
                            // For vendor dirs, accept PHP, Go, and Ruby
                            // (they all match "vendor" name in Spotlight)
                            if detected_category != Category::GoVendor
                                && detected_category != Category::PHPVendor
                                && detected_category != Category::RubyGems {
                                return None;
                            }
                        }
                        _ => {
                            // For other categories, just verify they match
                            if detected_category != *category {
                                return None;
                            }
                        }
                    }

                    // Skip if this is inside another deletable directory (avoid nested)
                    // Check parent directories to see if any are deletable
                    if let Some(parent) = p.parent() {
                        let mut current = parent;
                        while current != &base_path && current.starts_with(&base_path) {
                            if is_deletable(current).is_some() {
                                // This is nested inside another deletable dir, skip it
                                return None;
                            }
                            if let Some(next_parent) = current.parent() {
                                current = next_parent;
                            } else {
                                break;
                            }
                        }
                    }

                    // Return with the DETECTED category, not the query category!
                    Some((p, detected_category))
                })
                .collect::<Vec<_>>()
        })
        .collect();

    pb.finish_with_message(format!("✓ Spotlight found {} deletable directories. Calculating sizes...", pending_items.len()));

    // Calculate sizes in parallel
    let size_pb = Arc::new(Mutex::new(if quiet {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(pending_items.len() as u64)
    }));

    {
        let pb = size_pb.lock().unwrap();
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} Calculating sizes...")
                .unwrap()
                .progress_chars("#>-"),
        );
    }

    let items: Vec<DeletableItem> = pending_items
        .par_iter()
        .map(|(item_path, category)| {
            let metadata = fs::metadata(item_path).ok();
            let size = if item_path.is_dir() {
                calculate_dir_size_parallel(item_path)
            } else {
                metadata.as_ref().map(|m| m.len()).unwrap_or(0)
            };

            let last_modified = metadata
                .and_then(|m| m.modified().ok())
                .unwrap_or_else(SystemTime::now);

            let project_name = get_project_name(item_path);

            size_pb.lock().unwrap().inc(1);

            DeletableItem::new(item_path.clone(), size, *category, project_name, last_modified)
        })
        .collect();

    size_pb.lock().unwrap().finish_and_clear();

    Ok(items)
}

#[cfg(target_os = "macos")]
pub fn try_indexed_scan(path: &Path, max_depth: usize, quiet: bool) -> Result<Vec<DeletableItem>, String> {
    // Check if mdfind is available
    if Command::new("mdfind")
        .arg("-version")
        .output()
        .is_err()
    {
        return Err("mdfind not available".to_string());
    }

    scan_directory_macos(path, max_depth, quiet)
}

#[cfg(not(target_os = "macos"))]
pub fn try_indexed_scan(_path: &Path, _max_depth: usize, _quiet: bool) -> Result<Vec<DeletableItem>, String> {
    Err("System indexing only supported on macOS currently".to_string())
}
