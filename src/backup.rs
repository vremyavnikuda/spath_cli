use crate::constants::{
    BACKUP_DIR_NAME, BACKUP_FILE_EXTENSION, BACKUP_FILE_PREFIX, BACKUP_TIMESTAMP_FORMAT,
    MAX_BACKUPS,
};
use crate::registry::RegistryHelper;
use crate::security::acl;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathBackup {
    pub timestamp: String,
    pub user_path: String,
    pub system_path: Option<String>,
}

#[derive(Debug)]
pub struct BackupResult {
    pub path: PathBuf,
    pub cleaned_backups: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct RestoreResult {
    pub restored_from: PathBuf,
}

impl RestoreResult {
    pub fn path(&self) -> &PathBuf {
        &self.restored_from
    }
}

pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new() -> Result<Self> {
        let local_app_data =
            std::env::var("LOCALAPPDATA").context("LOCALAPPDATA environment variable not set")?;
        let backup_dir = PathBuf::from(local_app_data)
            .join("spath")
            .join(BACKUP_DIR_NAME);
        fs::create_dir_all(&backup_dir).context("Failed to create backup directory")?;
        Ok(Self { backup_dir })
    }
    pub fn create(&self) -> Result<BackupResult> {
        info!("Creating PATH backup");
        let backup = self.build_backup()?;
        let backup_file = self.build_backup_path(&backup.timestamp);
        debug!("Writing backup to: {}", backup_file.display());
        self.write_backup(&backup_file, &backup)?;
        self.set_acl(&backup_file);
        let cleaned = self.cleanup_old()?;
        info!("Backup created: {}", backup_file.display());
        Ok(BackupResult {
            path: backup_file,
            cleaned_backups: cleaned,
        })
    }
    fn build_backup(&self) -> Result<PathBackup> {
        let user_path = RegistryHelper::read_user_path_raw().context("Failed to read user PATH")?;
        let system_path = RegistryHelper::read_system_path_raw().ok();
        let timestamp = chrono::Local::now()
            .format(BACKUP_TIMESTAMP_FORMAT)
            .to_string();
        Ok(PathBackup {
            timestamp,
            user_path,
            system_path,
        })
    }
    fn build_backup_path(&self, timestamp: &str) -> PathBuf {
        self.backup_dir.join(format!(
            "{}{}.{}",
            BACKUP_FILE_PREFIX, timestamp, BACKUP_FILE_EXTENSION
        ))
    }
    fn write_backup(&self, path: &Path, backup: &PathBackup) -> Result<()> {
        let json = serde_json::to_string_pretty(backup).context("Failed to serialize backup")?;
        fs::write(path, json).context("Failed to write backup file")
    }
    fn set_acl(&self, path: &Path) {
        if let Err(e) = acl::set_user_only_acl(path) {
            warn!(
                "Failed to set ACL on backup: {}. Backup may be accessible to others.",
                e
            );
        }
    }
    pub fn restore(&self, backup_file: &Path) -> Result<RestoreResult> {
        info!("Restoring PATH from: {}", backup_file.display());
        self.validate_path(backup_file)?;
        let json = fs::read_to_string(backup_file).context("Failed to read backup file")?;
        let backup: PathBackup =
            serde_json::from_str(&json).context("Failed to parse backup file")?;
        RegistryHelper::write_user_path(&backup.user_path).context("Failed to restore PATH")?;
        info!("PATH restored successfully");
        Ok(RestoreResult {
            restored_from: backup_file.to_path_buf(),
        })
    }
    pub fn list(&self) -> Result<Vec<PathBuf>> {
        let mut backups = Vec::new();
        if !self.backup_dir.exists() {
            return Ok(backups);
        }
        for entry in fs::read_dir(&self.backup_dir)? {
            let path = entry?.path();
            if self.is_valid_backup_file(&path) {
                backups.push(path);
            }
        }
        backups.sort();
        backups.reverse();
        Ok(backups)
    }
    fn is_valid_backup_file(&self, path: &Path) -> bool {
        let has_json_ext = path.extension().and_then(|s| s.to_str()) == Some(BACKUP_FILE_EXTENSION);
        let has_prefix = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with(BACKUP_FILE_PREFIX))
            .unwrap_or(false);
        has_json_ext && has_prefix
    }
    fn cleanup_old(&self) -> Result<Vec<PathBuf>> {
        let mut backups = self.list()?;
        let mut cleaned = Vec::new();
        while backups.len() > MAX_BACKUPS {
            if let Some(oldest) = backups.pop() {
                debug!("Removing old backup: {}", oldest.display());
                fs::remove_file(&oldest)
                    .with_context(|| format!("Failed to remove: {}", oldest.display()))?;
                info!("Removed old backup: {}", oldest.display());
                cleaned.push(oldest);
            }
        }
        Ok(cleaned)
    }
    fn validate_path(&self, backup_file: &Path) -> Result<()> {
        let canonical_dir = self
            .backup_dir
            .canonicalize()
            .context("Failed to resolve backup directory")?;
        let canonical_file = backup_file
            .canonicalize()
            .context("Backup file does not exist")?;
        if !canonical_file.starts_with(&canonical_dir) {
            bail!(
                "Security error: Backup must be in {}\nUse 'spath list-backups' to see available backups.",
                self.backup_dir.display()
            );
        }
        if backup_file.extension().and_then(|s| s.to_str()) != Some(BACKUP_FILE_EXTENSION) {
            bail!(
                "Security error: Backup file must have .{} extension",
                BACKUP_FILE_EXTENSION
            );
        }
        if let Some(name) = backup_file.file_name().and_then(|s| s.to_str()) {
            if !name.starts_with(BACKUP_FILE_PREFIX) {
                bail!("Security error: Invalid backup file name format");
            }
        }
        Ok(())
    }
}
