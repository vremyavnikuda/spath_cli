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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { verbose, audit } => {
            println!("{}", "spath - Windows PATH Security Scanner".bold().cyan());
            println!();

            let scanner = PathScanner::new()?;
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
    }

    Ok(())
}
