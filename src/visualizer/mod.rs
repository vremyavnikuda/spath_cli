//! PATH visualization.
use crate::models::{PathEntry, PathLocation};
use colored::*;

pub fn visualize_simple(paths: &[String], use_color: bool) {
    println!("\n{}", format_header("PATH Entries", use_color));
    println!();
    let entries = build_entries(paths);
    for entry in &entries {
        print_simple_entry(entry, use_color);
    }
    print_summary(&entries, use_color);
}

pub fn visualize_tree(paths: &[String], use_color: bool) {
    println!(
        "\n{}",
        format_header("PATH Structure (Tree View)", use_color)
    );
    println!();
    let entries = build_entries(paths);
    for (i, entry) in entries.iter().enumerate() {
        print_tree_entry(entry, i == entries.len() - 1, use_color);
    }
    print_summary(&entries, use_color);
}

fn build_entries(paths: &[String]) -> Vec<PathEntry> {
    paths
        .iter()
        .enumerate()
        .map(|(i, p)| PathEntry::new(p.clone(), i, PathLocation::User, paths))
        .collect()
}

fn format_header(text: &str, use_color: bool) -> ColoredString {
    if use_color {
        text.bold().bright_blue()
    } else {
        text.normal()
    }
}

fn print_simple_entry(entry: &PathEntry, use_color: bool) {
    let index_str = format!("[{}]", entry.index);
    let status = if entry.exists { "✓" } else { "✗" };
    let line = format_entry_line(&index_str, status, &entry.path, entry, use_color, "");
    println!("{}", line);
    print_warnings(entry, use_color, "    ");
}

fn print_tree_entry(entry: &PathEntry, is_last: bool, use_color: bool) {
    let branch = if is_last { "└─" } else { "├─" };
    let index_str = format!("[{}]", entry.index);
    let status = if entry.exists { "✓" } else { "✗" };
    let line = format_entry_line(&index_str, status, &entry.path, entry, use_color, branch);
    println!("{}", line);
    let continuation = if is_last { "   " } else { "│  " };
    print_tree_warnings(entry, use_color, continuation);
    print_user_specific_info(entry, is_last, use_color);
}

fn format_entry_line(
    index_str: &str,
    status: &str,
    path: &str,
    entry: &PathEntry,
    use_color: bool,
    prefix: &str,
) -> String {
    if use_color {
        let colored_index = index_str.bright_black();
        let colored_status = if entry.exists {
            status.green()
        } else {
            status.red()
        };
        let colored_path = if entry.has_issues() {
            path.yellow()
        } else if entry.is_user_specific() {
            path.cyan()
        } else {
            path.normal()
        };
        if prefix.is_empty() {
            format!("{} {} {}", colored_index, colored_status, colored_path)
        } else {
            format!(
                "{} {} {} {}",
                prefix.bright_black(),
                colored_index,
                colored_status,
                colored_path
            )
        }
    } else if prefix.is_empty() {
        format!("{} {} {}", index_str, status, path)
    } else {
        format!("{} {} {} {}", prefix, index_str, status, path)
    }
}

fn print_warnings(entry: &PathEntry, use_color: bool, indent: &str) {
    for warning in entry.get_warnings() {
        let line = if use_color {
            format!("{}⚠ {}", indent, warning).yellow()
        } else {
            format!("{}! {}", indent, warning).normal()
        };
        println!("{}", line);
    }
}

fn print_tree_warnings(entry: &PathEntry, use_color: bool, continuation: &str) {
    let warnings = entry.get_warnings();
    for (j, warning) in warnings.iter().enumerate() {
        let sub_branch = if j == warnings.len() - 1 {
            "└─"
        } else {
            "├─"
        };
        let line = if use_color {
            format!("{}  {} ⚠ {}", continuation, sub_branch, warning).yellow()
        } else {
            format!("{}  {} ! {}", continuation, sub_branch, warning).normal()
        };
        println!("{}", line);
    }
}

fn print_user_specific_info(entry: &PathEntry, is_last: bool, use_color: bool) {
    if entry.is_user_specific() && entry.get_warnings().is_empty() {
        let continuation = if is_last { "   " } else { "│  " };
        let line = if use_color {
            format!("{}  └─ ℹ User-specific path", continuation).cyan()
        } else {
            format!("{}  └─ i User-specific path", continuation).normal()
        };
        println!("{}", line);
    }
}

fn print_summary(entries: &[PathEntry], use_color: bool) {
    println!();
    println!("{}", format_header("Summary", use_color));
    let total = entries.len();
    let existing = entries.iter().filter(|e: &&PathEntry| e.exists).count();
    let with_issues = entries
        .iter()
        .filter(|e: &&PathEntry| e.has_issues())
        .count();
    let user_specific = entries
        .iter()
        .filter(|e: &&PathEntry| e.is_user_specific())
        .count();
    let duplicates = entries
        .iter()
        .filter(|e: &&PathEntry| e.is_duplicate)
        .count();
    println!("  Total paths: {}", total);
    println!("  Existing: {} / {}", existing, total);
    println!("  With issues: {}", with_issues);
    println!("  User-specific: {}", user_specific);
    println!("  Duplicates: {}", duplicates);
    if use_color {
        println!();
        println!("Legend:");
        println!("  {} Exists", "✓".green());
        println!("  {} Does not exist", "✗".red());
        println!("  {} Has issues", "path".yellow());
        println!("  {} User-specific", "path".cyan());
    }
}
