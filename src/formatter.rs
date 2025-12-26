//! Console output formatting for spath results.
//!
//! This module separates presentation logic from data models,
//! providing formatted console output for scan, analysis, fix, and migration results.

use colored::*;

use crate::analyzer::{AnalysisResults, PathCategory, PathEntry, PathLocation};
use crate::fixer::FixResults;
use crate::migrator::{ActionType, MigrationPlan};
use crate::scanner::{IssueLevel, ScanResults};

/// Formatter for console output.
pub struct ConsoleFormatter;

impl ConsoleFormatter {
    /// Prints scan results with issues.
    pub fn print_scan_results(results: &ScanResults, verbose: bool) {
        for issue in &results.issues {
            match issue.level {
                IssueLevel::Critical => {
                    println!("{} {}", "[CRITICAL]".red().bold(), issue.path.yellow());
                    println!("    {}", issue.message.red());
                }
                IssueLevel::Warning => {
                    println!("{} {}", "[WARNING]".yellow().bold(), issue.path);
                    println!("    {}", issue.message.yellow());
                }
                IssueLevel::Info => {
                    if verbose {
                        println!("{} {}", "[INFO]".blue().bold(), issue.path);
                        println!("    {}", issue.message.blue());
                    }
                }
            }
            println!();
        }
        if results.issues.is_empty() {
            println!("{}", "No security issues found.".green().bold());
        }
    }

    /// Prints scan summary with issue counts.
    pub fn print_scan_summary(results: &ScanResults) {
        let critical = results
            .issues
            .iter()
            .filter(|i| matches!(i.level, IssueLevel::Critical))
            .count();
        let warning = results
            .issues
            .iter()
            .filter(|i| matches!(i.level, IssueLevel::Warning))
            .count();
        let info = results
            .issues
            .iter()
            .filter(|i| matches!(i.level, IssueLevel::Info))
            .count();
        println!("{}", "Summary:".bold());
        println!("  Total paths: {}", results.paths.len());
        println!("  {} Critical issues", critical.to_string().red().bold());
        println!("  {} Warnings", warning.to_string().yellow().bold());
        println!("  {} Info", info.to_string().blue());
    }

    /// Prints detailed audit report.
    pub fn print_scan_audit(results: &ScanResults) {
        println!();
        println!("{}", "Detailed Audit Report".bold().cyan());
        println!();
        println!("{}", "Path Statistics:".bold());
        println!(
            "  Total paths in PATH: {}",
            results.audit.total_paths.to_string().bold()
        );
        println!(
            "  Valid paths: {}",
            results.audit.valid_paths.to_string().green()
        );
        println!();
        println!("{}", "Security Issues:".bold());
        println!(
            "  {} Unquoted paths with spaces (CRITICAL)",
            results.audit.unquoted_with_spaces.to_string().red().bold()
        );
        println!("    These paths are vulnerable to DLL hijacking and privilege escalation");
        println!();
        println!("{}", "Path Quality Issues:".bold());
        println!(
            "  {} Non-existent paths",
            results.audit.non_existent.to_string().yellow()
        );
        println!("    These paths don't exist on the filesystem");
        println!(
            "  {} Relative paths",
            results.audit.relative_paths.to_string().yellow()
        );
        println!("    Should use absolute paths for consistency");
        println!();
        println!("{}", "Good Practices:".bold());
        println!(
            "  {} Properly quoted paths with spaces",
            results.audit.properly_quoted.to_string().green()
        );
        println!();
        let health_score = if results.audit.total_paths > 0 {
            ((results.audit.valid_paths as f64 / results.audit.total_paths as f64) * 100.0) as u32
        } else {
            0
        };
        let health_color = match health_score {
            90..=100 => "green",
            70..=89 => "yellow",
            _ => "red",
        };
        println!("{}", "PATH Health Score:".bold());
        println!(
            "  {}% {}",
            health_score.to_string().color(health_color).bold(),
            match health_score {
                90..=100 => "Excellent",
                70..=89 => "Good",
                50..=69 => "Fair",
                _ => "Poor - Immediate attention required",
            }
        );
    }

    /// Prints analysis results with categorized issues.
    pub fn print_analysis_results(results: &AnalysisResults) {
        println!("{}", "System PATH Analysis".bold().cyan());
        println!();
        let misplaced: Vec<_> = results
            .entries
            .iter()
            .filter(|e| e.should_be_in_user_path())
            .collect();
        if !misplaced.is_empty() {
            println!(
                "{}",
                "User Paths in SYSTEM PATH (should be moved):"
                    .yellow()
                    .bold()
            );
            println!();
            for entry in &misplaced {
                let status = if entry.needs_quotes() {
                    format!("{} + {}", "MISPLACED".yellow(), "UNQUOTED".red())
                } else {
                    "MISPLACED".yellow().to_string()
                };
                println!("  [{}] {}", status, entry.path);
                if !entry.exists {
                    println!("      Path does not exist");
                }
            }
            println!();
        }
        let unquoted_system: Vec<_> = results
            .entries
            .iter()
            .filter(|e| {
                matches!(e.location, PathLocation::System)
                    && matches!(e.category, PathCategory::SystemProgram)
                    && e.needs_quotes()
            })
            .collect();
        if !unquoted_system.is_empty() {
            println!("{}", "System Paths Needing Quotes:".red().bold());
            println!();
            for entry in &unquoted_system {
                println!("  [{}] {}", "UNQUOTED".red(), entry.path);
            }
            println!();
        }
        let unquoted_user: Vec<_> = results
            .entries
            .iter()
            .filter(|e| matches!(e.location, PathLocation::User) && e.needs_quotes())
            .collect();
        if !unquoted_user.is_empty() {
            println!("{}", "User Paths Needing Quotes:".yellow().bold());
            println!();
            for entry in &unquoted_user {
                println!("  [{}] {}", "UNQUOTED".yellow(), entry.path);
            }
            println!();
        }
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for entry in &results.entries {
            let normalized = entry.path.trim_matches('"').to_lowercase();
            if !seen.insert(normalized.clone()) {
                duplicates.push(entry);
            }
        }
        if !duplicates.is_empty() {
            println!("{}", "Duplicate Paths:".blue().bold());
            println!();
            for entry in &duplicates {
                let loc = match entry.location {
                    PathLocation::System => "SYSTEM",
                    PathLocation::User => "USER",
                };
                println!("  [{}] {}", loc.blue(), entry.path);
            }
            println!();
        }
        Self::print_analysis_summary(
            results,
            &misplaced,
            &unquoted_system,
            &unquoted_user,
            &duplicates,
        );
    }

    fn print_analysis_summary(
        results: &AnalysisResults,
        misplaced: &[&PathEntry],
        unquoted_system: &[&PathEntry],
        unquoted_user: &[&PathEntry],
        duplicates: &[&PathEntry],
    ) {
        println!("{}", "Summary:".bold());
        println!();
        let system_count = results
            .entries
            .iter()
            .filter(|e| matches!(e.location, PathLocation::System))
            .count();
        let user_count = results
            .entries
            .iter()
            .filter(|e| matches!(e.location, PathLocation::User))
            .count();
        println!(
            "  Total paths: {}",
            (system_count + user_count).to_string().bold()
        );
        println!("    SYSTEM PATH: {}", system_count);
        println!("    USER PATH: {}", user_count);
        println!();
        println!("{}", "Issues Found:".bold());
        println!(
            "  {} User paths in SYSTEM PATH (should be moved)",
            misplaced.len().to_string().yellow().bold()
        );
        println!(
            "  {} System paths needing quotes (requires admin)",
            unquoted_system.len().to_string().red().bold()
        );
        println!(
            "  {} User paths needing quotes",
            unquoted_user.len().to_string().yellow().bold()
        );
        println!(
            "  {} Duplicate paths",
            duplicates.len().to_string().blue().bold()
        );
        println!();
        if !misplaced.is_empty() || !unquoted_system.is_empty() {
            println!("{}", "Recommendations:".bold().green());
            if !misplaced.is_empty() {
                println!("  Run 'spath clean --dry-run' to see cleanup plan");
            }
            if !unquoted_system.is_empty() {
                println!("  System paths require administrator rights to fix");
            }
            if !unquoted_user.is_empty() {
                println!("  Run 'spath fix' to fix user paths");
            }
        } else {
            println!("{}", "No major issues found.".green().bold());
        }
    }

    /// Prints fix results with changes.
    pub fn print_fix_results(results: &FixResults) {
        if results.changes.is_empty() {
            println!(
                "{}",
                "No issues found - PATH is already clean.".green().bold()
            );
            return;
        }
        println!("{}", "Changes to be applied:".bold());
        println!();
        for change in &results.changes {
            println!("  {}", change);
        }
        println!();
        if results.dry_run {
            println!(
                "{}",
                "This was a dry run - no changes were made.".yellow().bold()
            );
            println!("Run without --dry-run to apply these changes.");
        } else if results.changed {
            println!("{}", "Changes applied successfully.".green().bold());
        }
    }

    /// Prints migration plan with actions.
    pub fn print_migration_plan(plan: &MigrationPlan, dry_run: bool) {
        if plan.actions.is_empty() {
            println!(
                "{}",
                "No migration needed - PATH is already optimal."
                    .green()
                    .bold()
            );
            return;
        }
        println!("{}", "Migration Plan:".bold().cyan());
        println!();
        let duplicates_count = plan
            .actions
            .iter()
            .filter(|a| matches!(a.action_type, ActionType::RemoveDuplicate))
            .count();
        let moves_count = plan
            .actions
            .iter()
            .filter(|a| matches!(a.action_type, ActionType::MoveToUser))
            .count();
        let duplicates: Vec<_> = plan
            .actions
            .iter()
            .filter(|a| matches!(a.action_type, ActionType::RemoveDuplicate))
            .collect();
        let moves: Vec<_> = plan
            .actions
            .iter()
            .filter(|a| matches!(a.action_type, ActionType::MoveToUser))
            .collect();
        if !duplicates.is_empty() {
            println!("{}", "Remove Duplicates:".blue().bold());
            println!();
            for action in duplicates {
                let location = match action.from_location {
                    PathLocation::System => "SYSTEM",
                    PathLocation::User => "USER",
                };
                println!("  [{}] {}", location.blue(), action.path);
                println!("      {}", action.reason.dimmed());
            }
            println!();
        }
        if !moves.is_empty() {
            println!("{}", "Move to USER PATH:".yellow().bold());
            println!();
            for action in moves {
                println!("  [SYSTEM -> USER] {}", action.path.yellow());
                println!("      {}", action.reason.dimmed());
            }
            println!();
        }
        println!("{}", "Summary:".bold());
        println!("  Total actions: {}", plan.actions.len().to_string().bold());
        println!("  Duplicates to remove: {}", duplicates_count);
        println!("  Paths to move: {}", moves_count);
        println!();
        if plan.requires_admin {
            println!(
                "{}",
                "Administrator rights required for SYSTEM PATH changes"
                    .yellow()
                    .bold()
            );
            println!();
        }
        if dry_run {
            println!(
                "{}",
                "This is a DRY RUN - no changes will be made."
                    .yellow()
                    .bold()
            );
            println!("Run without --dry-run to apply these changes.");
        }
    }
}
