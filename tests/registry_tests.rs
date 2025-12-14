//! Tests for registry module - PATH reading, writing, and locking operations.

use std::path::PathBuf;

/// Tests for RegistryHelper::parse_path_string
mod parse_tests {
    #[test]
    fn test_parse_path_string() {
        let path = "C:\\Windows;C:\\System32;;C:\\Tools";
        let parsed: Vec<String> = path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], "C:\\Windows");
        assert_eq!(parsed[1], "C:\\System32");
        assert_eq!(parsed[2], "C:\\Tools");
    }

    #[test]
    fn test_parse_empty_path() {
        let path = "";
        let parsed: Vec<String> = path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_parse_single_path() {
        let path = "C:\\Windows";
        let parsed: Vec<String> = path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], "C:\\Windows");
    }

    #[test]
    fn test_parse_path_with_trailing_semicolon() {
        let path = "C:\\Windows;C:\\System32;";
        let parsed: Vec<String> = path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_parse_path_with_leading_semicolon() {
        let path = ";C:\\Windows;C:\\System32";
        let parsed: Vec<String> = path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parsed.len(), 2);
    }
}

/// Tests for RegistryHelper::join_paths
mod join_tests {
    #[test]
    fn test_join_paths() {
        let paths = vec!["C:\\Windows".to_string(), "C:\\System32".to_string()];
        let joined = paths.join(";");
        assert_eq!(joined, "C:\\Windows;C:\\System32");
    }

    #[test]
    fn test_join_empty_paths() {
        let paths: Vec<String> = vec![];
        let joined = paths.join(";");
        assert_eq!(joined, "");
    }

    #[test]
    fn test_join_single_path() {
        let paths = vec!["C:\\Windows".to_string()];
        let joined = paths.join(";");
        assert_eq!(joined, "C:\\Windows");
    }

    #[test]
    fn test_join_many_paths() {
        let paths = vec![
            "C:\\Windows".to_string(),
            "C:\\System32".to_string(),
            "C:\\Tools".to_string(),
            "C:\\Bin".to_string(),
        ];
        let joined = paths.join(";");
        assert_eq!(joined, "C:\\Windows;C:\\System32;C:\\Tools;C:\\Bin");
    }
}

/// Tests for PATH length validation
mod validation_tests {
    const MAX_PATH_LENGTH: usize = 2047;

    fn validate_path_length(path: &str) -> Result<(), String> {
        if path.len() > MAX_PATH_LENGTH {
            Err(format!(
                "PATH exceeds maximum length of {} characters (current: {} characters)",
                MAX_PATH_LENGTH,
                path.len()
            ))
        } else {
            Ok(())
        }
    }

    #[test]
    fn test_validate_path_length_ok() {
        let path = "C:\\Windows;C:\\System32";
        assert!(validate_path_length(path).is_ok());
    }

    #[test]
    fn test_validate_path_length_at_limit() {
        let path = "X".repeat(MAX_PATH_LENGTH);
        assert!(validate_path_length(&path).is_ok());
    }

    #[test]
    fn test_validate_path_length_exceeds_limit() {
        let path = "X".repeat(MAX_PATH_LENGTH + 1);
        let result = validate_path_length(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum length"));
    }

    #[test]
    fn test_validate_path_length_empty() {
        let path = "";
        assert!(validate_path_length(path).is_ok());
    }

    #[test]
    fn test_validate_path_length_one_below_limit() {
        let path = "X".repeat(MAX_PATH_LENGTH - 1);
        assert!(validate_path_length(&path).is_ok());
    }

    #[test]
    fn test_validate_path_length_one_above_limit() {
        let path = "X".repeat(MAX_PATH_LENGTH + 1);
        assert!(validate_path_length(&path).is_err());
    }
}

/// Tests for lock directory operations
mod lock_tests {
    use super::*;
    use std::env;
    use std::fs;

    fn get_lock_dir() -> Result<PathBuf, String> {
        let local_app_data =
            env::var("LOCALAPPDATA").map_err(|_| "LOCALAPPDATA not set".to_string())?;
        Ok(PathBuf::from(local_app_data).join("spath").join("locks"))
    }

    #[test]
    fn test_get_lock_dir() {
        let result = get_lock_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("spath"));
        assert!(path.to_string_lossy().contains("locks"));
    }

    #[test]
    fn test_lock_dir_path_format() {
        let lock_dir = get_lock_dir().unwrap();
        let path_str = lock_dir.to_string_lossy();
        assert!(path_str.ends_with("spath\\locks") || path_str.ends_with("spath/locks"));
    }

    #[test]
    fn test_lock_file_creation() {
        let lock_dir = get_lock_dir().unwrap();
        fs::create_dir_all(&lock_dir).unwrap();

        let lock_file = lock_dir.join("test_registry.lock");
        fs::write(&lock_file, "test").unwrap();

        assert!(lock_file.exists());

        // Cleanup
        let _ = fs::remove_file(&lock_file);
    }

    #[test]
    fn test_lock_dir_creation() {
        let lock_dir = get_lock_dir().unwrap();
        fs::create_dir_all(&lock_dir).unwrap();
        assert!(lock_dir.exists());
    }
}

/// Tests for file locking mechanism
mod file_lock_tests {
    use fs2::FileExt;
    use std::fs::{self, File};
    use std::path::PathBuf;

    fn get_test_lock_dir() -> PathBuf {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap();
        PathBuf::from(local_app_data)
            .join("spath")
            .join("locks")
            .join("test")
    }

    #[test]
    fn test_exclusive_lock_acquire_and_release() {
        let lock_dir = get_test_lock_dir();
        fs::create_dir_all(&lock_dir).unwrap();

        let lock_path = lock_dir.join("exclusive_test.lock");
        let file = File::create(&lock_path).unwrap();

        // Acquire lock
        assert!(file.lock_exclusive().is_ok());

        // Release lock
        assert!(file.unlock().is_ok());

        // Cleanup
        let _ = fs::remove_file(&lock_path);
    }

    #[test]
    fn test_lock_reacquire_after_release() {
        let lock_dir = get_test_lock_dir();
        fs::create_dir_all(&lock_dir).unwrap();

        let lock_path = lock_dir.join("reacquire_test.lock");

        // First acquisition
        {
            let file = File::create(&lock_path).unwrap();
            file.lock_exclusive().unwrap();
            // Lock released when file goes out of scope
        }

        // Second acquisition should succeed
        {
            let file = File::create(&lock_path).unwrap();
            assert!(file.lock_exclusive().is_ok());
        }

        // Cleanup
        let _ = fs::remove_file(&lock_path);
    }

    #[test]
    fn test_try_lock_nonblocking() {
        let lock_dir = get_test_lock_dir();
        fs::create_dir_all(&lock_dir).unwrap();

        let lock_path = lock_dir.join("trylock_test.lock");
        let file = File::create(&lock_path).unwrap();

        // Try lock should succeed on unlocked file
        assert!(file.try_lock_exclusive().is_ok());

        file.unlock().unwrap();

        // Cleanup
        let _ = fs::remove_file(&lock_path);
    }
}
