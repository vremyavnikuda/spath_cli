use crate::constants::USER_PATHS;
use colored::*;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PathEntry {
    pub index: usize,
    pub path: String,
    pub exists: bool,
    pub has_spaces: bool,
    pub is_quoted: bool,
    pub is_user_specific: bool,
    pub is_duplicate: bool,
}

impl PathEntry {
    pub fn new(index: usize, path: String, all_paths: &[String]) -> Self {
        let trimmed = path.trim_matches('"');
        let exists = Path::new(trimmed).exists();
        let has_spaces = trimmed.contains(' ');
        let is_quoted = path.starts_with('"') && path.ends_with('"');
        let is_user_specific = Self::check_user_specific(trimmed);
        let normalized = trimmed.to_lowercase();
        let is_duplicate = all_paths
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .any(|(_, p)| p.trim_matches('"').to_lowercase() == normalized);

        Self {
            index,
            path,
            exists,
            has_spaces,
            is_quoted,
            is_user_specific,
            is_duplicate,
        }
    }

    fn check_user_specific(path: &str) -> bool {
        let lower = path.to_lowercase();
        lower.contains("\\users\\") || USER_PATHS.iter().any(|p| lower.contains(p))
    }

    pub fn has_issues(&self) -> bool {
        !self.exists || (self.has_spaces && !self.is_quoted) || self.is_duplicate
    }

    pub fn get_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if !self.exists {
            warnings.push("Path does not exist".to_string());
        }
        if self.has_spaces && !self.is_quoted {
            warnings.push("Contains spaces but not quoted".to_string());
        }
        if self.is_duplicate {
            warnings.push("Duplicate path".to_string());
        }
        if self.path.len() > 260 {
            warnings.push("Path exceeds 260 characters".to_string());
        }
        warnings
    }
}

pub fn visualize_simple(paths: &[String], use_color: bool) {
    println!("\n{}", format_header("PATH Entries", use_color));
    println!();
    for (i, path) in paths.iter().enumerate() {
        let entry = PathEntry::new(i, path.clone(), paths);
        print_simple_entry(&entry, use_color);
    }
    print_summary(paths, use_color);
}

pub fn visualize_tree(paths: &[String], use_color: bool) {
    println!(
        "\n{}",
        format_header("PATH Structure (Tree View)", use_color)
    );
    println!();
    for (i, path) in paths.iter().enumerate() {
        let entry = PathEntry::new(i, path.clone(), paths);
        let is_last = i == paths.len() - 1;
        print_tree_entry(&entry, is_last, use_color);
    }
    print_summary(paths, use_color);
}

fn print_simple_entry(entry: &PathEntry, use_color: bool) {
    let index_str = format!("[{}]", entry.index);
    let status = if entry.exists { "✓" } else { "✗" };
    let line = if use_color {
        let colored_index = index_str.bright_black();
        let colored_status = if entry.exists {
            status.green()
        } else {
            status.red()
        };
        let colored_path = if entry.has_issues() {
            entry.path.yellow()
        } else if entry.is_user_specific {
            entry.path.cyan()
        } else {
            entry.path.normal()
        };
        format!("{} {} {}", colored_index, colored_status, colored_path)
    } else {
        format!("{} {} {}", index_str, status, entry.path)
    };
    println!("{}", line);
    let warnings = entry.get_warnings();
    if !warnings.is_empty() {
        for warning in warnings {
            let warning_line = if use_color {
                format!("    ⚠ {}", warning).yellow()
            } else {
                format!("    ! {}", warning).normal()
            };
            println!("{}", warning_line);
        }
    }
}

fn print_tree_entry(entry: &PathEntry, is_last: bool, use_color: bool) {
    let branch = if is_last { "└─" } else { "├─" };
    let index_str = format!("[{}]", entry.index);
    let status = if entry.exists { "✓" } else { "✗" };
    let line = if use_color {
        let colored_branch = branch.bright_black();
        let colored_index = index_str.bright_black();
        let colored_status = if entry.exists {
            status.green()
        } else {
            status.red()
        };
        let colored_path = if entry.has_issues() {
            entry.path.yellow()
        } else if entry.is_user_specific {
            entry.path.cyan()
        } else {
            entry.path.normal()
        };
        format!(
            "{} {} {} {}",
            colored_branch, colored_index, colored_status, colored_path
        )
    } else {
        format!("{} {} {} {}", branch, index_str, status, entry.path)
    };
    println!("{}", line);
    let warnings = entry.get_warnings();
    if !warnings.is_empty() {
        let continuation = if is_last { "   " } else { "│  " };
        for (j, warning) in warnings.iter().enumerate() {
            let is_last_warning = j == warnings.len() - 1;
            let sub_branch = if is_last_warning { "└─" } else { "├─" };
            let warning_line = if use_color {
                format!("{}  {} ⚠ {}", continuation, sub_branch, warning).yellow()
            } else {
                format!("{}  {} ! {}", continuation, sub_branch, warning).normal()
            };
            println!("{}", warning_line);
        }
    }
    if entry.is_user_specific && warnings.is_empty() {
        let continuation = if is_last { "   " } else { "│  " };
        let info_line = if use_color {
            format!("{}  └─ ℹ User-specific path", continuation).cyan()
        } else {
            format!("{}  └─ i User-specific path", continuation).normal()
        };
        println!("{}", info_line);
    }
}

fn format_header(text: &str, use_color: bool) -> ColoredString {
    if use_color {
        text.bold().bright_blue()
    } else {
        text.normal()
    }
}

fn print_summary(paths: &[String], use_color: bool) {
    println!();
    println!("{}", format_header("Summary", use_color));
    let entries: Vec<PathEntry> = paths
        .iter()
        .enumerate()
        .map(|(i, p)| PathEntry::new(i, p.clone(), paths))
        .collect();

    let total = entries.len();
    let existing = entries.iter().filter(|e| e.exists).count();
    let with_issues = entries.iter().filter(|e| e.has_issues()).count();
    let user_specific = entries.iter().filter(|e| e.is_user_specific).count();
    let duplicates = entries.iter().filter(|e| e.is_duplicate).count();
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
