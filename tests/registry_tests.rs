use spath_cli::constants::WINDOWS_PATH;
use std::path::PathBuf;

mod parse_tests {
    use super::*;

    #[test]
    fn test_parse_path_string() {
        let path = format!("{};C:\\System32;;C:\\Tools", WINDOWS_PATH);
        let parsed: Vec<String> = path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], WINDOWS_PATH);
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
        let parsed: Vec<String> = WINDOWS_PATH
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0], WINDOWS_PATH);
    }

    #[test]
    fn test_parse_path_with_trailing_semicolon() {
        let path = format!("{};C:\\System32;", WINDOWS_PATH);
        let parsed: Vec<String> = path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_parse_path_with_leading_semicolon() {
        let path = format!(";{};C:\\System32", WINDOWS_PATH);
        let parsed: Vec<String> = path
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parsed.len(), 2);
    }
}
mod join_tests {
    use super::*;

    #[test]
    fn test_join_paths() {
        let paths = [WINDOWS_PATH.to_string(), "C:\\System32".to_string()];
        let joined = paths.join(";");
        assert_eq!(joined, format!("{};C:\\System32", WINDOWS_PATH));
    }

    #[test]
    fn test_join_empty_paths() {
        let paths: [String; 0] = [];
        let joined = paths.join(";");
        assert_eq!(joined, "");
    }

    #[test]
    fn test_join_single_path() {
        let paths = [WINDOWS_PATH.to_string()];
        let joined = paths.join(";");
        assert_eq!(joined, WINDOWS_PATH);
    }

    #[test]
    fn test_join_many_paths() {
        let paths = [
            WINDOWS_PATH.to_string(),
            "C:\\System32".to_string(),
            "C:\\Tools".to_string(),
            "C:\\Bin".to_string(),
        ];
        let joined = paths.join(";");
        assert_eq!(
            joined,
            format!("{};C:\\System32;C:\\Tools;C:\\Bin", WINDOWS_PATH)
        );
    }
}
mod validation_tests {
    use super::*;
    use spath_cli::constants::MAX_PATH_LENGTH;

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
        let path = format!("{};C:\\System32", WINDOWS_PATH);
        assert!(validate_path_length(&path).is_ok());
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
        let _ = fs::remove_file(&lock_file);
    }

    #[test]
    fn test_lock_dir_creation() {
        let lock_dir = get_lock_dir().unwrap();
        fs::create_dir_all(&lock_dir).unwrap();
        assert!(lock_dir.exists());
    }
}
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
        assert!(file.lock_exclusive().is_ok());
        assert!(file.unlock().is_ok());
        let _ = fs::remove_file(&lock_path);
    }

    #[test]
    fn test_lock_reacquire_after_release() {
        let lock_dir = get_test_lock_dir();
        fs::create_dir_all(&lock_dir).unwrap();
        let lock_path = lock_dir.join("reacquire_test.lock");
        {
            let file = File::create(&lock_path).unwrap();
            file.lock_exclusive().unwrap();
        }
        {
            let file = File::create(&lock_path).unwrap();
            assert!(file.lock_exclusive().is_ok());
        }
        let _ = fs::remove_file(&lock_path);
    }

    #[test]
    fn test_try_lock_nonblocking() {
        let lock_dir = get_test_lock_dir();
        fs::create_dir_all(&lock_dir).unwrap();
        let lock_path = lock_dir.join("trylock_test.lock");
        let file = File::create(&lock_path).unwrap();
        assert!(file.try_lock_exclusive().is_ok());
        file.unlock().unwrap();
        let _ = fs::remove_file(&lock_path);
    }
}
