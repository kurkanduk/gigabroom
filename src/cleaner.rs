use crate::types::{Category, DeletableItem};
use crate::utils::format_size;
use crate::{print_error, println_unless_quiet};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};
use std::collections::{HashMap, HashSet};

#[cfg(target_os = "macos")]
use std::process::Command;

/// Select categories interactively
pub fn select_categories() -> Vec<Category> {
    crate::ui::show_breadcrumb(&["Scan & Clean", "Select Categories"]);

    let categories = vec![
        ("🦀 Rust - target directories", Category::RustTarget),
        ("📦 JavaScript/Node - node_modules", Category::NodeModules),
        ("🐍 Python - cache & venv", Category::PythonCache),
        ("🐘 PHP - vendor (Composer)", Category::PHPVendor),
        ("💎 Ruby - vendor/bundle (Bundler)", Category::RubyGems),
        ("☕ Java - Maven target", Category::MavenTarget),
        ("☕ Java - Gradle build", Category::GradleBuild),
        ("🐹 Go - vendor directories", Category::GoVendor),
        ("⚙️  C/C++ - build artifacts", Category::CCache),
        ("🔷 .NET - bin/obj/packages", Category::DotNetBuild),
        ("🦢 Swift - .build/DerivedData", Category::SwiftBuild),
        ("💡 IDE - .idea/.vscode/.vs", Category::IDECache),
        ("🗑️  OS Junk - .DS_Store/Thumbs.db", Category::OSJunk),
        ("📝 Temp/Logs - *.log/*.tmp", Category::TempFiles),
        ("⚠️  Package Caches (DANGEROUS: global caches!)", Category::PackageCache),
        ("📁 General - build/dist/out", Category::BuildCache),
    ];

    let category_names: Vec<String> = categories.iter().map(|(name, _)| (*name).to_string()).collect();

    println!("\n{}", "Select categories to clean:".bright_cyan().bold());
    crate::ui::show_inline_hint();
    println!("{}", "⚠️  WARNING: Items marked DANGEROUS will delete important data!".bright_red().bold());

    let selections = match MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&category_names)
        .interact()
    {
        Ok(sel) => sel,
        Err(_) => {
            println!("\n{}", "Cancelled".yellow());
            return Vec::new();
        }
    };

    if selections.is_empty() {
        println!("\n{}", "No categories selected (you need to press Space to select items)".yellow());
        return Vec::new();
    }

    let selected_categories: Vec<Category> = selections
        .into_iter()
        .filter_map(|idx| categories.get(idx).map(|(_, cat)| *cat))
        .collect();

    // Check if dangerous categories are selected
    let dangerous_selected: Vec<Category> = selected_categories
        .iter()
        .filter(|cat| cat.is_dangerous())
        .copied()
        .collect();

    if !dangerous_selected.is_empty() {
        println!("\n{}", "═".repeat(80).bright_red());
        println!("{}", "⚠️  DANGER WARNING ⚠️".bright_red().bold());
        println!("{}", "═".repeat(80).bright_red());
        println!("\n{}", "You selected these DANGEROUS categories:".bright_red().bold());
        for cat in &dangerous_selected {
            match cat {
                Category::PackageCache => {
                    println!("  {} Deleting package caches will affect ALL projects!", "•".bright_red());
                    println!("    Includes: npm, pip, yarn, Maven global caches");
                    println!("    All projects will need to re-download dependencies!");
                }
                _ => {}
            }
        }
        println!("\n{}", "═".repeat(80).bright_red());

        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you ABSOLUTELY SURE you want to proceed?")
            .default(false)
            .interact()
            .unwrap_or(false);

        if !confirm {
            println!("{}", "Cancelled for safety.".yellow());
            return Vec::new();
        }
    }

    selected_categories
}

/// Show interactive menu for selecting items to delete
pub fn show_interactive_menu(items: &[DeletableItem]) -> Vec<usize> {
    if items.is_empty() {
        println!("\n{}", "No deletable items found!".green().bold());
        return Vec::new();
    }

    let total_size: u64 = items.iter().map(|i| i.size).sum();

    println!("\n{}", "Found deletable items:".bright_yellow().bold());
    println!("{}", "=".repeat(80).bright_black());
    println!(
        "{} {} in {} items",
        "Total reclaimable space:".bright_white().bold(),
        format_size(total_size).bright_green().bold(),
        items.len()
    );
    println!("{}", "=".repeat(80).bright_black());

    // Offer quick selection menu for many items
    if items.len() > 10 {
        println!("\n{}", "Choose selection method:".bright_cyan().bold());
        crate::ui::show_inline_hint();

        let options = vec![
            format!("✓ Select All ({} items)", items.len()),
            "🗂️  Select by Category".to_string(),
            "📝 Select Individually".to_string(),
            "✗ Cancel".to_string(),
        ];

        let choice = match dialoguer::Select::with_theme(&ColorfulTheme::default())
            .items(&options)
            .default(0)
            .interact()
        {
            Ok(c) => c,
            Err(_) => {
                println!("\n{}", "Cancelled".yellow());
                return Vec::new();
            }
        };

        match choice {
            0 => {
                // Select All
                println!("\n{}", "✓ All items selected".green());
                return (0..items.len()).collect();
            }
            1 => {
                // Select by Category
                return select_items_by_category(items);
            }
            2 => {
                // Select Individually (continue below)
            }
            _ => {
                println!("\n{}", "Cancelled".yellow());
                return Vec::new();
            }
        }
    }

    // Individual selection
    println!("\n{}", "Select items to delete (sorted by size):".bright_cyan());
    println!("{}", "Use ↑↓ to navigate, Space to select, Enter to confirm".dimmed());

    // Create a sorted index map (largest first)
    let mut sorted_indices: Vec<usize> = (0..items.len()).collect();
    sorted_indices.sort_by(|&a, &b| items[b].size.cmp(&items[a].size));

    let menu_items: Vec<String> = sorted_indices
        .iter()
        .map(|&idx| {
            let item = &items[idx];
            format!(
                "{} - {} ({})",
                item.project_name,
                item.category.name(),
                format_size(item.size)
            )
        })
        .collect();

    let selected_sorted_indices = match MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&menu_items)
        .interact()
    {
        Ok(sel) => sel,
        Err(_) => {
            println!("\n{}", "Cancelled".yellow());
            return Vec::new();
        }
    };

    if selected_sorted_indices.is_empty() {
        println!("\n{}", "No items selected (use Space bar to select)".yellow());
        return Vec::new();
    }

    // Map back to original indices
    selected_sorted_indices
        .into_iter()
        .map(|sorted_idx| sorted_indices[sorted_idx])
        .collect()
}

/// Helper function to get emoji for category
fn get_category_emoji(cat: &Category) -> &'static str {
    match cat {
        Category::RustTarget => "🦀",
        Category::NodeModules => "📦",
        Category::PythonCache => "🐍",
        Category::PHPVendor => "🐘",
        Category::RubyGems => "💎",
        Category::MavenTarget | Category::GradleBuild => "☕",
        Category::GoVendor => "🐹",
        Category::CCache => "⚙️",
        Category::DotNetBuild => "🔷",
        Category::SwiftBuild => "🦢",
        Category::IDECache => "💡",
        Category::OSJunk => "🗑️",
        Category::TempFiles => "📝",
        Category::PackageCache => "⚠️",
        Category::BuildCache => "📁",
    }
}

/// Select items by category with hierarchical drill-down navigation
fn select_items_by_category(items: &[DeletableItem]) -> Vec<usize> {
    // Group items by category
    let mut category_map: HashMap<Category, Vec<usize>> = HashMap::new();
    for (idx, item) in items.iter().enumerate() {
        category_map.entry(item.category).or_insert_with(Vec::new).push(idx);
    }

    // Calculate size per category
    let mut category_sizes: HashMap<Category, u64> = HashMap::new();
    for item in items.iter() {
        *category_sizes.entry(item.category).or_insert(0) += item.size;
    }

    // Sort categories by size (largest first)
    let mut sorted_categories: Vec<Category> = category_map.keys().copied().collect();
    sorted_categories.sort_by(|a, b| category_sizes[b].cmp(&category_sizes[a]));

    // Track all selected items across categories
    let mut all_selections: HashSet<usize> = HashSet::new();

    // Main category navigation loop
    loop {
        crate::ui::clear_screen();
        crate::ui::show_breadcrumb(&["Scan & Clean", "Select by Category"]);

        // Calculate total selected
        let total_selected_count = all_selections.len();
        let total_selected_size: u64 = all_selections
            .iter()
            .filter_map(|&idx| items.get(idx))
            .map(|item| item.size)
            .sum();

        // Build menu with current selection status
        let mut menu_options = Vec::new();

        if total_selected_count > 0 {
            menu_options.push(format!(
                "✅ Review & Delete Selected ({} items, {})",
                total_selected_count,
                format_size(total_selected_size)
            ));
        } else {
            menu_options.push("❌ No items selected yet".to_string());
        }

        menu_options.push("──────────────────────────────".to_string());

        for cat in &sorted_categories {
            let count = category_map[cat].len();
            let size = category_sizes[cat];
            let emoji = get_category_emoji(cat);

            // Check how many items from this category are selected
            let selected_in_cat = all_selections
                .iter()
                .filter(|&&idx| items.get(idx).map(|i| i.category == *cat).unwrap_or(false))
                .count();

            let selection_indicator = if selected_in_cat > 0 {
                format!(" [✓ {} selected]", selected_in_cat)
            } else {
                String::new()
            };

            menu_options.push(format!(
                "{} {} - {} ({} items){}",
                emoji,
                cat.name(),
                format_size(size),
                count,
                selection_indicator.bright_green()
            ));
        }

        println!("\n{}", "Browse categories:".bright_cyan().bold());
        crate::ui::show_inline_hint();
        println!("{}", "💡 Tip: Press Enter to drill into a category, ESC to finish".dimmed());

        let selection = match Select::with_theme(&ColorfulTheme::default())
            .items(&menu_options)
            .default(0)
            .interact()
        {
            Ok(idx) => idx,
            Err(_) => {
                // ESC pressed - return current selections
                return all_selections.into_iter().collect();
            }
        };

        if selection == 0 {
            // Review & Delete or No items selected
            if total_selected_count > 0 {
                return all_selections.into_iter().collect();
            } else {
                continue; // No items selected, stay in menu
            }
        } else if selection == 1 {
            // Separator, ignore
            continue;
        } else {
            // Category selected, drill down
            let cat_idx = selection - 2; // Account for "Review" option and separator
            if let Some(cat) = sorted_categories.get(cat_idx) {
                if let Some(indices) = category_map.get(cat) {
                    // Show items in this category
                    let category_selections = select_items_in_category(items, indices, cat, &all_selections);

                    // Update selections
                    // Remove old selections from this category
                    all_selections.retain(|idx| {
                        items.get(*idx).map(|i| i.category != *cat).unwrap_or(false)
                    });
                    // Add new selections
                    all_selections.extend(category_selections);
                }
            }
        }
    }
}

/// Select specific items within a category
fn select_items_in_category(
    items: &[DeletableItem],
    category_indices: &[usize],
    category: &Category,
    current_selections: &HashSet<usize>,
) -> Vec<usize> {
    crate::ui::clear_screen();
    crate::ui::show_breadcrumb(&["Scan & Clean", "Select by Category", category.name()]);

    // Sort by size (largest first)
    let mut sorted_indices = category_indices.to_vec();
    sorted_indices.sort_by(|&a, &b| {
        items.get(b).map(|i| i.size).unwrap_or(0)
            .cmp(&items.get(a).map(|i| i.size).unwrap_or(0))
    });

    let menu_items: Vec<String> = sorted_indices
        .iter()
        .map(|&idx| {
            let item = &items[idx];
            format!(
                "{} ({})",
                item.project_name,
                format_size(item.size)
            )
        })
        .collect();

    // Pre-select items that are already in current_selections
    let defaults: Vec<bool> = sorted_indices
        .iter()
        .map(|idx| current_selections.contains(idx))
        .collect();

    println!("\n{}", format!("Select items from {}:", category.name()).bright_cyan().bold());
    crate::ui::show_inline_hint();
    println!("{}", "💡 Tip: Press ESC to go back to category list".dimmed());

    let selected = match MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&menu_items)
        .defaults(&defaults)
        .interact()
    {
        Ok(sel) => sel,
        Err(_) => {
            // ESC pressed - return current selections for this category
            return sorted_indices
                .iter()
                .filter(|idx| current_selections.contains(idx))
                .copied()
                .collect();
        }
    };

    // Map back to original indices
    selected
        .into_iter()
        .filter_map(|sorted_idx| sorted_indices.get(sorted_idx).copied())
        .collect()
}

/// Delete items with optional dry-run mode
pub fn delete_items(
    items: &[DeletableItem],
    indices: &[usize],
    dry_run: bool,
    quiet: bool,
) -> bool {
    if indices.is_empty() {
        println_unless_quiet!(quiet, "\n{}", "No items selected for deletion.".yellow());
        return false;
    }

    if dry_run {
        println_unless_quiet!(
            quiet,
            "\n{}",
            "DRY RUN - No files will be deleted".bright_yellow().bold()
        );
        println_unless_quiet!(quiet, "{}", "=".repeat(80).bright_black());

        let mut total_size = 0u64;
        for &idx in indices {
            if let Some(item) = items.get(idx) {
                println_unless_quiet!(quiet, "Would delete: {}", item.path.display());
                total_size += item.size;
            }
        }

        println_unless_quiet!(quiet, "\n{}", "=".repeat(80).bright_black());
        println_unless_quiet!(
            quiet,
            "{} {} items",
            "Would delete:".bright_yellow().bold(),
            indices.len().to_string().bright_yellow().bold()
        );
        println_unless_quiet!(
            quiet,
            "{} {}",
            "Would free:".bright_yellow().bold(),
            format_size(total_size).bright_yellow().bold()
        );

        return false;
    }

    println_unless_quiet!(
        quiet,
        "\n{}",
        "Deleting selected items...".bright_yellow().bold()
    );

    let mut deleted_count = 0;
    let mut failed_count = 0;
    let mut total_freed = 0u64;

    for &idx in indices {
        if let Some(item) = items.get(idx) {
            if !quiet {
                print!("Deleting {} ... ", item.path.display());
            }

            match std::fs::remove_dir_all(&item.path)
                .or_else(|_| std::fs::remove_file(&item.path))
            {
                Ok(_) => {
                    println_unless_quiet!(quiet, "{}", "✓".green().bold());
                    deleted_count += 1;
                    total_freed += item.size;
                }
                Err(e) => {
                    if quiet {
                        print_error!("Failed to delete {}: {}", item.path.display(), e);
                    } else {
                        println!("{} {}", "✗".red().bold(), e.to_string().red());
                    }
                    failed_count += 1;
                }
            }
        }
    }

    println_unless_quiet!(quiet, "\n{}", "=".repeat(80).bright_black());
    println_unless_quiet!(
        quiet,
        "{} {} items",
        "Successfully deleted:".bright_green().bold(),
        deleted_count.to_string().bright_green().bold()
    );

    if failed_count > 0 {
        println_unless_quiet!(
            quiet,
            "{} {} items",
            "Failed to delete:".bright_red().bold(),
            failed_count.to_string().bright_red().bold()
        );
    }

    println_unless_quiet!(
        quiet,
        "{} {}",
        "Space freed:".bright_green().bold(),
        format_size(total_freed).bright_green().bold()
    );

    deleted_count > 0
}

/// Confirm deletion with user - Enhanced visual summary
pub fn confirm_deletion(item_count: usize, total_size: u64) -> bool {
    show_deletion_summary(item_count, total_size, &HashMap::new());

    Confirm::new()
        .with_prompt("Proceed with deletion?")
        .default(false)
        .interact()
        .unwrap_or(false)
}

/// Show detailed deletion summary with visual box
pub fn show_deletion_summary(item_count: usize, total_size: u64, categories: &HashMap<Category, usize>) {
    use crate::ui;

    println!();

    // Build content lines
    let mut content = vec![];
    content.push(format!("{}  {} items", "Will Delete:".bright_white().bold(), item_count.to_string().bright_yellow().bold()));
    content.push(format!("{}  {}", "Will Free:".bright_white().bold(), format_size(total_size).bright_green().bold()));

    if !categories.is_empty() {
        content.push(String::new());
        content.push("Categories:".bright_white().bold().to_string());

        let mut sorted_cats: Vec<_> = categories.iter().collect();
        sorted_cats.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        for (cat, count) in sorted_cats {
            let emoji = match cat {
                Category::RustTarget => "🦀",
                Category::NodeModules => "📦",
                Category::PythonCache => "🐍",
                Category::PHPVendor => "🐘",
                Category::RubyGems => "💎",
                Category::MavenTarget | Category::GradleBuild => "☕",
                Category::GoVendor => "🐹",
                Category::CCache => "⚙️ ",
                Category::DotNetBuild => "🔷",
                Category::SwiftBuild => "🦢",
                Category::IDECache => "💡",
                Category::OSJunk => "🗑️ ",
                Category::TempFiles => "📝",
                Category::PackageCache => "⚠️ ",
                Category::BuildCache => "📁",
            };
            content.push(format!("  {} {:20} {} items", emoji, cat.name(), count));
        }
    }

    content.push(String::new());
    content.push("⚠ Warning: This action cannot be undone!".bright_red().to_string());

    content.push(String::new());
    content.push(format!("{} {} {} {}",
        "[d]".bright_cyan(),
        "Dry Run".dimmed(),
        "[Enter]".bright_green(),
        "Proceed".dimmed()
    ));

    ui::draw_box("DELETION SUMMARY", &content, 45, true);
}

/// Display statistics dashboard with breakdown by category
#[allow(dead_code)]
pub fn show_statistics(items: &[DeletableItem]) {
    if items.is_empty() {
        return;
    }

    // Calculate totals by category
    let mut category_stats: HashMap<Category, (u64, usize)> = HashMap::new();
    let mut total_size = 0u64;

    for item in items {
        let entry = category_stats.entry(item.category).or_insert((0, 0));
        entry.0 += item.size;
        entry.1 += 1;
        total_size += item.size;
    }

    // Sort categories by size (descending)
    let mut sorted_categories: Vec<_> = category_stats.iter().collect();
    sorted_categories.sort_by(|a, b| b.1.0.cmp(&a.1.0));

    // Display statistics
    println!("\n{}", "═".repeat(80).bright_cyan());
    println!("  {}", "📊 SCAN SUMMARY".bright_cyan().bold());
    println!("{}", "═".repeat(80).bright_cyan());

    println!(
        "\n{}  {}  {}",
        "Total:".bright_white().bold(),
        format_size(total_size).bright_green().bold(),
        format!("({} items)", items.len()).dimmed()
    );

    println!("\n{}", "By Category:".bright_white().bold());
    println!("{}", "-".repeat(80).bright_black());

    for (category, (size, count)) in &sorted_categories {
        let percentage = (*size as f64 / total_size as f64) * 100.0;
        let bar_length = (percentage / 2.0) as usize; // Scale to 50 chars max
        let bar = "█".repeat(bar_length);

        let category_emoji = match category {
            Category::RustTarget => "🦀",
            Category::NodeModules => "📦",
            Category::PythonCache => "🐍",
            Category::PHPVendor => "🐘",
            Category::RubyGems => "💎",
            Category::MavenTarget => "☕",
            Category::GradleBuild => "☕",
            Category::GoVendor => "🐹",
            Category::CCache => "⚙️ ",
            Category::DotNetBuild => "🔷",
            Category::SwiftBuild => "🦢",
            Category::IDECache => "💡",
            Category::OSJunk => "🗑️ ",
            Category::TempFiles => "📝",
            Category::PackageCache => "⚠️ ",
            Category::BuildCache => "📁",
        };

        println!(
            "{} {:20}  {:>10}  {:>6.1}%  {} {}",
            category_emoji,
            category.name(),
            format_size(*size).bright_green(),
            percentage,
            bar.bright_cyan(),
            format!("({} items)", count).dimmed()
        );
    }

    // Find largest projects
    let mut sorted_items = items.to_vec();
    sorted_items.sort_by(|a, b| b.size.cmp(&a.size));

    println!("\n{}", "Largest Items:".bright_white().bold());
    println!("{}", "-".repeat(80).bright_black());

    for (i, item) in sorted_items.iter().take(5).enumerate() {
        println!(
            "  {}. {:30}  {:>10}  {}",
            i + 1,
            item.project_name.chars().take(30).collect::<String>(),
            format_size(item.size).bright_yellow(),
            item.category.name().dimmed()
        );
    }

    println!("{}", "═".repeat(80).bright_cyan());
}

/// Display disk space context (macOS only for now)
#[allow(dead_code)]
#[cfg(target_os = "macos")]
pub fn show_disk_space(reclaimable_size: u64) {
    // Get disk space info using `df -h /`
    let output = Command::new("df")
        .args(&["-k", "/"])  // Use kilobytes for consistent parsing
        .output();

    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        // Parse df output: Filesystem 1024-blocks Used Available Capacity Mounted
        if let Some(line) = output_str.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let (Ok(total_kb), Ok(used_kb), Ok(avail_kb)) = (
                    parts[1].parse::<u64>(),
                    parts[2].parse::<u64>(),
                    parts[3].parse::<u64>(),
                ) {
                    let total = total_kb * 1024;
                    let used = used_kb * 1024;
                    let available = avail_kb * 1024;
                    let used_percent = (used as f64 / total as f64) * 100.0;
                    let after_cleanup = available + reclaimable_size;
                    let after_percent = ((total - used + reclaimable_size) as f64 / total as f64) * 100.0;

                    println!("\n{}", "═".repeat(80).bright_blue());
                    println!("  {}", "💾 DISK SPACE CONTEXT".bright_blue().bold());
                    println!("{}", "═".repeat(80).bright_blue());

                    println!("\n{}", "Current:".bright_white().bold());
                    println!("  Total:      {}", format_size(total).bright_white());
                    println!("  Used:       {} ({:.1}%)", format_size(used).bright_red(), used_percent);
                    println!("  Available:  {}", format_size(available).bright_green());

                    println!("\n{}", "After cleanup:".bright_white().bold());
                    println!("  Available:  {} ({:.1}% free)",
                        format_size(after_cleanup).bright_green().bold(),
                        after_percent
                    );
                    println!("  Gain:       {} ({:.1}%)",
                        format_size(reclaimable_size).bright_cyan().bold(),
                        (reclaimable_size as f64 / total as f64) * 100.0
                    );

                    println!("{}", "═".repeat(80).bright_blue());
                }
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn show_disk_space(_reclaimable_size: u64) {
    // Not implemented for non-macOS systems yet
}
