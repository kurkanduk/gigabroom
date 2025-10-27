use crate::types::Category;
use clap::{Parser, Subcommand, ValueEnum};

/// Gigabroom 🧹 - Sweep away gigabytes of build artifacts
#[derive(Parser, Debug)]
#[command(name = "gigabroom")]
#[command(version, author)]
#[command(about = "🧹 Sweep away gigabytes of build artifacts - the ultimate disk space cleaner for developers", long_about = None)]
#[command(after_help = "EXAMPLES:\n  \
    gigabroom                                  # Launch interactive menu\n  \
    gigabroom scan                             # Scan current directory\n  \
    gigabroom scan ~/projects -d 5             # Scan with max depth 5\n  \
    gigabroom clean --category rust node       # Clean Rust and Node artifacts\n  \
    gigabroom clean --all --yes                # Clean everything without confirmation\n  \
    gigabroom clean --dry-run                  # Preview what would be deleted\n  \
    gigabroom cache clear                      # Clear the scan cache")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan a directory for deletable items
    Scan {
        /// Directory to scan (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Maximum depth to scan
        #[arg(short = 'd', long, default_value = "10")]
        max_depth: usize,

        /// Force fresh scan, ignore cache
        #[arg(short, long)]
        force: bool,

        /// Use system indexing (Spotlight on macOS) - faster but may miss items
        #[arg(short, long)]
        index: bool,

        /// Minimum size threshold (e.g., "100MB", "1GB")
        #[arg(short = 's', long)]
        min_size: Option<String>,

        /// Only show items older than (e.g., "30d", "1w", "7d")
        #[arg(short = 'o', long)]
        older_than: Option<String>,

        /// Output results as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Clean (delete) build artifacts and caches
    Clean {
        /// Directory to clean (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Maximum depth to scan
        #[arg(short = 'd', long, default_value = "10")]
        max_depth: usize,

        /// Language/category to clean: rust, node, python, java-maven, java-gradle, build, git, cargo
        #[arg(short, long, value_enum)]
        category: Vec<CategoryFilter>,

        /// Clean all categories
        #[arg(short, long)]
        all: bool,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,

        /// Preview what would be deleted without actually deleting
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Force fresh scan, ignore cache
        #[arg(short, long)]
        force: bool,

        /// Use system indexing - faster but may miss items
        #[arg(short, long)]
        index: bool,

        /// Minimum size threshold (e.g., "100MB", "1GB")
        #[arg(short = 's', long)]
        min_size: Option<String>,

        /// Only show items older than (e.g., "30d", "1w", "7d")
        #[arg(short = 'o', long)]
        older_than: Option<String>,

        /// Output results as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Manage scan cache
    Cache {
        #[command(subcommand)]
        action: CacheCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// Clear the scan cache
    Clear,

    /// Show cache information
    Info,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum CategoryFilter {
    /// Rust target directories
    Rust,
    /// Node.js node_modules
    Node,
    /// Python cache and virtual environments
    Python,
    /// PHP Composer vendor
    PHP,
    /// Ruby Bundler gems
    Ruby,
    /// Java Maven target directories
    JavaMaven,
    /// Java Gradle build directories
    JavaGradle,
    /// Go vendor directories
    Go,
    /// C/C++ build artifacts
    CCache,
    /// .NET bin/obj/packages
    DotNet,
    /// Swift .build/DerivedData
    Swift,
    /// IDE caches (.idea, .vscode, .vs)
    IDE,
    /// OS junk files (.DS_Store, Thumbs.db)
    OSJunk,
    /// Temp and log files
    Temp,
    /// Package manager global caches (DANGEROUS)
    PackageCache,
    /// General build/dist/out directories
    Build,
}

impl CategoryFilter {
    pub const fn to_category(&self) -> Category {
        match self {
            CategoryFilter::Rust => Category::RustTarget,
            CategoryFilter::Node => Category::NodeModules,
            CategoryFilter::Python => Category::PythonCache,
            CategoryFilter::PHP => Category::PHPVendor,
            CategoryFilter::Ruby => Category::RubyGems,
            CategoryFilter::JavaMaven => Category::MavenTarget,
            CategoryFilter::JavaGradle => Category::GradleBuild,
            CategoryFilter::Go => Category::GoVendor,
            CategoryFilter::CCache => Category::CCache,
            CategoryFilter::DotNet => Category::DotNetBuild,
            CategoryFilter::Swift => Category::SwiftBuild,
            CategoryFilter::IDE => Category::IDECache,
            CategoryFilter::OSJunk => Category::OSJunk,
            CategoryFilter::Temp => Category::TempFiles,
            CategoryFilter::PackageCache => Category::PackageCache,
            CategoryFilter::Build => Category::BuildCache,
        }
    }
}
