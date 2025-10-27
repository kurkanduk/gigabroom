/// UI Components and Utilities for Rust Cleaner
use colored::*;

/// Clear the terminal screen
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

/// Box drawing characters
pub mod boxes {
    // Single line
    pub const TOP_LEFT: &str = "┌";
    pub const TOP_RIGHT: &str = "┐";
    pub const BOTTOM_LEFT: &str = "└";
    pub const BOTTOM_RIGHT: &str = "┘";
    pub const HORIZONTAL: &str = "─";
    pub const VERTICAL: &str = "│";

    #[allow(dead_code)]
    pub const T_DOWN: &str = "┬";
    #[allow(dead_code)]
    pub const T_UP: &str = "┴";
    #[allow(dead_code)]
    pub const T_RIGHT: &str = "├";
    #[allow(dead_code)]
    pub const T_LEFT: &str = "┤";
    #[allow(dead_code)]
    pub const CROSS: &str = "┼";

    // Double line (for important boxes)
    pub const DOUBLE_TOP_LEFT: &str = "╔";
    pub const DOUBLE_TOP_RIGHT: &str = "╗";
    pub const DOUBLE_BOTTOM_LEFT: &str = "╚";
    pub const DOUBLE_BOTTOM_RIGHT: &str = "╝";
    pub const DOUBLE_HORIZONTAL: &str = "═";
    pub const DOUBLE_VERTICAL: &str = "║";
}

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\x1B' {
            // Skip until we find a letter (end of escape sequence)
            for next_c in chars.by_ref() {
                if next_c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Calculate visual length of a string (excluding ANSI escape codes)
fn visual_len(s: &str) -> usize {
    let stripped = strip_ansi_codes(s);

    // Count visual width: most emoji and wide chars take 2 columns
    let mut width = 0;
    for c in stripped.chars() {
        // Skip zero-width characters
        if c >= '\u{200B}' && c <= '\u{200D}' {
            // Zero-width space, ZWSP, ZWJ
            continue;
        }
        if c == '\u{FE0F}' || c == '\u{FE0E}' {
            // Variation selectors (emoji vs text style) - don't add width
            continue;
        }

        if c.is_ascii() {
            width += 1;
        } else {
            // Unicode characters, including emoji, typically take 2 columns
            width += 2;
        }
    }
    width
}

/// Draw a box with title
pub fn draw_box(title: &str, content: &[String], width: usize, double_line: bool) {
    let (tl, tr, bl, br, h, v) = if double_line {
        (
            boxes::DOUBLE_TOP_LEFT,
            boxes::DOUBLE_TOP_RIGHT,
            boxes::DOUBLE_BOTTOM_LEFT,
            boxes::DOUBLE_BOTTOM_RIGHT,
            boxes::DOUBLE_HORIZONTAL,
            boxes::DOUBLE_VERTICAL,
        )
    } else {
        (
            boxes::TOP_LEFT,
            boxes::TOP_RIGHT,
            boxes::BOTTOM_LEFT,
            boxes::BOTTOM_RIGHT,
            boxes::HORIZONTAL,
            boxes::VERTICAL,
        )
    };

    // Top border with title
    if !title.is_empty() {
        let title_visual_len = visual_len(title);
        let left_pad = (width - title_visual_len - 2) / 2;
        let right_pad = width - title_visual_len - 2 - left_pad;
        println!(
            "{}{}{}{}{}",
            tl,
            h.repeat(left_pad),
            format!(" {} ", title).bright_cyan().bold(),
            h.repeat(right_pad),
            tr
        );
    } else {
        println!("{}{}{}", tl, h.repeat(width), tr);
    }

    // Content
    for line in content {
        let line_visual_len = visual_len(line);
        // Calculate padding with a safety limit
        let padding = if line_visual_len >= width {
            0
        } else {
            (width - line_visual_len).min(width) // Safety clamp
        };
        println!("{} {}{} {}", v, line, " ".repeat(padding), v);
    }

    // Bottom border
    println!("{}{}{}", bl, h.repeat(width), br);
}

/// Draw a simple divider
#[allow(dead_code)]
pub fn draw_divider(width: usize, style: DividerStyle) {
    let line = match style {
        DividerStyle::Light => boxes::HORIZONTAL,
        DividerStyle::Heavy => boxes::DOUBLE_HORIZONTAL,
        DividerStyle::Dotted => "·",
    };
    println!("{}", line.repeat(width).dimmed());
}

#[allow(dead_code)]
pub enum DividerStyle {
    Light,
    Heavy,
    Dotted,
}

/// Create a progress bar string
pub fn progress_bar(current: u64, total: u64, width: usize) -> String {
    if total == 0 {
        return format!("[{}]", " ".repeat(width));
    }

    let percentage = (current as f64 / total as f64) * 100.0;
    let filled = ((current as f64 / total as f64) * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    let bar = format!(
        "[{}{}] {:.1}%",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage
    );

    // Color based on percentage
    if percentage >= 90.0 {
        bar.bright_red().to_string()
    } else if percentage >= 70.0 {
        bar.bright_yellow().to_string()
    } else {
        bar.bright_green().to_string()
    }
}

/// Format a summary section
#[allow(dead_code)]
pub fn format_summary_line(label: &str, value: &str) -> String {
    format!("  {:<20} {}", label.bright_white().bold(), value)
}

/// Keyboard shortcuts help (available shortcuts)
pub fn show_keyboard_shortcuts() {
    println!("\n{}", "┌──────────────────────────────────────────────────────┐".bright_cyan());
    println!("{}", "│              KEYBOARD SHORTCUTS                      │".bright_cyan().bold());
    println!("{}", "└──────────────────────────────────────────────────────┘".bright_cyan());
    println!("  {}         Navigate items", "↑ ↓".bright_green().bold());
    println!("  {}       Select/Deselect current item", "Space".bright_green().bold());
    println!("  {}       Confirm selection", "Enter".bright_green().bold());
    println!("  {}         Cancel/Go back", "Esc".bright_green().bold());
    println!("  {}  {}   Quick navigation", "PgUp/PgDn".bright_green().bold(), " ".dimmed());
    println!("{}", "──────────────────────────────────────────────────────".bright_black());
}

/// Show compact inline hint
pub fn show_inline_hint() {
    println!("{}", "  💡 Hint: ↑↓ navigate | Space select | Enter confirm | Esc cancel".dimmed());
}

/// Display a formatted error with context and solutions
pub fn show_error(title: &str, details: &str, solutions: &[&str]) {
    println!("\n{}", "┌─────────────────────────────────────────────────────────────────────┐".bright_red());
    println!("{}", format!("│ ❌ ERROR: {}                              ", title).bright_red().bold());
    println!("{}", "└─────────────────────────────────────────────────────────────────────┘".bright_red());

    println!("\n{} {}", "Problem:".bright_red().bold(), details);

    if !solutions.is_empty() {
        println!("\n{}", "Possible solutions:".bright_yellow().bold());
        for (idx, solution) in solutions.iter().enumerate() {
            println!("  {}. {}", (idx + 1).to_string().bright_cyan(), solution);
        }
    }

    println!("{}", "\n─────────────────────────────────────────────────────────────────────".bright_black());
}

/// Display a formatted warning
#[allow(dead_code)]
pub fn show_warning(title: &str, message: &str) {
    println!("\n{}", "┌─────────────────────────────────────────────────────────────────────┐".bright_yellow());
    println!("{}", format!("│ ⚠️  WARNING: {}                           ", title).bright_yellow().bold());
    println!("{}", "└─────────────────────────────────────────────────────────────────────┘".bright_yellow());
    println!("\n{}", message.yellow());
    println!("{}", "─────────────────────────────────────────────────────────────────────".bright_black());
}

/// Display a formatted success message
#[allow(dead_code)]
pub fn show_success(message: &str) {
    println!("\n{}", "┌─────────────────────────────────────────────────────────────────────┐".bright_green());
    println!("{}", format!("│ ✓ {}                                       ", message).bright_green().bold());
    println!("{}", "└─────────────────────────────────────────────────────────────────────┘".bright_green());
}

/// Display breadcrumb navigation
pub fn show_breadcrumb(steps: &[&str]) {
    if steps.is_empty() {
        return;
    }

    let breadcrumb = steps
        .iter()
        .map(|s| s.bright_cyan().to_string())
        .collect::<Vec<_>>()
        .join(&" → ".dimmed().to_string());

    println!("\n{} {}", "📍".dimmed(), breadcrumb);
    println!("{}", "─".repeat(80).bright_black());
}

/// Preset profile definitions
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CleanPreset {
    pub name: &'static str,
    pub emoji: &'static str,
    pub description: &'static str,
    pub categories: Vec<crate::types::Category>,
    pub safety: SafetyLevel,
    pub estimated_gb: &'static str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyLevel {
    Safe,
    Moderate,
    Dangerous,
}

#[allow(dead_code)]
impl CleanPreset {
    pub fn quick_clean() -> Self {
        Self {
            name: "Quick Clean",
            emoji: "🚀",
            description: "Safe, common build artifacts",
            categories: vec![
                crate::types::Category::RustTarget,
                crate::types::Category::NodeModules,
                crate::types::Category::PythonCache,
                crate::types::Category::IDECache,
                crate::types::Category::OSJunk,
            ],
            safety: SafetyLevel::Safe,
            estimated_gb: "2-5 GB",
        }
    }

    pub fn deep_clean() -> Self {
        Self {
            name: "Deep Clean",
            emoji: "🧹",
            description: "All build artifacts + caches + temp",
            categories: vec![
                crate::types::Category::RustTarget,
                crate::types::Category::NodeModules,
                crate::types::Category::PythonCache,
                crate::types::Category::PHPVendor,
                crate::types::Category::RubyGems,
                crate::types::Category::MavenTarget,
                crate::types::Category::GradleBuild,
                crate::types::Category::GoVendor,
                crate::types::Category::CCache,
                crate::types::Category::DotNetBuild,
                crate::types::Category::SwiftBuild,
                crate::types::Category::IDECache,
                crate::types::Category::OSJunk,
                crate::types::Category::TempFiles,
                crate::types::Category::BuildCache,
            ],
            safety: SafetyLevel::Moderate,
            estimated_gb: "5-15 GB",
        }
    }

    pub fn nuclear_clean() -> Self {
        Self {
            name: "Nuclear Clean",
            emoji: "⚠️",
            description: "EVERYTHING including global package caches!",
            categories: vec![
                crate::types::Category::RustTarget,
                crate::types::Category::NodeModules,
                crate::types::Category::PythonCache,
                crate::types::Category::PHPVendor,
                crate::types::Category::RubyGems,
                crate::types::Category::MavenTarget,
                crate::types::Category::GradleBuild,
                crate::types::Category::GoVendor,
                crate::types::Category::CCache,
                crate::types::Category::DotNetBuild,
                crate::types::Category::SwiftBuild,
                crate::types::Category::IDECache,
                crate::types::Category::OSJunk,
                crate::types::Category::TempFiles,
                crate::types::Category::PackageCache,
                crate::types::Category::BuildCache,
            ],
            safety: SafetyLevel::Dangerous,
            estimated_gb: "10-30 GB",
        }
    }

    pub fn all_presets() -> Vec<Self> {
        vec![Self::quick_clean(), Self::deep_clean(), Self::nuclear_clean()]
    }

    pub fn display_name(&self) -> String {
        let safety_indicator = match self.safety {
            SafetyLevel::Safe => "✓".green(),
            SafetyLevel::Moderate => "⚠".yellow(),
            SafetyLevel::Dangerous => "⚠⚠".red(),
        };

        format!(
            "{} {} {} - {} ({})",
            self.emoji,
            self.name,
            safety_indicator,
            self.description,
            self.estimated_gb
        )
    }
}
