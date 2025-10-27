/// Interactive menu system
use crate::cache::{clear_cache, show_cache_info};
use crate::cleaner::{confirm_deletion, delete_items, show_interactive_menu};
use crate::display::print_header;
use crate::types::DeletableItem;
use crate::utils::{expand_tilde, format_size};
use crate::ui;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

/// Run the main interactive menu loop
pub fn run_interactive_menu(
    handle_scan_fn: impl Fn(String, usize, bool, bool, Option<String>, Option<String>, bool, bool, bool, bool) -> Vec<DeletableItem>
) {
    loop {
        ui::clear_screen();
        print_header(false, false);
        println!("\n{}", "=".repeat(80).bright_black());
        ui::show_inline_hint();

        let options = vec![
            "📊 Scan & Clean",
            "💾 Cache Management",
            "❓ Help & Keyboard Shortcuts",
            "❌ Exit",
        ];

        let selection = match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Main Menu - Select an option")
            .items(&options)
            .default(0)
            .interact()
        {
            Ok(idx) => idx,
            Err(_) => {
                // ESC pressed - exit gracefully
                println!("\n{}", "Goodbye!".bright_green());
                break;
            }
        };

        match selection {
            0 => menu_scan(&handle_scan_fn),
            1 => menu_cache(),
            2 => menu_help(),
            3 => {
                println!("\n{}", "Goodbye!".bright_green());
                break;
            }
            _ => break,
        }
    }
}

/// Interactive scan menu
fn menu_scan<F>(handle_scan_fn: &F)
where
    F: Fn(String, usize, bool, bool, Option<String>, Option<String>, bool, bool, bool, bool) -> Vec<DeletableItem>
{
    ui::clear_screen();
    ui::show_breadcrumb(&["Main Menu", "Scan & Clean"]);
    println!("\n{}", "Scan & Clean Build Artifacts".bright_cyan().bold());

    let path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Directory to scan")
        .default(".".to_string())
        .interact_text()
        .unwrap_or_else(|_| ".".to_string());

    let max_depth: usize = loop {
        match Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Maximum depth")
            .default("10".to_string())
            .interact_text()
        {
            Ok(input) => {
                match input.trim().parse::<usize>() {
                    Ok(depth) if depth > 0 && depth <= 100 => break depth,
                    Ok(_) => {
                        println!("{}", "Please enter a depth between 1 and 100".yellow());
                        continue;
                    }
                    Err(_) => {
                        println!("{}", "Please enter a valid number".yellow());
                        continue;
                    }
                }
            }
            Err(_) => break 10,
        }
    };

    let use_index = if cfg!(target_os = "macos") {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Use Spotlight? (fast, finds ALL dirs, ignores depth)")
            .default(true)
            .interact()
            .unwrap_or(true)
    } else {
        false
    };

    let force = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Force fresh scan (ignore cache)?")
        .default(false)
        .interact()
        .unwrap_or(false);

    let min_size_input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Minimum size filter (e.g., '100MB', '1GB', or press Enter to skip)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();

    let min_size = if min_size_input.trim().is_empty() {
        None
    } else {
        Some(min_size_input)
    };

    let verbose = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Show file paths (verbose)?")
        .default(false)
        .interact()
        .unwrap_or(false);

    println!();

    // Expand tilde in path
    let expanded_path = expand_tilde(&path);
    let path_str = expanded_path.to_string_lossy().to_string();

    let items = handle_scan_fn(path_str, max_depth, force, use_index, min_size, None, false, false, verbose, true);

    // If no items found, show message and wait
    if items.is_empty() {
        println!("\n{}", "[Press Enter or ESC to return to main menu]".dimmed());
        let _ = Input::<String>::new().allow_empty(true).interact();
        return;
    }

    // Items found, offer deletion
    if !items.is_empty() {
        println!();

        let want_delete = match Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to delete some of these items?")
            .default(true)
            .interact()
        {
            Ok(result) => result,
            Err(_) => {
                // ESC pressed - return to main menu
                return;
            }
        };

        if want_delete {
            ui::clear_screen();

            // Go directly to item selection (user already saw categories in scan results)
            let selections = if items.len() == 1 {
                // Only one item, ask directly
                let item = &items[0];
                println!("\n{}", "Found one item:".bright_cyan().bold());
                println!("  {} - {} ({})",
                    item.project_name.bright_yellow(),
                    format_size(item.size).bright_green(),
                    item.category.name().dimmed()
                );

                let confirm = match Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Delete this item?")
                    .default(true)
                    .interact()
                {
                    Ok(result) => result,
                    Err(_) => {
                        // ESC pressed - return to main menu
                        return;
                    }
                };

                if confirm {
                    vec![0]  // Select the only item
                } else {
                    vec![]   // User declined
                }
            } else {
                // Multiple items, show interactive menu
                show_interactive_menu(&items)
            };

            if !selections.is_empty() {
                // Ask for dry-run
                let dry_run = match Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Dry-run mode (preview only)?")
                    .default(false)
                    .interact()
                {
                    Ok(result) => result,
                    Err(_) => {
                        // ESC pressed - return to main menu
                        return;
                    }
                };

                // Confirm deletion if not in dry-run mode
                if !dry_run {
                    let total_size: u64 = selections
                        .iter()
                        .filter_map(|&i| items.get(i))
                        .map(|item| item.size)
                        .sum();

                    if !confirm_deletion(selections.len(), total_size) {
                        println!("{}", "Cancelled.".yellow());
                        return;
                    }
                }

                let items_deleted = delete_items(&items, &selections, dry_run, false);

                if items_deleted {
                    clear_cache();
                    println!("\n{}", "Cache cleared.".dimmed());
                }
            }
        }
    }

    // Wait before returning to main menu
    println!("\n{}", "[Press Enter or ESC to return to main menu]".dimmed());
    let _ = Input::<String>::new().allow_empty(true).interact();
}

/// Interactive cache menu
fn menu_cache() {
    ui::clear_screen();
    ui::show_breadcrumb(&["Main Menu", "Cache Management"]);
    println!("\n{}", "Cache Management".bright_cyan().bold());

    let options = vec![
        "📋 Show Cache Info",
        "🗑️  Clear Cache",
        "↩️  Back to Main Menu",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an option")
        .items(&options)
        .default(0)
        .interact()
        .unwrap_or(2);

    match selection {
        0 => {
            println!();
            show_cache_info();
            println!("\n{}", "[Press Enter or ESC to return]".dimmed());
            let _ = Input::<String>::new().allow_empty(true).interact();
        }
        1 => {
            let confirm = match Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Are you sure you want to clear the cache?")
                .default(false)
                .interact()
            {
                Ok(result) => result,
                Err(_) => {
                    // ESC pressed - return to cache menu
                    return;
                }
            };

            if confirm {
                clear_cache();
                println!("\n{}", "Cache cleared successfully.".green());
            } else {
                println!("\n{}", "Cancelled.".yellow());
            }
            println!("\n{}", "[Press Enter or ESC to return]".dimmed());
            let _ = Input::<String>::new().allow_empty(true).interact();
        }
        _ => {}
    }
}

/// Interactive help menu
fn menu_help() {
    ui::clear_screen();
    ui::show_breadcrumb(&["Main Menu", "Help & Documentation"]);
    println!("\n{}", "Help & Documentation".bright_cyan().bold());

    ui::show_keyboard_shortcuts();

    println!("\n{}", "COMMAND-LINE USAGE:".bright_cyan().bold());
    println!("{}", "──────────────────────────────────────────────────────".bright_black());
    println!("  {:<30} {}", "gigabroom".bright_green(), "Launch interactive menu");
    println!("  {:<30} {}", "gigabroom scan [PATH]".bright_green(), "Scan for build artifacts");
    println!("  {:<30} {}", "gigabroom clean [PATH]".bright_green(), "Clean with interactive selection");
    println!("  {:<30} {}", "gigabroom --help".bright_green(), "Show detailed help");

    println!("\n{}", "TIPS & TRICKS:".bright_cyan().bold());
    println!("{}", "──────────────────────────────────────────────────────".bright_black());
    println!("  {} Use presets for one-click cleaning", "•".bright_yellow());
    println!("  {} Spotlight search (macOS) is faster than recursive", "•".bright_yellow());
    println!("  {} Check disk space before/after with --verbose", "•".bright_yellow());
    println!("  {} Use --dry-run to preview without deleting", "•".bright_yellow());
    println!("  {} Filter by size: --min-size 100MB", "•".bright_yellow());

    println!("\n{}", "PRESETS:".bright_cyan().bold());
    println!("{}", "──────────────────────────────────────────────────────".bright_black());
    println!("  {} {} Safe, common build artifacts", "🚀 Quick Clean:".bright_green().bold(), "-");
    println!("  {} {} All build artifacts + caches", "🧹 Deep Clean:".bright_yellow().bold(), "-");
    println!("  {} {} Includes global package caches", "⚠️  Nuclear Clean:".bright_red().bold(), "-");

    println!("\n{}", "[Press Enter or ESC to return]".dimmed());
    let _ = Input::<String>::new().allow_empty(true).interact();
}
