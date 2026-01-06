use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, Write};
use tracing_subscriber::EnvFilter;

mod analyzer;
mod backup;
mod constants;
mod fixer;
mod formatter;
mod migrator;
mod models;
mod registry;
mod scanner;
mod security;
mod utils;
mod visualizer;

use analyzer::SystemAnalyzer;
use fixer::PathFixer;
use formatter::ConsoleFormatter;
use migrator::PathMigrator;
use models::IssueLevel;
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
    Scan {
        #[arg(short, long)]
        verbose: bool,
        #[arg(short, long)]
        audit: bool,
        #[arg(short, long)]
        system: bool,
    },
    Fix {
        #[arg(short, long)]
        dry_run: bool,
        #[arg(long)]
        delicate: bool,
    },
    Backup,
    ListBackups,
    Restore {
        backup_file: String,
        #[arg(long)]
        delicate: bool,
    },
    Analyze,
    Clean {
        #[arg(short, long)]
        system: bool,
        #[arg(short, long)]
        dry_run: bool,
        #[arg(long)]
        delicate: bool,
    },
    Verify {
        #[arg(short, long)]
        system: bool,
    },
    Visualize {
        #[arg(short, long)]
        tree: bool,
        #[arg(short, long)]
        system: bool,
        #[arg(short, long)]
        user: bool,
        #[arg(long)]
        no_color: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan {
            verbose,
            audit,
            system,
        } => handle_scan(verbose, audit, system),
        Commands::Fix { dry_run, delicate } => handle_fix(dry_run, delicate),
        Commands::Backup => handle_backup(),
        Commands::ListBackups => handle_list_backups(),
        Commands::Restore {
            backup_file,
            delicate,
        } => handle_restore(&backup_file, delicate),
        Commands::Analyze => handle_analyze(),
        Commands::Clean {
            system,
            dry_run,
            delicate,
        } => handle_clean(system, dry_run, delicate),
        Commands::Verify { system } => handle_verify(system),
        Commands::Visualize {
            tree,
            system,
            user,
            no_color,
        } => handle_visualize(tree, system, user, no_color),
    }
}

fn handle_scan(verbose: bool, audit: bool, system: bool) -> Result<()> {
    println!("{}", "spath - Windows PATH Security Scanner".bold().cyan());
    if system {
        println!(
            "{}",
            "Scanning SYSTEM PATH (requires admin rights to fix)".yellow()
        );
    }
    let scanner = PathScanner::new(system)?;
    let results = scanner.scan()?;
    ConsoleFormatter::print_scan_results(&results, verbose);
    ConsoleFormatter::print_scan_summary(&results);
    if audit {
        ConsoleFormatter::print_scan_audit(&results);
    }
    Ok(())
}

fn handle_fix(dry_run: bool, delicate: bool) -> Result<()> {
    println!("{}", "spath - PATH Fixer".bold().cyan());
    println!();
    if dry_run {
        println!(
            "{}",
            "Running in DRY RUN mode - no changes will be made"
                .yellow()
                .bold()
        );
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
    }
    let results = fixer.fix_user_path(dry_run)?;
    ConsoleFormatter::print_fix_results(&results);
    Ok(())
}

fn handle_backup() -> Result<()> {
    println!("{}", "spath - Create Backup".bold().cyan());
    println!();
    let fixer = PathFixer::new()?;
    let result = fixer.create_backup()?;
    ConsoleFormatter::print_backup_result(&result);
    Ok(())
}

fn handle_list_backups() -> Result<()> {
    println!("{}", "spath - Available Backups".bold().cyan());
    let fixer = PathFixer::new()?;
    let backups = fixer.list_backups()?;
    if backups.is_empty() {
        println!("{}", "No backups found.".yellow());
    } else {
        println!("Found {} backup(s):", backups.len());
        for backup in backups {
            println!("  {}", backup.display());
        }
    }
    Ok(())
}

fn handle_restore(backup_file: &str, delicate: bool) -> Result<()> {
    println!("{}", "spath - Restore Backup".bold().cyan());
    println!();
    let fixer = PathFixer::new()?;
    let backup_path = std::path::PathBuf::from(backup_file);
    if delicate {
        println!("{}", "Delicate mode: Confirm restore operation.".cyan());
        println!("This will replace your current PATH with the backup.");
        if !ask_confirmation(&format!("Restore from {}?", backup_path.display())) {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
        println!();
    }
    let result = fixer.restore_backup(&backup_path)?;
    ConsoleFormatter::print_restore_result(&result);
    Ok(())
}

fn handle_analyze() -> Result<()> {
    println!("{}", "spath - System PATH Analyzer".bold().cyan());
    let analyzer = SystemAnalyzer::new()?;
    let results = analyzer.analyze()?;
    ConsoleFormatter::print_analysis_results(&results);
    Ok(())
}

fn handle_clean(system: bool, dry_run: bool, delicate: bool) -> Result<()> {
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
    let analyzer = SystemAnalyzer::new()?;
    let analysis = analyzer.analyze()?;
    let migrator = PathMigrator::new()?;
    let plan = migrator.plan_migration(&analysis, true, system)?;
    ConsoleFormatter::print_migration_plan(&plan, dry_run);
    if !dry_run && !plan.actions.is_empty() {
        println!();
        if delicate {
            println!("{}", "Delicate mode: Confirm the cleanup operation.".cyan());
            if !ask_confirmation("Apply these changes?") {
                println!("{}", "Operation cancelled.".yellow());
                return Ok(());
            }
        }
        if plan.requires_admin {
            ConsoleFormatter::print_migration_requires_admin();
        }
        let result = migrator.execute_migration(&plan, dry_run)?;
        ConsoleFormatter::print_migration_result(&result);
        println!("{}", "Cleanup completed.".green().bold());
        println!(
            "{}",
            "  Note: You may need to restart applications for changes to take effect.".yellow()
        );
    }
    Ok(())
}

fn handle_verify(system: bool) -> Result<()> {
    println!("{}", "spath - Security Verification".bold().cyan());
    if system {
        println!("{}", "Verifying SYSTEM PATH security...".yellow());
    } else {
        println!("{}", "Verifying USER PATH security...".yellow());
    }
    let scanner = PathScanner::new(system)?;
    let results = scanner.scan()?;
    let critical_paths: Vec<&str> = results
        .issues
        .iter()
        .filter(|i| matches!(i.level, IssueLevel::Critical))
        .map(|i| i.path.as_str())
        .collect();
    if critical_paths.is_empty() {
        println!("{}", "✓ No critical security issues found!".green().bold());
        return Ok(());
    }
    println!(
        "{}",
        format!(
            "Found {} critical issue(s). Verifying exploitability...",
            critical_paths.len()
        )
        .yellow()
    );
    let (results, summary) = security::exploits::verify_paths(&critical_paths);
    ConsoleFormatter::print_verification_results(&results, &summary);
    Ok(())
}

fn handle_visualize(tree: bool, system: bool, user: bool, no_color: bool) -> Result<()> {
    let use_color = !no_color && atty::is(atty::Stream::Stdout);
    let (system_paths, user_paths) = get_paths_for_visualization(system, user);
    if system && !user {
        print_path_visualization("SYSTEM PATH", &system_paths, tree, use_color);
    } else if user && !system {
        print_path_visualization("USER PATH", &user_paths, tree, use_color);
    } else {
        print_path_visualization("SYSTEM PATH", &system_paths, tree, use_color);
        println!();
        print_path_visualization("USER PATH", &user_paths, tree, use_color);
    }
    Ok(())
}

fn get_paths_for_visualization(system: bool, user: bool) -> (Vec<String>, Vec<String>) {
    let show_both = !system && !user;
    let sys = if system || show_both {
        registry::RegistryHelper::read_system_path().unwrap_or_default()
    } else {
        Vec::new()
    };
    let usr = if user || show_both {
        registry::RegistryHelper::read_user_path().unwrap_or_default()
    } else {
        Vec::new()
    };
    (sys, usr)
}

fn print_path_visualization(title: &str, paths: &[String], tree: bool, use_color: bool) {
    println!("{}", title.bold().cyan());
    if tree {
        visualizer::visualize_tree(paths, use_color);
    } else {
        visualizer::visualize_simple(paths, use_color);
    }
}
