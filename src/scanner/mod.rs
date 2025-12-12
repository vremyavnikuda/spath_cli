use anyhow::{Context, Result};
use colored::*;
use std::env;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum IssueLevel {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct PathIssue {
    pub path: String,
    pub level: IssueLevel,
    pub message: String,
}

pub struct ScanResults {
    pub paths: Vec<String>,
    pub issues: Vec<PathIssue>,
    pub audit: AuditStats,
}

#[derive(Debug, Default)]
pub struct AuditStats {
    pub total_paths: usize,
    pub unquoted_with_spaces: usize,
    pub non_existent: usize,
    pub relative_paths: usize,
    pub properly_quoted: usize,
    pub valid_paths: usize,
}

impl ScanResults {
    pub fn print(&self, verbose: bool) {
        for issue in &self.issues {
            match issue.level {
                IssueLevel::Critical => {
                    println!("{} {}", "[CRITICAL]".red().bold(), issue.path.yellow());
                    println!("  └─ {}", issue.message.red());
                }
                IssueLevel::Warning => {
                    println!("{} {}", "[WARNING]".yellow().bold(), issue.path);
                    println!("  └─ {}", issue.message.yellow());
                }
                IssueLevel::Info => {
                    if verbose {
                        println!("{} {}", "[INFO]".blue().bold(), issue.path);
                        println!("  └─ {}", issue.message.blue());
                    }
                }
            }
            println!();
        }

        if self.issues.is_empty() {
            println!("{}", "✓ No security issues found!".green().bold());
        }
    }

    pub fn print_summary(&self) {
        let critical = self
            .issues
            .iter()
            .filter(|i| matches!(i.level, IssueLevel::Critical))
            .count();
        let warning = self
            .issues
            .iter()
            .filter(|i| matches!(i.level, IssueLevel::Warning))
            .count();
        let info = self
            .issues
            .iter()
            .filter(|i| matches!(i.level, IssueLevel::Info))
            .count();

        println!("{}", "Summary:".bold());
        println!("  Total paths: {}", self.paths.len());
        println!("  {} Critical issues", critical.to_string().red().bold());
        println!("  {} Warnings", warning.to_string().yellow().bold());
        println!("  {} Info", info.to_string().blue());
    }

    pub fn print_audit(&self) {
        println!();
        println!("{}", "=".repeat(50).cyan());
        println!("{}", "Detailed Audit Report".bold().cyan());
        println!("{}", "=".repeat(50).cyan());
        println!();

        println!("{}", "Path Statistics:".bold());
        println!(
            "  Total paths in PATH: {}",
            self.audit.total_paths.to_string().bold()
        );
        println!(
            "  Valid paths: {}",
            self.audit.valid_paths.to_string().green()
        );
        println!();

        println!("{}", "Security Issues:".bold());
        println!(
            "  {} Unquoted paths with spaces (CRITICAL)",
            self.audit.unquoted_with_spaces.to_string().red().bold()
        );
        println!("    └─ These paths are vulnerable to DLL hijacking and privilege escalation");
        println!();

        println!("{}", "Path Quality Issues:".bold());
        println!(
            "  {} Non-existent paths",
            self.audit.non_existent.to_string().yellow()
        );
        println!("    └─ These paths don't exist on the filesystem");
        println!(
            "  {} Relative paths",
            self.audit.relative_paths.to_string().yellow()
        );
        println!("    └─ Should use absolute paths for consistency");
        println!();

        println!("{}", "Good Practices:".bold());
        println!(
            "  {} Properly quoted paths with spaces",
            self.audit.properly_quoted.to_string().green()
        );
        println!();

        // Calculate health score
        let health_score = if self.audit.total_paths > 0 {
            ((self.audit.valid_paths as f64 / self.audit.total_paths as f64) * 100.0) as u32
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
}

pub struct PathScanner {
    path_var: String,
}

impl PathScanner {
    pub fn new() -> Result<Self> {
        let path_var = env::var("PATH").context("Failed to read PATH environment variable")?;

        Ok(Self { path_var })
    }

    pub fn scan(&self) -> Result<ScanResults> {
        let paths: Vec<String> = self
            .path_var
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let mut issues = Vec::new();
        let mut audit = AuditStats {
            total_paths: paths.len(),
            ..Default::default()
        };

        for path in &paths {
            let has_spaces = path.contains(' ');
            let is_quoted = path.starts_with('"');
            let exists = Path::new(path.trim_matches('"')).exists();
            let is_absolute = path.contains(':') || path.starts_with('"');

            // Track statistics
            if has_spaces && !is_quoted {
                audit.unquoted_with_spaces += 1;
            }

            if !exists {
                audit.non_existent += 1;
            }

            if !is_absolute && !path.is_empty() {
                audit.relative_paths += 1;
            }

            if has_spaces && is_quoted {
                audit.properly_quoted += 1;
            }

            // Valid path: exists, absolute, and if has spaces then quoted
            if exists && is_absolute && (!has_spaces || is_quoted) {
                audit.valid_paths += 1;
            }

            // Check for spaces without quotes
            if has_spaces && !is_quoted {
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Critical,
                    message: "Path contains spaces but is not quoted. This can be exploited!"
                        .to_string(),
                });
            } else if has_spaces && is_quoted && exists {
                // Properly quoted path with spaces - good!
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Info,
                    message: "Path is properly quoted".to_string(),
                });
            }

            // Check if path exists
            if !exists {
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Warning,
                    message: "Path does not exist".to_string(),
                });
            }

            // Check for relative paths
            if !is_absolute && !path.is_empty() {
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Warning,
                    message: "Relative path detected - should use absolute paths".to_string(),
                });
            }
        }

        Ok(ScanResults {
            paths,
            issues,
            audit,
        })
    }
}
