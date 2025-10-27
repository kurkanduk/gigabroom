/// Display and formatting functions
use crate::types::{Category, DeletableItem};
use crate::utils::format_size;
use crate::ui;
use colored::*;
use std::collections::HashMap;

/// Print ASCII art header with version
pub fn print_header(quiet: bool, json: bool) {
    if !quiet && !json {
        println!("\n{}", "   ____ _             _                            ".bright_cyan());
        println!("{}", "  / ___(_) __ _  __ _| |__  _ __ ___   ___  _ __ ___  ".bright_cyan());
        println!("{}", " | |  _| |/ _` |/ _` | '_ \\| '__/ _ \\ / _ \\| '_ ` _ \\ ".bright_cyan());
        println!("{}", " | |_| | | (_| | (_| | |_) | | | (_) | (_) | | | | | |".bright_cyan());
        println!("{}", "  \\____|_|\\__, |\\__,_|_.__/|_|  \\___/ \\___/|_| |_| |_|".bright_cyan());
        println!("{}", "          |___/                                         ".bright_cyan());
        println!("{}", format!("                                            v{}", env!("CARGO_PKG_VERSION")).dimmed());
        println!("{}", "  🧹 Sweep away gigabytes of build artifacts".dimmed());
    }
}

/// Display scan results grouped by category
pub fn display_scan_results(items: &[DeletableItem], verbose: bool, _quiet: bool, from_interactive_menu: bool) {
    // Clear screen in interactive mode to avoid clutter
    if from_interactive_menu {
        ui::clear_screen();
    }

    if items.is_empty() {
        println!("\n{}", "No deletable items found!".green().bold());
        return;
    }

    // Group items by category
    let mut category_groups: HashMap<Category, Vec<&DeletableItem>> = HashMap::new();
    for item in items {
        category_groups.entry(item.category).or_default().push(item);
    }

    // Calculate totals
    let total_size: u64 = items.iter().map(|i| i.size).sum();
    let total_count = items.len();

    println!("\n📋 {} items found ({})",
        total_count.to_string().bright_yellow().bold(),
        format_size(total_size).bright_green().bold()
    );

    // Sort category groups by total size (descending)
    let mut sorted_categories: Vec<_> = category_groups.iter().collect();
    sorted_categories.sort_by(|a, b| {
        let size_a: u64 = a.1.iter().map(|item| item.size).sum();
        let size_b: u64 = b.1.iter().map(|item| item.size).sum();
        size_b.cmp(&size_a)
    });

    for (category, category_items) in sorted_categories {
        let category_size: u64 = category_items.iter().map(|item| item.size).sum();
        let percentage = (category_size as f64 / total_size as f64) * 100.0;

        // Category header with emoji and stats
        let emoji = get_category_emoji(category);

        println!("\n{} {} • {} • {} items ({:.1}%)",
            emoji,
            category.name().bright_white().bold(),
            format_size(category_size).bright_green().bold(),
            category_items.len(),
            percentage
        );

        // Sort items within category by size (largest first)
        let mut sorted_items: Vec<_> = category_items.iter().copied().collect();
        sorted_items.sort_by(|a, b| b.size.cmp(&a.size));

        // Display items (show max 5)
        let display_count = if sorted_items.len() <= 5 { sorted_items.len() } else { 5 };

        for (idx, item) in sorted_items.iter().enumerate().take(display_count) {
            let bar_width = 20;
            let bar = ui::progress_bar(item.size, category_size, bar_width);

            println!(
                "  {:2}. {:30} {:>10}  {}",
                idx + 1,
                item.project_name.chars().take(30).collect::<String>().bright_cyan(),
                format_size(item.size),
                bar
            );

            if verbose {
                println!("      {}", item.path.display().to_string().dimmed());
            }
        }

        if sorted_items.len() > display_count {
            println!("      {} {} more items", "...and".dimmed(), (sorted_items.len() - display_count).to_string().bright_yellow());
        }
    }
}

/// Get emoji for a category
fn get_category_emoji(category: &Category) -> &str {
    match category {
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
