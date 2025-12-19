use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, Write};

mod analyzer;
mod fixer;
mod formatter;
mod migrator;
mod registry;
mod scanner;

use analyzer::SystemAnalyzer;
use fixer::PathFixer;
use formatter::ConsoleFormatter;
use migrator::PathMigrator;
use scanner::PathScanner;

fn ask_confirmation(message: &str) -> bool {
    print!("{} [y/N]: ", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let answer = input.trim().to_lowercase();
    answer == "y" || answer == "yes"
}

#[derive(Parser)]
#[command(name = "spath")]
#[command(bin_name = "spath")]
#[command(about = "Windows PATH security scanner and fixer", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan PATH for security issues
    Scan {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,

        /// Show detailed audit report
        #[arg(short, long)]
        audit: bool,

        /// Include SYSTEM PATH in scan (requires admin to fix)
        #[arg(short, long)]
        system: bool,
    },

    /// Fix PATH security issues
    Fix {
        /// Preview changes without applying them
        #[arg(short, long)]
        dry_run: bool,

        /// Ask for confirmation before each change
        #[arg(long)]
        delicate: bool,
    },

    /// Create a backup of current PATH
    Backup,

    /// List available backups
    ListBackups,

    /// Restore PATH from a backup
    Restore {
        /// Backup file to restore from
        backup_file: String,

        /// Ask for confirmation before restoring
        #[arg(long)]
        delicate: bool,
    },

    /// Analyze SYSTEM and USER PATH for issues
    Analyze,

    /// Clean and optimize PATH by removing duplicates
    Clean {
        /// Include SYSTEM PATH cleanup (requires admin)
        #[arg(short, long)]
        system: bool,

        /// Preview changes without applying them
        #[arg(short, long)]
        dry_run: bool,

        /// Ask for confirmation before each change
        #[arg(long)]
        delicate: bool,
    },

    /// Verify if critical issues are actually exploitable
    Verify {
        /// Include SYSTEM PATH in verification
        #[arg(short, long)]
        system: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            verbose,
            audit,
            system,
        } => {
            println!("{}", "spath - Windows PATH Security Scanner".bold().cyan());
            println!();

            if system {
                println!(
                    "{}",
                    "Scanning SYSTEM PATH (requires admin rights to fix)".yellow()
                );
                println!();
            }

            let scanner = PathScanner::new(system)?;
            let results = scanner.scan()?;

            ConsoleFormatter::print_scan_results(&results, verbose);

            println!();
            ConsoleFormatter::print_scan_summary(&results);

            if audit {
                ConsoleFormatter::print_scan_audit(&results);
            }
        }

        Commands::Fix { dry_run, delicate } => {
            println!("{}", "spath - PATH Fixer".bold().cyan());
            println!();

            if dry_run {
                println!(
                    "{}",
                    "Running in DRY RUN mode - no changes will be made"
                        .yellow()
                        .bold()
                );
                println!();
            }

            let fixer = PathFixer::new()?;

            if delicate && !dry_run {
                println!(
                    "{}",
                    "Delicate mode: You will be asked to confirm each change.".cyan()
                );
                println!();
                if !ask_confirmation("Proceed with fixing USER PATH?") {
                    println!("{}", "Operation cancelled.".yellow());
                    return Ok(());
                }
                println!();
            }

            let results = fixer.fix_user_path(dry_run)?;

            ConsoleFormatter::print_fix_results(&results);
        }

        Commands::Backup => {
            println!("{}", "spath - Create Backup".bold().cyan());
            println!();

            let fixer = PathFixer::new()?;
            fixer.create_backup()?;
        }

        Commands::ListBackups => {
            println!("{}", "spath - Available Backups".bold().cyan());
            println!();

            let fixer = PathFixer::new()?;
            let backups = fixer.list_backups()?;

            if backups.is_empty() {
                println!("{}", "No backups found.".yellow());
            } else {
                println!("Found {} backup(s):", backups.len());
                println!();
                for backup in backups {
                    println!("  {}", backup.display());
                }
            }
        }

        Commands::Restore {
            backup_file,
            delicate,
        } => {
            println!("{}", "spath - Restore Backup".bold().cyan());
            println!();

            let fixer = PathFixer::new()?;
            let backup_path = std::path::PathBuf::from(&backup_file);

            if delicate {
                println!("{}", "Delicate mode: Confirm restore operation.".cyan());
                println!("This will replace your current PATH with the backup.");
                println!();
                if !ask_confirmation(&format!("Restore from {}?", backup_path.display())) {
                    println!("{}", "Operation cancelled.".yellow());
                    return Ok(());
                }
                println!();
            }

            fixer.restore_backup(&backup_path)?;
        }

        Commands::Analyze => {
            println!("{}", "spath - System PATH Analyzer".bold().cyan());
            println!();

            let analyzer = SystemAnalyzer::new()?;
            let results = analyzer.analyze()?;

            ConsoleFormatter::print_analysis_results(&results);
        }

        Commands::Clean {
            system,
            dry_run,
            delicate,
        } => {
            println!("{}", "spath - PATH Cleanup".bold().cyan());
            println!();

            if dry_run {
                println!(
                    "{}",
                    "Running in DRY RUN mode - no changes will be made"
                        .yellow()
                        .bold()
                );
                println!();
            }

            let migrator = PathMigrator::new()?;
            let plan = migrator.plan_migration(true, system)?;

            ConsoleFormatter::print_migration_plan(&plan, dry_run);

            if !dry_run && !plan.actions.is_empty() {
                println!();

                if delicate {
                    println!("{}", "Delicate mode: Confirm the cleanup operation.".cyan());
                    if !ask_confirmation("Apply these changes?") {
                        println!("{}", "Operation cancelled.".yellow());
                        return Ok(());
                    }
                    println!();
                }

                migrator.execute_migration(&plan, dry_run)?;
                println!();
                println!("{}", "Cleanup completed.".green().bold());
                println!(
                    "{}",
                    "  Note: You may need to restart applications for changes to take effect."
                        .yellow()
                );
            }
        }

        Commands::Verify { system } => {
            println!("{}", "spath - Security Verification".bold().cyan());
            println!();

            if system {
                println!("{}", "Verifying SYSTEM PATH security...".yellow());
            } else {
                println!("{}", "Verifying USER PATH security...".yellow());
            }
            println!();

            let scanner = PathScanner::new(system)?;
            let results = scanner.scan()?;

            // Filter only critical issues
            let critical_issues: Vec<_> = results
                .issues
                .iter()
                .filter(|issue| matches!(issue.level, scanner::IssueLevel::Critical))
                .collect();

            if critical_issues.is_empty() {
                println!("{}", "✓ No critical security issues found!".green().bold());
                return Ok(());
            }

            println!(
                "{}",
                format!(
                    "Found {} critical issue(s). Verifying exploitability...",
                    critical_issues.len()
                )
                .yellow()
            );
            println!();

            let mut real_threats = 0;
            let mut false_positives = 0;

            for issue in &critical_issues {
                let path = &issue.path;
                let exploit_paths = generate_exploit_paths(path);
                let mut found_exploits = Vec::new();
                for exploit_path in &exploit_paths {
                    if std::path::Path::new(exploit_path).exists() {
                        found_exploits.push(exploit_path.clone());
                    }
                }

                if found_exploits.is_empty() {
                    false_positives += 1;
                    println!("{} {}", "✓".green(), path);
                    println!("  No exploit files found - safe for now");
                } else {
                    real_threats += 1;
                    println!("{} {}", "✗".red().bold(), path);
                    println!(
                        "  {} Potential exploit files found:",
                        "DANGER:".red().bold()
                    );
                    for exploit in found_exploits {
                        println!("    - {}", exploit.red());
                    }
                }
                println!();
            }

            println!();
            println!("{}", "Verification Summary:".bold());
            println!("  Total critical issues: {}", critical_issues.len());
            println!(
                "  {} Real threats (exploit files exist): {}",
                "✗".red(),
                real_threats
            );
            println!(
                "  {} Potential risks (no exploits yet): {}",
                "✓".green(),
                false_positives
            );

            if real_threats > 0 {
                println!();
                println!("{}", "⚠ IMMEDIATE ACTION REQUIRED!".red().bold());
                println!("  Malicious files detected that could exploit your PATH.");
                println!("  Remove these files or fix your PATH immediately.");
            } else {
                println!();
                println!("{}", "Current Status: SAFE".green().bold());
                println!("  No active exploits detected, but paths are vulnerable.");
                println!("  Consider fixing these issues to prevent future attacks.");
            }
        }
    }

    Ok(())
}

/// Generates potential exploit file paths for an unquoted path with spaces.
///
/// For example, `"C:\Program Files\App\bin"` could be exploited by:
/// - `C:\Program.exe`, `C:\Program.com`, `C:\Program.bat`, `C:\Program.cmd`
/// - `C:\Program Files\App.exe`, etc.
fn generate_exploit_paths(path: &str) -> Vec<String> {
    let mut exploits = Vec::new();
    let path_lower = path.to_lowercase();
    let clean_path = path.trim_matches('"');
    let parts: Vec<&str> = clean_path.split(' ').collect();
    if parts.len() < 2 {
        return exploits;
    }
    if path_lower.starts_with("c:\\program files") {
        exploits.push("C:\\Program.exe".to_string());
        exploits.push("C:\\Program.com".to_string());
        exploits.push("C:\\Program.bat".to_string());
        exploits.push("C:\\Program.cmd".to_string());
    }
    if path_lower.contains("\\common files") {
        exploits.push("C:\\Program Files\\Common.exe".to_string());
        exploits.push("C:\\Program Files\\Common.com".to_string());
        exploits.push("C:\\Program Files (x86)\\Common.exe".to_string());
        exploits.push("C:\\Program Files (x86)\\Common.com".to_string());
    }
    let mut accumulated = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            accumulated.push(' ');
        }
        accumulated.push_str(part);

        if i < parts.len() - 1 {
            for ext in &[".exe", ".com", ".bat", ".cmd"] {
                exploits.push(format!("{}{}", accumulated, ext));
            }
        }
    }
    exploits
}
