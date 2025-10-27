//! # Types Module
//!
//! Core data structures used throughout Gigabroom.
//!
//! This module defines the primary types for representing deletable items,
//! categories of build artifacts, and scan cache data.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// Represents a deletable build artifact or cache directory.
///
/// Contains all information needed to identify, categorize, and display
/// a potential deletion target.
///
/// # Examples
///
/// ```
/// use gigabroom::types::{DeletableItem, Category};
/// use std::path::PathBuf;
/// use std::time::SystemTime;
///
/// let item = DeletableItem::new(
///     PathBuf::from("/project/target"),
///     1048576,  // 1 MB
///     Category::RustTarget,
///     String::from("my-project"),
///     SystemTime::now()
/// );
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeletableItem {
    /// Filesystem path to the item
    pub path: PathBuf,
    /// Size in bytes
    pub size: u64,
    /// Category classification
    pub category: Category,
    /// Parent project or directory name
    pub project_name: String,
    /// Last modification time
    pub last_modified: SystemTime,
}

/// Categories of build artifacts and caches that Gigabroom can detect.
///
/// Each variant represents a specific type of deletable item from various
/// programming languages, build tools, or system caches.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
pub enum Category {
    // Rust
    RustTarget,

    // JavaScript/TypeScript
    NodeModules,

    // Python
    PythonCache,

    // PHP
    PHPVendor,

    // Ruby
    RubyGems,

    // Java
    MavenTarget,
    GradleBuild,

    // Go
    GoVendor,

    // C/C++
    CCache,

    // .NET
    DotNetBuild,

    // Swift
    SwiftBuild,

    // IDE Caches
    IDECache,

    // OS Junk
    OSJunk,

    // Temp/Logs
    TempFiles,

    // Package Manager Caches (DANGEROUS)
    PackageCache,

    // General
    BuildCache,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanCache {
    pub scan_path: PathBuf,
    pub scan_time: SystemTime,
    pub items: Vec<DeletableItem>,
    pub max_depth: usize,
}

impl Category {
    pub const fn name(&self) -> &'static str {
        match self {
            Category::RustTarget => "Rust target",
            Category::NodeModules => "Node modules",
            Category::PythonCache => "Python cache",
            Category::PHPVendor => "PHP vendor",
            Category::RubyGems => "Ruby gems",
            Category::MavenTarget => "Maven target",
            Category::GradleBuild => "Gradle build",
            Category::GoVendor => "Go vendor",
            Category::CCache => "C/C++ cache",
            Category::DotNetBuild => ".NET build",
            Category::SwiftBuild => "Swift build",
            Category::IDECache => "IDE cache",
            Category::OSJunk => "OS junk",
            Category::TempFiles => "Temp/log files",
            Category::PackageCache => "Package cache",
            Category::BuildCache => "Build cache",
        }
    }

    pub const fn all() -> &'static [Category] {
        &[
            Category::RustTarget,
            Category::NodeModules,
            Category::PythonCache,
            Category::PHPVendor,
            Category::RubyGems,
            Category::MavenTarget,
            Category::GradleBuild,
            Category::GoVendor,
            Category::CCache,
            Category::DotNetBuild,
            Category::SwiftBuild,
            Category::IDECache,
            Category::OSJunk,
            Category::TempFiles,
            Category::PackageCache,
            Category::BuildCache,
        ]
    }

    /// Returns true if this category is dangerous to delete (loses data)
    pub const fn is_dangerous(&self) -> bool {
        matches!(self, Category::PackageCache)
    }
}

impl DeletableItem {
    #[inline]
    pub fn new(path: PathBuf, size: u64, category: Category, project_name: String, last_modified: SystemTime) -> Self {
        Self {
            path,
            size,
            category,
            project_name,
            last_modified,
        }
    }
}
