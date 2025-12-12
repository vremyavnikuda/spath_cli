use anyhow::{Context, Result};
use colored::*;
use std::env;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone)]
pub enum PathLocation {
    System,
    User,
}

#[derive(Debug, Clone)]
pub enum PathCategory {
    // C:\Program Files, C:\Windows
    SystemProgram,
    // C:\Users\username\...
    UserProgram,
    // C:\ProgramData
    ProgramData,
    // Unclear
    Ambiguous,
}

#[derive(Debug, Clone)]
pub struct PathEntry {
    pub path: String,
    pub location: PathLocation,
    pub category: PathCategory,
    pub has_spaces: bool,
    pub is_quoted: bool,
    pub exists: bool,
}

impl PathEntry {
    pub fn should_be_in_user_path(&self) -> bool {
        matches!(self.category, PathCategory::UserProgram)
            && matches!(self.location, PathLocation::System)
    }

    pub fn needs_quotes(&self) -> bool {
        self.has_spaces && !self.is_quoted
    }
}

pub struct SystemAnalyzer {
    current_username: String,
}

impl SystemAnalyzer {
    pub fn new() -> Result<Self> {
        let current_username = env::var("USERNAME").context("Failed to get current username")?;

        Ok(Self { current_username })
    }

    pub fn analyze(&self) -> Result<AnalysisResults> {
        let system_paths = self.read_system_path()?;
        let user_paths = self.read_user_path()?;

        let mut entries = Vec::new();

        // Analyze SYSTEM PATH
        for path in system_paths {
            let entry = self.analyze_path(&path, PathLocation::System);
            entries.push(entry);
        }

        // Analyze USER PATH
        for path in user_paths {
            let entry = self.analyze_path(&path, PathLocation::User);
            entries.push(entry);
        }

        Ok(AnalysisResults {
            entries,
            current_username: self.current_username.clone(),
        })
    }

    fn read_system_path(&self) -> Result<Vec<String>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env_key = hklm
            .open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")
            .context("Failed to open system environment key. Try running as administrator.")?;

        let path: String = env_key
            .get_value("Path")
            .context("Failed to read system PATH")?;

        Ok(path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect())
    }

    fn read_user_path(&self) -> Result<Vec<String>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu
            .open_subkey("Environment")
            .context("Failed to open user environment key")?;

        let path: String = env_key
            .get_value("Path")
            .context("Failed to read user PATH")?;

        Ok(path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect())
    }

    fn analyze_path(&self, path: &str, location: PathLocation) -> PathEntry {
        let trimmed = path.trim_matches('"');
        let has_spaces = path.contains(' ');
        let is_quoted = path.starts_with('"') && path.ends_with('"');
        let exists = Path::new(trimmed).exists();

        let category = self.categorize_path(trimmed);

        PathEntry {
            path: path.to_string(),
            location,
            category,
            has_spaces,
            is_quoted,
            exists,
        }
    }

    fn categorize_path(&self, path: &str) -> PathCategory {
        let lower = path.to_lowercase();

        // System programs
        if lower.starts_with("c:\\windows")
            || lower.starts_with("c:\\program files")
            || lower.starts_with("c:\\program files (x86)")
        {
            return PathCategory::SystemProgram;
        }

        // User programs
        if lower.contains(&format!(
            "c:\\users\\{}",
            self.current_username.to_lowercase()
        )) || lower.contains("\\appdata\\")
            || lower.contains("\\.cargo\\")
            || lower.contains("\\.dotnet\\")
            || lower.contains("\\.npm\\")
            || lower.contains("\\.bun\\")
            || lower.contains("\\.local\\")
        {
            return PathCategory::UserProgram;
        }

        // ProgramData (shared but not system)
        if lower.starts_with("c:\\programdata") {
            return PathCategory::ProgramData;
        }

        PathCategory::Ambiguous
    }
}

pub struct AnalysisResults {
    pub entries: Vec<PathEntry>,
    #[allow(dead_code)]
    pub current_username: String,
}

impl AnalysisResults {
    pub fn print(&self) {
        println!("{}", "System PATH Analysis".bold().cyan());
        println!("{}", "=".repeat(70).cyan());
        println!();

        // Find misplaced paths
        let misplaced: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.should_be_in_user_path())
            .collect();

        if !misplaced.is_empty() {
            println!(
                "{}",
                "⚠ User Paths in SYSTEM PATH (should be moved):"
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
                    println!("      └─ {} Path does not exist", "⚠".yellow());
                }
            }
            println!();
        }

        // Find unquoted system paths
        let unquoted_system: Vec<_> = self
            .entries
            .iter()
            .filter(|e| {
                matches!(e.location, PathLocation::System)
                    && matches!(e.category, PathCategory::SystemProgram)
                    && e.needs_quotes()
            })
            .collect();

        if !unquoted_system.is_empty() {
            println!("{}", "🔒 System Paths Needing Quotes:".red().bold());
            println!();
            for entry in &unquoted_system {
                println!("  [{}] {}", "UNQUOTED".red(), entry.path);
            }
            println!();
        }

        // Find unquoted user paths
        let unquoted_user: Vec<_> = self
            .entries
            .iter()
            .filter(|e| matches!(e.location, PathLocation::User) && e.needs_quotes())
            .collect();

        if !unquoted_user.is_empty() {
            println!("{}", "🔓 User Paths Needing Quotes:".yellow().bold());
            println!();
            for entry in &unquoted_user {
                println!("  [{}] {}", "UNQUOTED".yellow(), entry.path);
            }
            println!();
        }

        // Find duplicates
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for entry in &self.entries {
            let normalized = entry.path.trim_matches('"').to_lowercase();
            if !seen.insert(normalized.clone()) {
                duplicates.push(entry);
            }
        }

        if !duplicates.is_empty() {
            println!("{}", "🔄 Duplicate Paths:".blue().bold());
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

        // Summary
        println!("{}", "=".repeat(70).cyan());
        self.print_summary(&misplaced, &unquoted_system, &unquoted_user, &duplicates);
    }

    fn print_summary(
        &self,
        misplaced: &[&PathEntry],
        unquoted_system: &[&PathEntry],
        unquoted_user: &[&PathEntry],
        duplicates: &[&PathEntry],
    ) {
        println!("{}", "Summary:".bold());
        println!();

        let system_count = self
            .entries
            .iter()
            .filter(|e| matches!(e.location, PathLocation::System))
            .count();
        let user_count = self
            .entries
            .iter()
            .filter(|e| matches!(e.location, PathLocation::User))
            .count();

        println!(
            "  Total paths: {}",
            (system_count + user_count).to_string().bold()
        );
        println!("    • SYSTEM PATH: {}", system_count);
        println!("    • USER PATH: {}", user_count);
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
                println!("  • Run 'spath clean --dry-run' to see cleanup plan");
            }
            if !unquoted_system.is_empty() {
                println!("  • System paths require administrator rights to fix");
            }
            if !unquoted_user.is_empty() {
                println!("  • Run 'spath fix' to fix user paths");
            }
        } else {
            println!("{}", "✓ No major issues found!".green().bold());
        }
    }
}
