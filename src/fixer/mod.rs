use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Serialize, Deserialize)]
pub struct PathBackup {
    pub timestamp: String,
    pub user_path: String,
    pub system_path: Option<String>,
}

pub struct PathFixer {
    backup_dir: PathBuf,
}

impl PathFixer {
    pub fn new() -> Result<Self> {
        let local_app_data =
            env::var("LOCALAPPDATA").context("Failed to get LOCALAPPDATA environment variable")?;

        let backup_dir = PathBuf::from(local_app_data).join("spath").join("backups");

        fs::create_dir_all(&backup_dir).context("Failed to create backup directory")?;

        Ok(Self { backup_dir })
    }

    pub fn create_backup(&self) -> Result<PathBuf> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu
            .open_subkey("Environment")
            .context("Failed to open HKCU\\Environment")?;

        let user_path: String = env_key
            .get_value("Path")
            .context("Failed to read user PATH from registry")?;

        // Try to read system PATH (may fail without admin rights)
        let system_path = self.read_system_path().ok();

        let backup = PathBackup {
            timestamp: chrono::Local::now().format("%Y%m%d_%H%M%S").to_string(),
            user_path,
            system_path,
        };

        let backup_file = self
            .backup_dir
            .join(format!("path_backup_{}.json", backup.timestamp));
        let json = serde_json::to_string_pretty(&backup).context("Failed to serialize backup")?;

        fs::write(&backup_file, json).context("Failed to write backup file")?;

        println!(
            "{} {}",
            "✓ Backup created:".green().bold(),
            backup_file.display()
        );

        Ok(backup_file)
    }

    fn read_system_path(&self) -> Result<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env_key = hklm
            .open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")
            .context("Failed to open system environment key")?;

        env_key
            .get_value("Path")
            .context("Failed to read system PATH")
    }

    pub fn fix_user_path(&self, dry_run: bool) -> Result<FixResults> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu
            .open_subkey("Environment")
            .context("Failed to open HKCU\\Environment")?;

        let current_path: String = env_key
            .get_value("Path")
            .context("Failed to read user PATH from registry")?;

        let paths: Vec<String> = current_path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let mut fixed_paths = Vec::new();
        let mut changes = Vec::new();
        let mut seen = HashSet::new();

        for path in paths {
            let trimmed = path.trim();

            // Skip duplicates
            if seen.contains(trimmed) {
                changes.push(format!("Removed duplicate: {}", trimmed.yellow()));
                continue;
            }
            seen.insert(trimmed.to_string());

            // Fix unquoted paths with spaces
            if trimmed.contains(' ') && !trimmed.starts_with('"') {
                let quoted = format!("\"{}\"", trimmed);
                changes.push(format!("Fixed: {} → {}", trimmed.red(), quoted.green()));
                fixed_paths.push(quoted);
            } else {
                fixed_paths.push(trimmed.to_string());
            }
        }

        let new_path = fixed_paths.join(";");
        let changed = new_path != current_path;

        if !dry_run && changed {
            // Create backup before making changes
            self.create_backup()?;

            // Write new PATH to registry
            let env_key = hkcu
                .open_subkey_with_flags("Environment", KEY_WRITE)
                .context("Failed to open HKCU\\Environment for writing")?;

            env_key
                .set_value("Path", &new_path)
                .context("Failed to write new PATH to registry")?;

            println!();
            println!("{}", "✓ PATH has been fixed!".green().bold());
            println!(
                "{}",
                "  Note: You may need to restart applications for changes to take effect.".yellow()
            );
        }

        Ok(FixResults {
            changes,
            dry_run,
            changed,
        })
    }

    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups = Vec::new();

        if !self.backup_dir.exists() {
            return Ok(backups);
        }

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                backups.push(path);
            }
        }

        backups.sort();
        backups.reverse();

        Ok(backups)
    }

    pub fn restore_backup(&self, backup_file: &PathBuf) -> Result<()> {
        let json = fs::read_to_string(backup_file).context("Failed to read backup file")?;

        let backup: PathBackup =
            serde_json::from_str(&json).context("Failed to parse backup file")?;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu
            .open_subkey_with_flags("Environment", KEY_WRITE)
            .context("Failed to open HKCU\\Environment for writing")?;

        env_key
            .set_value("Path", &backup.user_path)
            .context("Failed to restore PATH from backup")?;

        println!("{}", "✓ PATH restored from backup!".green().bold());
        println!(
            "{}",
            "  Note: You may need to restart applications for changes to take effect.".yellow()
        );

        Ok(())
    }
}

pub struct FixResults {
    pub changes: Vec<String>,
    pub dry_run: bool,
    pub changed: bool,
}

impl FixResults {
    pub fn print(&self) {
        if self.changes.is_empty() {
            println!(
                "{}",
                "✓ No issues found - PATH is already clean!".green().bold()
            );
            return;
        }

        println!("{}", "Changes to be applied:".bold());
        println!();

        for change in &self.changes {
            println!("  • {}", change);
        }

        println!();

        if self.dry_run {
            println!(
                "{}",
                "This was a dry run - no changes were made.".yellow().bold()
            );
            println!("Run without --dry-run to apply these changes.");
        } else if self.changed {
            println!("{}", "✓ Changes applied successfully!".green().bold());
        }
    }
}
