// Constants for spath-cli

/// Windows system directory
pub const WINDOWS_PATH: &str = "c:\\windows";

/// Program Files directory (64-bit)
pub const PROGRAM_FILES: &str = "c:\\program files";

/// Program Files directory (32-bit)
pub const PROGRAM_FILES_X86: &str = "c:\\program files (x86)";

/// ProgramData directory (shared application data)
pub const PROGRAM_DATA: &str = "c:\\programdata";

/// Common user-specific path patterns
pub const USER_PATHS: &[&str] = &[".cargo", ".dotnet", ".npm", ".bun", ".local", "\\appdata\\"];

/// Backup directory name
pub const BACKUP_DIR_NAME: &str = "backups";

/// Maximum number of backup files to keep
pub const MAX_BACKUPS: usize = 10;

/// Maximum PATH environment variable length (Windows limitation)
pub const MAX_PATH_LENGTH: usize = 2047;

/// Registry key paths
pub const SYSTEM_ENV_KEY: &str = "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
pub const USER_ENV_KEY: &str = "Environment";

/// Lock file names for preventing race conditions
pub const USER_PATH_LOCK: &str = "user_path.lock";
pub const SYSTEM_PATH_LOCK: &str = "system_path.lock";

/// Backup file prefix
pub const BACKUP_FILE_PREFIX: &str = "path_backup_";

/// Backup file extension
pub const BACKUP_FILE_EXTENSION: &str = "json";

/// Backup timestamp format
pub const BACKUP_TIMESTAMP_FORMAT: &str = "%Y%m%d_%H%M%S";

/// Maximum single path length (Windows MAX_PATH limitation)
pub const MAX_SINGLE_PATH_LENGTH: usize = 260;
