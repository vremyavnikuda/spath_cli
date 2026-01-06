//! PATH fixer for security issues.
use crate::backup::{BackupManager, BackupResult, RestoreResult};
use crate::registry::RegistryHelper;
use crate::utils::{expand_env_vars, quote_if_needed};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

pub struct FixResults {
    pub changes: Vec<String>,
    pub dry_run: bool,
    pub changed: bool,
    pub backup_created: Option<BackupResult>,
}

pub struct PathFixer {
    backup_manager: BackupManager,
}

impl PathFixer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            backup_manager: BackupManager::new()?,
        })
    }
    pub fn create_backup(&self) -> Result<BackupResult> {
        self.backup_manager.create()
    }
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        self.backup_manager.list()
    }
    pub fn restore_backup(&self, backup_file: &Path) -> Result<RestoreResult> {
        self.backup_manager.restore(backup_file)
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
        let backup_created = if !dry_run && changed {
            Some(self.apply_fix(&new_path)?)
        } else {
            None
        };
        Ok(FixResults {
            changes,
            dry_run,
            changed,
            backup_created,
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
            let quoted = quote_if_needed(trimmed);
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
    fn apply_fix(&self, new_path: &str) -> Result<BackupResult> {
        let backup_result = self.backup_manager.create()?;
        RegistryHelper::write_user_path(new_path)
            .context("Failed to write new PATH to registry")?;
        info!("PATH successfully updated in registry");
        Ok(backup_result)
    }
}
