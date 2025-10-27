mod cache;
mod cleaner;
mod cli;
mod display;
mod menu;
mod scanner;
mod types;
mod ui;
mod utils;

use cache::{clear_cache, load_cache, save_cache, show_cache_info};
use cleaner::{confirm_deletion, delete_items, select_categories, show_interactive_menu};
use cli::{CacheCommands, Cli, Commands};
use clap::Parser;
use colored::*;
use display::{display_scan_results, print_header};
use menu::run_interactive_menu;
use scanner::{scan_directory, try_indexed_scan};
use std::path::Path;
use types::{Category, DeletableItem};
use utils::{expand_tilde, parse_size};

/// Perform a scan with caching logic
fn perform_scan(
    path: &Path,
    max_depth: usize,
    force: bool,
    use_index: bool,
    quiet: bool,
) -> Vec<DeletableItem> {
    // Use indexing only if explicitly enabled (Spotlight can be unreliable)
    let should_use_index = use_index;

    if force {
        println_unless_quiet!(quiet, "{}", "Forcing fresh scan (cache ignored)...".yellow());

        let items = if should_use_index {
            match try_indexed_scan(path, max_depth, quiet) {
                Ok(items) => {
                    println_unless_quiet!(quiet, "{}", "✓ Used Spotlight indexing".green());
                    println_unless_quiet!(quiet, "{}", "  Finds ALL directories (ignores depth limit)".dimmed());
                    println_unless_quiet!(quiet, "{}", "  Note: May miss very recently created files".dimmed());
                    items
                }
                Err(e) => {
                    println_unless_quiet!(quiet, "{} {}", "⚠ Spotlight failed:".yellow(), e);
                    println_unless_quiet!(quiet, "{}", "→ Using filesystem walk (respects depth)...".yellow());
                    scan_directory(path, max_depth, quiet)
                }
            }
        } else {
            scan_directory(path, max_depth, quiet)
        };

        save_cache(path, max_depth, &items);
        println_unless_quiet!(quiet, "{}", "Scan results cached for future use".dimmed());
        items
    } else if let Some(cached_items) = load_cache(path, max_depth) {
        println_unless_quiet!(
            quiet,
            "{}",
            "Using cached scan results (less than 5 minutes old)".green()
        );
        println_unless_quiet!(
            quiet,
            "{} {} items\n",
            "Loaded:".bright_green(),
            cached_items.len()
        );
        cached_items
    } else {
        println_unless_quiet!(quiet, "{}", "Performing fresh scan...".yellow());

        let items = if should_use_index {
            match try_indexed_scan(path, max_depth, quiet) {
                Ok(items) => {
                    println_unless_quiet!(quiet, "{}", "✓ Used Spotlight indexing".green());
                    println_unless_quiet!(quiet, "{}", "  Finds ALL directories (ignores depth limit)".dimmed());
                    println_unless_quiet!(quiet, "{}", "  Note: May miss very recently created files".dimmed());
                    items
                }
                Err(e) => {
                    println_unless_quiet!(quiet, "{} {}", "⚠ Spotlight failed:".yellow(), e);
                    println_unless_quiet!(quiet, "{}", "→ Using filesystem walk (respects depth)...".yellow());
                    scan_directory(path, max_depth, quiet)
                }
            }
        } else {
            scan_directory(path, max_depth, quiet)
        };

        save_cache(path, max_depth, &items);
        println_unless_quiet!(quiet, "{}", "Scan results cached for future use".dimmed());
        items
    }
}

/// Handle scan command - returns items for potential cleanup
fn handle_scan(
    path: String,
    max_depth: usize,
    force: bool,
    index: bool,
    min_size: Option<String>,
    _older_than: Option<String>,
    json: bool,
    quiet: bool,
    verbose: bool,
    from_interactive_menu: bool,
) -> Vec<DeletableItem> {
    let expanded_path = expand_tilde(&path);
    let scan_path = expanded_path.as_path();

    if !scan_path.exists() {
        ui::show_error(
            "Path Not Found",
            &format!("The specified path does not exist: {}", path),
            &[
                "Check if the path is typed correctly",
                "Use an absolute path (e.g., /Users/name/projects)",
                "Try using '.' for the current directory",
            ],
        );
        std::process::exit(1);
    }

    if !scan_path.is_dir() {
        ui::show_error(
            "Invalid Path Type",
            &format!("The path is not a directory: {}", path),
            &[
                "Provide a directory path, not a file",
                "Use the parent directory instead",
            ],
        );
        std::process::exit(1);
    }

    // Only print header when not in interactive menu (command-line mode)
    if !from_interactive_menu {
        print_header(quiet, json);
    }

    if !quiet && !json {
        println!();
        print_info!("Scanning", scan_path.display());
        print_info!("Max depth", max_depth);
        println!();
    }

    let mut items = perform_scan(scan_path, max_depth, force, index, quiet);

    // Apply size filter if specified
    if let Some(min_size_str) = min_size {
        match parse_size(&min_size_str) {
            Ok(min_size_bytes) => {
                items.retain(|item| item.size >= min_size_bytes);
                println_unless_quiet!(
                    quiet || json,
                    "{} {}",
                    "Filtered by minimum size:".dimmed(),
                    min_size_str
                );
            }
            Err(e) => {
                ui::show_error(
                    "Invalid Size Format",
                    &format!("Could not parse minimum size: {}", e),
                    &[
                        "Use format like: 100MB, 1GB, 500KB",
                        "Examples: --min-size 100MB or --min-size 1GB",
                        "Make sure there's no space between number and unit",
                    ],
                );
                std::process::exit(1);
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&items).unwrap());
    } else {
        display_scan_results(&items, verbose, quiet, from_interactive_menu);
        // Statistics and disk space are now integrated into the grouped view
    }

    items
}

/// Handle clean command
#[allow(clippy::too_many_arguments)]
fn handle_clean(
    path: String,
    max_depth: usize,
    category: Vec<cli::CategoryFilter>,
    all: bool,
    yes: bool,
    dry_run: bool,
    force: bool,
    index: bool,
    min_size: Option<String>,
    _older_than: Option<String>,
    json: bool,
    quiet: bool,
) {
    let expanded_path = expand_tilde(&path);
    let clean_path = expanded_path.as_path();

    if !clean_path.exists() {
        ui::show_error(
            "Path Not Found",
            &format!("The specified path does not exist: {}", path),
            &[
                "Check if the path is typed correctly",
                "Use an absolute path (e.g., /Users/name/projects)",
                "Try using '.' for the current directory",
            ],
        );
        std::process::exit(1);
    }

    if !clean_path.is_dir() {
        ui::show_error(
            "Invalid Path Type",
            &format!("The path is not a directory: {}", path),
            &[
                "Provide a directory path, not a file",
                "Use the parent directory instead",
            ],
        );
        std::process::exit(1);
    }

    print_header(quiet, json);

    if !quiet && !json {
        print_info!("Scanning", clean_path.display());
        print_info!("Max depth", max_depth);
        println!();
    }

    let mut all_items = perform_scan(clean_path, max_depth, force, index, quiet || json);

    // Apply size filter if specified
    if let Some(min_size_str) = min_size {
        match parse_size(&min_size_str) {
            Ok(min_size_bytes) => {
                all_items.retain(|item| item.size >= min_size_bytes);
                println_unless_quiet!(
                    quiet || json,
                    "{} {}",
                    "Filtered by minimum size:".dimmed(),
                    min_size_str
                );
            }
            Err(e) => {
                ui::show_error(
                    "Invalid Size Format",
                    &format!("Could not parse minimum size: {}", e),
                    &[
                        "Use format like: 100MB, 1GB, 500KB",
                        "Examples: --min-size 100MB or --min-size 1GB",
                        "Make sure there's no space between number and unit",
                    ],
                );
                std::process::exit(1);
            }
        }
    }

    // Determine which categories to clean
    let selected_categories: Vec<Category> = if all {
        Category::all().to_vec()
    } else if !category.is_empty() {
        category.iter().map(|c| c.to_category()).collect()
    } else if yes {
        ui::show_error(
            "Missing Required Option",
            "When using --yes (non-interactive mode), you must specify which categories to clean",
            &[
                "Use --category rust,node,python (comma-separated list)",
                "Or use --all to clean all categories",
                "Example: rust-cleaner clean --yes --category rust,node",
            ],
        );
        std::process::exit(1);
    } else {
        select_categories()
    };

    if selected_categories.is_empty() {
        println_unless_quiet!(quiet, "\n{}", "No categories selected. Exiting...".yellow());
        return;
    }

    // Filter items by selected categories
    let filtered_items: Vec<DeletableItem> = all_items
        .into_iter()
        .filter(|item| selected_categories.contains(&item.category))
        .collect();

    if filtered_items.is_empty() {
        println_unless_quiet!(
            quiet || json,
            "\n{}",
            "No items found for selected categories.".yellow()
        );
        return;
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&filtered_items).unwrap());
        return;
    }

    // Select items to delete
    let selections = if yes {
        (0..filtered_items.len()).collect()
    } else {
        show_interactive_menu(&filtered_items)
    };

    if selections.is_empty() {
        println_unless_quiet!(quiet, "\n{}", "No items selected for deletion.".yellow());
        return;
    }

    // Confirm deletion if not in yes mode
    if !yes && !dry_run && !quiet {
        let total_size: u64 = selections
            .iter()
            .filter_map(|&i| filtered_items.get(i))
            .map(|item| item.size)
            .sum();

        if !confirm_deletion(selections.len(), total_size) {
            println!("{}", "Cancelled.".yellow());
            return;
        }
    }

    let items_deleted = delete_items(&filtered_items, &selections, dry_run, quiet);

    if items_deleted {
        clear_cache();
        println_unless_quiet!(quiet, "\n{}", "Cache cleared.".dimmed());
    }
}
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan {
            path,
            max_depth,
            force,
            index,
            min_size,
            older_than,
            json,
        }) => {
            handle_scan(path, max_depth, force, index, min_size, older_than, json, cli.quiet, cli.verbose, false);
        }

        Some(Commands::Clean {
            path,
            max_depth,
            category,
            all,
            yes,
            dry_run,
            force,
            index,
            min_size,
            older_than,
            json,
        }) => handle_clean(
            path, max_depth, category, all, yes, dry_run, force, index, min_size, older_than, json, cli.quiet,
        ),

        Some(Commands::Cache { action }) => match action {
            CacheCommands::Clear => {
                clear_cache();
                println_unless_quiet!(cli.quiet, "{}", "Cache cleared successfully.".green());
            }
            CacheCommands::Info => {
                show_cache_info();
            }
        },

        None => {
            print_header(cli.quiet, false);
            if !cli.quiet {
                run_interactive_menu(handle_scan);
            } else {
                println_unless_quiet!(
                    cli.quiet,
                    "\n{}",
                    "No command specified. Use --help for usage information.".yellow()
                );
            }
        }
    }
}
