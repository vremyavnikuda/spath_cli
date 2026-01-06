//! PATH fixer for security issues.
use crate::constants::{
    BACKUP_DIR_NAME, BACKUP_FILE_EXTENSION, BACKUP_FILE_PREFIX, BACKUP_TIMESTAMP_FORMAT,
    MAX_BACKUPS,
};
use crate::registry::RegistryHelper;
use crate::security::acl;
use crate::utils::expand_env_vars;
use anyhow::{bail, Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct PathBackup {
    pub timestamp: String,
    pub user_path: String,
    pub system_path: Option<String>,
}

pub struct FixResults {
    pub changes: Vec<String>,
    pub dry_run: bool,
    pub changed: bool,
}

pub struct PathFixer {
    backup_dir: PathBuf,
}

impl PathFixer {
    pub fn new() -> Result<Self> {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .context("Failed to get LOCALAPPDATA environment variable")?;
        let backup_dir = PathBuf::from(local_app_data)
            .join("spath")
            .join(BACKUP_DIR_NAME);
        fs::create_dir_all(&backup_dir).context("Failed to create backup directory")?;
        Ok(Self { backup_dir })
    }
    pub fn create_backup(&self) -> Result<PathBuf> {
        info!("Creating PATH backup");
        let backup = self.build_backup()?;
        let backup_file = self.backup_dir.join(format!(
            "{}{}.{}",
            BACKUP_FILE_PREFIX, backup.timestamp, BACKUP_FILE_EXTENSION
        ));
        debug!("Backup file path: {}", backup_file.display());
        self.write_backup_file(&backup_file, &backup)?;
        self.set_backup_acl(&backup_file)?;
        info!("Backup created successfully: {}", backup_file.display());
        println!(
            "{} {}",
            "Backup created:".green().bold(),
            backup_file.display()
        );
        self.cleanup_old_backups()?;
        Ok(backup_file)
    }
    fn build_backup(&self) -> Result<PathBackup> {
        let user_path = RegistryHelper::read_user_path_raw()
            .context("Failed to read user PATH from registry")?;
        let system_path = RegistryHelper::read_system_path_raw().ok();
        Ok(PathBackup {
            timestamp: chrono::Local::now()
                .format(BACKUP_TIMESTAMP_FORMAT)
                .to_string(),
            user_path,
            system_path,
        })
    }
    fn write_backup_file(&self, path: &Path, backup: &PathBackup) -> Result<()> {
        let json = serde_json::to_string_pretty(backup).context("Failed to serialize backup")?;
        fs::write(path, json).context("Failed to write backup file")
    }
    fn set_backup_acl(&self, path: &Path) -> Result<()> {
        acl::set_user_only_acl(path).context("Failed to set ACL on backup file. Backup created but may be accessible to other users.")
    }
    pub fn fix_user_path(&self, dry_run: bool) -> Result<FixResults> {
        info!("Starting USER PATH fix (dry_run: {})", dry_run);
        let current_path = RegistryHelper::read_user_path_raw()
            .context("Failed to read user PATH from registry")?;
        let paths = RegistryHelper::parse_path_string(&current_path);
        debug!("Found {} path entries to process", paths.len());
        let (fixed_paths, changes) = self.process_paths(paths);
        let new_path = fixed_paths.join(";");
        let changed = new_path != current_path;
        info!(
            "PATH fix completed: {} changes, changed: {}",
            changes.len(),
            changed
        );
        if !dry_run && changed {
            self.apply_fix(&new_path)?;
        }
        Ok(FixResults {
            changes,
            dry_run,
            changed,
        })
    }
    fn process_paths(&self, paths: Vec<String>) -> (Vec<String>, Vec<String>) {
        let mut fixed_paths = Vec::new();
        let mut changes = Vec::new();
        let mut seen = HashSet::new();
        for path in paths {
            self.process_single_path(&path, &mut fixed_paths, &mut changes, &mut seen);
        }
        (fixed_paths, changes)
    }
    fn process_single_path(
        &self,
        path: &str,
        fixed_paths: &mut Vec<String>,
        changes: &mut Vec<String>,
        seen: &mut HashSet<String>,
    ) {
        let trimmed = path.trim();
        if seen.contains(trimmed) {
            warn!("Duplicate path found: {}", trimmed);
            changes.push(format!("Removed duplicate: {}", trimmed));
            return;
        }
        seen.insert(trimmed.to_string());
        if self.should_remove_path(trimmed) {
            warn!("Non-existent path found: {}", trimmed);
            changes.push(format!("Removed non-existent: {}", trimmed));
            return;
        }
        if trimmed.contains(' ') && !trimmed.starts_with('"') {
            let quoted = format!("\"{}\"", trimmed);
            info!("Adding quotes to path: {}", trimmed);
            changes.push(format!("Added quotes: {} -> {}", trimmed, quoted));
            fixed_paths.push(quoted);
        } else {
            fixed_paths.push(trimmed.to_string());
        }
    }
    fn should_remove_path(&self, trimmed: &str) -> bool {
        let path_to_check = trimmed.trim_matches('"');
        let exists = Path::new(path_to_check).exists();
        if exists {
            return false;
        }
        if trimmed.contains('%') {
            let expanded = expand_env_vars(trimmed);
            let expanded_exists = Path::new(&expanded).exists();
            return !expanded_exists || expanded == trimmed;
        }
        true
    }
    fn apply_fix(&self, new_path: &str) -> Result<()> {
        self.create_backup()?;
        RegistryHelper::write_user_path(new_path)
            .context("Failed to write new PATH to registry")?;
        info!("PATH successfully updated in registry");
        println!();
        println!("{}", "PATH has been fixed.".green().bold());
        println!(
            "{}",
            "  Note: You may need to restart applications for changes to take effect.".yellow()
        );
        Ok(())
    }
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups = Vec::new();
        if !self.backup_dir.exists() {
            return Ok(backups);
        }
        for entry in fs::read_dir(&self.backup_dir)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                backups.push(path);
            }
        }
        backups.sort();
        backups.reverse();
        Ok(backups)
    }
    fn cleanup_old_backups(&self) -> Result<()> {
        let mut backups = self.list_backups()?;
        if backups.len() > MAX_BACKUPS {
            info!(
                "Cleaning up old backups: {} > {}",
                backups.len(),
                MAX_BACKUPS
            );
        }
        while backups.len() > MAX_BACKUPS {
            if let Some(oldest) = backups.pop() {
                debug!("Removing old backup: {}", oldest.display());
                fs::remove_file(&oldest).with_context(|| {
                    format!("Failed to remove old backup: {}", oldest.display())
                })?;
                info!("Removed old backup: {}", oldest.display());
                println!("{} Removed old backup: {}", "✓".green(), oldest.display());
            }
        }
        Ok(())
    }
    pub fn restore_backup(&self, backup_file: &PathBuf) -> Result<()> {
        info!("Restoring PATH from backup: {}", backup_file.display());
        self.validate_backup_path(backup_file)?;
        let json = fs::read_to_string(backup_file).context("Failed to read backup file")?;
        let backup: PathBackup =
            serde_json::from_str(&json).context("Failed to parse backup file")?;
        RegistryHelper::write_user_path(&backup.user_path)
            .context("Failed to restore PATH from backup")?;
        info!("PATH successfully restored from backup");
        println!("{}", "PATH restored from backup.".green().bold());
        println!(
            "{}",
            "  Note: You may need to restart applications for changes to take effect.".yellow()
        );
        Ok(())
    }
    fn validate_backup_path(&self, backup_file: &Path) -> Result<()> {
        let canonical_backup_dir = self
            .backup_dir
            .canonicalize()
            .context("Failed to resolve backup directory path")?;
        let canonical_file = backup_file
            .canonicalize()
            .context("Backup file does not exist or path is invalid")?;
        if !canonical_file.starts_with(&canonical_backup_dir) {
            bail!("Security error: Backup file must be located in {}\nUse 'spath list-backups' to see available backups.", self.backup_dir.display());
        }
        if backup_file.extension().and_then(|s| s.to_str()) != Some(BACKUP_FILE_EXTENSION) {
            bail!(
                "Security error: Backup file must have .{} extension",
                BACKUP_FILE_EXTENSION
            );
        }
        if let Some(file_name) = backup_file.file_name().and_then(|s| s.to_str()) {
            if !file_name.starts_with(BACKUP_FILE_PREFIX) {
                bail!("Security error: Invalid backup file name format.\nExpected: path_backup_YYYYMMDD_HHMMSS.json");
            }
        } else {
            bail!("Security error: Invalid backup file name");
        }
        Ok(())
    }
}
