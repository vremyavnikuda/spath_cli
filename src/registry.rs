//! Common registry operations for PATH management.
//!
//! This module provides unified access to Windows Registry for reading and writing
//! PATH environment variables, eliminating code duplication across modules.
//!
//! ## Race Condition Protection
//!
//! Write operations use file-based locking via `fs2` crate to prevent concurrent
//! modifications to PATH by multiple spath processes. Lock files are stored in
//! `%LOCALAPPDATA%\spath\locks\`.

use anyhow::{bail, Context, Result};
use fs2::FileExt;
use std::fs::{self, File};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use winreg::enums::*;
use winreg::RegKey;

use crate::constants::{
    MAX_PATH_LENGTH, SYSTEM_ENV_KEY, SYSTEM_PATH_LOCK, USER_ENV_KEY, USER_PATH_LOCK,
};

/// RAII guard for file lock. Automatically releases lock when dropped.
pub struct PathLockGuard {
    _file: File,
}

impl PathLockGuard {
    /// Acquires an exclusive lock on the specified lock file.
    /// Blocks until the lock is acquired.
    fn acquire(lock_name: &str) -> Result<Self> {
        debug!("Attempting to acquire lock: {}", lock_name);
        let lock_dir = get_lock_dir()?;
        fs::create_dir_all(&lock_dir).context("Failed to create lock directory")?;
        let lock_path = lock_dir.join(lock_name);
        let file = File::create(&lock_path).context("Failed to create lock file")?;
        file.lock_exclusive()
            .context("Failed to acquire exclusive lock. Another spath process may be running.")?;
        debug!("Lock acquired: {}", lock_name);
        Ok(Self { _file: file })
    }
}

/// Returns the directory for lock files: `%LOCALAPPDATA%\spath\locks\`
fn get_lock_dir() -> Result<PathBuf> {
    let local_app_data =
        std::env::var("LOCALAPPDATA").context("LOCALAPPDATA environment variable not set")?;
    Ok(PathBuf::from(local_app_data).join("spath").join("locks"))
}

/// Helper for Windows Registry PATH operations.
pub struct RegistryHelper;

impl RegistryHelper {
    /// Reads SYSTEM PATH as raw string.
    /// May fail without administrator rights.
    pub fn read_system_path_raw() -> Result<String> {
        debug!("Reading SYSTEM PATH from registry");
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env_key = hklm.open_subkey(SYSTEM_ENV_KEY).map_err(|e| {
            warn!("Failed to open system environment key: {}", e);
            anyhow::anyhow!("Failed to open system environment key. Try running as administrator.")
        })?;
        let path = env_key.get_value("Path").map_err(|e| {
            error!("Failed to read system PATH: {}", e);
            anyhow::anyhow!("Failed to read system PATH")
        })?;
        info!("Successfully read SYSTEM PATH");
        Ok(path)
    }

    /// Reads SYSTEM PATH as `Vec<String>`.
    ///
    /// May fail without administrator rights.
    pub fn read_system_path() -> Result<Vec<String>> {
        let path = Self::read_system_path_raw()?;
        Ok(Self::parse_path_string(&path))
    }

    /// Reads USER PATH as raw string.
    pub fn read_user_path_raw() -> Result<String> {
        debug!("Reading USER PATH from registry");
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu.open_subkey(USER_ENV_KEY).map_err(|e| {
            error!("Failed to open user environment key: {}", e);
            anyhow::anyhow!("Failed to open user environment key")
        })?;
        let path = env_key.get_value("Path").map_err(|e| {
            error!("Failed to read user PATH: {}", e);
            anyhow::anyhow!("Failed to read user PATH")
        })?;
        info!("Successfully read USER PATH");
        Ok(path)
    }

    /// Reads USER PATH as `Vec<String>`.
    pub fn read_user_path() -> Result<Vec<String>> {
        let path = Self::read_user_path_raw()?;
        Ok(Self::parse_path_string(&path))
    }

    /// Validates that PATH length does not exceed Windows limit.
    ///
    /// # Errors
    /// Returns an error if the path exceeds MAX_PATH_LENGTH (2047 characters).
    pub fn validate_path_length(path: &str) -> Result<()> {
        if path.len() > MAX_PATH_LENGTH {
            error!(
                "PATH exceeds maximum length: {} > {}",
                path.len(),
                MAX_PATH_LENGTH
            );
            bail!(
                "PATH exceeds maximum length of {} characters (current: {} characters). \
                Consider removing unused paths.",
                MAX_PATH_LENGTH,
                path.len()
            );
        }
        debug!("PATH length validated: {} characters", path.len());
        Ok(())
    }

    /// Writes USER PATH to registry with exclusive locking.
    ///
    /// Uses file-based locking to prevent race conditions when multiple
    /// spath processes try to modify PATH simultaneously.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Lock cannot be acquired (another process is modifying PATH)
    /// - PATH exceeds maximum length (2047 characters)
    /// - Registry key cannot be opened for writing
    /// - Value cannot be written to registry
    pub fn write_user_path(path: &str) -> Result<()> {
        debug!("Writing USER PATH to registry");
        let _lock = PathLockGuard::acquire(USER_PATH_LOCK)
            .context("Failed to acquire lock for USER PATH modification")?;
        Self::validate_path_length(path)?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu
            .open_subkey_with_flags(USER_ENV_KEY, KEY_WRITE)
            .map_err(|e| {
                error!("Failed to open user environment key for writing: {}", e);
                anyhow::anyhow!("Failed to open user environment key for writing")
            })?;
        env_key.set_value("Path", &path).map_err(|e| {
            error!("Failed to write user PATH to registry: {}", e);
            anyhow::anyhow!("Failed to write user PATH to registry")
        })?;
        info!("Successfully wrote USER PATH to registry");
        Ok(())
    }

    /// Writes SYSTEM PATH to registry with exclusive locking.
    /// Requires administrator rights.
    ///
    /// Uses file-based locking to prevent race conditions when multiple
    /// spath processes try to modify PATH simultaneously.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Lock cannot be acquired (another process is modifying PATH)
    /// - PATH exceeds maximum length (2047 characters)
    /// - Registry key cannot be opened (requires admin)
    /// - Value cannot be written to registry
    pub fn write_system_path(path: &str) -> Result<()> {
        debug!("Writing SYSTEM PATH to registry");
        let _lock = PathLockGuard::acquire(SYSTEM_PATH_LOCK)
            .context("Failed to acquire lock for SYSTEM PATH modification")?;
        Self::validate_path_length(path)?;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env_key = hklm
            .open_subkey_with_flags(SYSTEM_ENV_KEY, KEY_READ | KEY_WRITE)
            .map_err(|e| {
                warn!("Failed to open system environment key for writing: {}", e);
                anyhow::anyhow!(
                    "Failed to open system environment key for writing (requires admin)"
                )
            })?;
        env_key.set_value("Path", &path).map_err(|e| {
            error!("Failed to write system PATH to registry: {}", e);
            anyhow::anyhow!("Failed to write system PATH to registry")
        })?;
        info!("Successfully wrote SYSTEM PATH to registry");
        Ok(())
    }

    /// Parses PATH string into `Vec<String>`, filtering empty entries.
    pub fn parse_path_string(path: &str) -> Vec<String> {
        path.split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Joins path entries into a single PATH string.
    pub fn join_paths(paths: &[String]) -> String {
        paths.join(";")
    }
}
