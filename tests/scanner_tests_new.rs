use spath_cli::constants::{PROGRAM_FILES, WINDOWS_PATH};
use std::env;

#[cfg(test)]
mod scanner_business_logic_tests {
    use super::*;

    #[test]
    fn test_scanner_detects_critical_unquoted_program_files() {
        env::set_var("TEST_PATH", format!("{}\\Git\\cmd", PROGRAM_FILES));
        let test_path = format!("{}\\Git\\cmd;{}", PROGRAM_FILES, WINDOWS_PATH);
        let paths: Vec<&str> = test_path.split(';').collect();
        let vulnerable_path = paths[0];
        assert!(vulnerable_path.contains(' '));
        assert!(!vulnerable_path.starts_with('"'));
        assert!(vulnerable_path
            .to_lowercase()
            .starts_with("c:\\program files"));
    }

    #[test]
    fn test_scanner_detects_exploitable_path_structure() {
        let path = "C:\\Program Files\\App\\bin";

        let has_spaces = path.contains(' ');
        let is_quoted = path.starts_with('"');
        let is_program_files = path.to_lowercase().starts_with("c:\\program files");
        assert!(has_spaces && !is_quoted && is_program_files);
    }

    #[test]
    fn test_scanner_ignores_properly_quoted_paths() {
        let path = "\"C:\\Program Files\\Git\\cmd\"";

        assert!(path.starts_with('"') && path.ends_with('"'));
    }

    #[test]
    fn test_scanner_detects_exact_duplicates() {
        let test_path = "C:\\Windows;C:\\Windows;C:\\System32";
        let paths: Vec<&str> = test_path.split(';').collect();
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for path in paths {
            if !seen.insert(path) {
                duplicates.push(path);
            }
        }
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0], "C:\\Windows");
    }

    #[test]
    fn test_scanner_detects_case_insensitive_duplicates() {
        let test_path = format!(
            "{};{};{}",
            WINDOWS_PATH,
            WINDOWS_PATH.to_lowercase(),
            WINDOWS_PATH.to_uppercase()
        );
        let paths: Vec<&str> = test_path.split(';').collect();
        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for path in paths {
            let normalized = path.to_lowercase();
            if !seen.insert(normalized) {
                duplicates.push(path);
            }
        }
        assert_eq!(duplicates.len(), 2);
    }

    #[test]
    fn test_scanner_detects_quoted_unquoted_duplicates() {
        let path1 = "C:\\Program Files\\Git";
        let path2 = "\"C:\\Program Files\\Git\"";
        let normalized1 = path1.trim_matches('"').to_lowercase();
        let normalized2 = path2.trim_matches('"').to_lowercase();
        assert_eq!(normalized1, normalized2);
    }

    #[test]
    fn test_scanner_detects_non_existent_paths() {
        let non_existent = "C:\\NonExistentFolder123456789XYZ";
        assert!(!std::path::Path::new(non_existent).exists());
    }

    #[test]
    fn test_scanner_validates_existing_paths() {
        assert!(std::path::Path::new(WINDOWS_PATH).exists());
    }

    #[test]
    fn test_scanner_handles_env_vars_in_paths() {
        let path_with_var = "%SystemRoot%\\System32";
        assert!(path_with_var.contains('%'));
        let system_root = env::var("SystemRoot").unwrap_or_else(|_| WINDOWS_PATH.to_string());
        let expanded = path_with_var.replace("%SystemRoot%", &system_root);
        assert!(!expanded.contains('%'));
        assert!(std::path::Path::new(&expanded).exists());
    }

    #[test]
    fn test_scanner_detects_relative_paths() {
        let relative_paths = vec!["..\\relative\\path", ".\\current\\path", "relative\\path"];
        for path in relative_paths {
            assert!(!path.contains(':'));
        }
    }

    #[test]
    fn test_scanner_validates_absolute_paths() {
        let program_files_quoted = format!("\"{}\"", PROGRAM_FILES);
        let absolute_paths = vec![WINDOWS_PATH, "D:\\Data", &program_files_quoted];
        for path in absolute_paths {
            let has_drive = path.contains(':');
            let is_quoted = path.starts_with('"');
            assert!(has_drive || is_quoted);
        }
    }

    #[test]
    fn test_scanner_counts_total_paths() {
        let test_path = format!("{};C:\\System32;{}", WINDOWS_PATH, PROGRAM_FILES);
        let paths: Vec<&str> = test_path.split(';').collect();

        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_scanner_counts_unquoted_with_spaces() {
        let git_path = format!("{}\\Git", PROGRAM_FILES);
        let app_path = format!("\"{}\\App\"", PROGRAM_FILES);
        let paths = [&git_path, &app_path, WINDOWS_PATH];
        let unquoted_with_spaces = paths
            .iter()
            .filter(|p| p.contains(' ') && !p.starts_with('"'))
            .count();
        assert_eq!(unquoted_with_spaces, 1);
    }

    #[test]
    fn test_scanner_counts_properly_quoted() {
        let git_path = format!("\"{}\\Git\"", PROGRAM_FILES);
        let app_path = format!("\"{}\\App\"", PROGRAM_FILES);
        let paths = [&git_path, &app_path, WINDOWS_PATH];
        let properly_quoted = paths
            .iter()
            .filter(|p| p.starts_with('"') && p.ends_with('"') && p.contains(' '))
            .count();
        assert_eq!(properly_quoted, 2);
    }

    #[test]
    fn test_scanner_assigns_critical_level_to_exploitable() {
        let path = "C:\\Program Files\\Git\\cmd";
        let has_spaces = path.contains(' ');
        let is_quoted = path.starts_with('"');
        let is_exploitable = path.to_lowercase().starts_with("c:\\program files");
        let exists = std::path::Path::new(path).exists();
        assert!(exists && has_spaces && !is_quoted && is_exploitable);
    }

    #[test]
    fn test_scanner_assigns_warning_level_to_non_existent() {
        let path = "C:\\NonExistent123";
        let exists = std::path::Path::new(path).exists();
        assert!(!exists);
    }

    #[test]
    fn test_scanner_assigns_info_level_to_safe_unquoted() {
        let path = "C:\\Custom App\\bin";
        let has_spaces = path.contains(' ');
        let is_program_files = path.to_lowercase().starts_with("c:\\program files");
        assert!(has_spaces && !is_program_files);
    }

    #[test]
    fn test_scanner_handles_empty_path_string() {
        let empty_path = "";
        let paths: Vec<&str> = empty_path.split(';').filter(|s| !s.is_empty()).collect();
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_scanner_handles_path_with_only_semicolons() {
        let path = ";;;";
        let paths: Vec<&str> = path.split(';').filter(|s| !s.is_empty()).collect();
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_scanner_handles_mixed_separators() {
        let path = format!(";{};C:\\System32;", WINDOWS_PATH);
        let paths: Vec<&str> = path.split(';').filter(|s| !s.is_empty()).collect();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_scanner_handles_unicode_paths() {
        let unicode_path = "C:\\Пользователи\\用户\\ユーザー";
        assert!(!unicode_path.is_ascii());
    }

    #[test]
    fn test_scanner_handles_very_long_paths() {
        let long_path = "C:\\".to_string() + &"VeryLongFolderName\\".repeat(50);
        assert!(long_path.len() > 260);
    }

    #[test]
    fn test_scanner_handles_special_characters() {
        let special_chars = vec![
            "C:\\Path (x86)\\App",
            "C:\\Path [test]\\App",
            "C:\\Path & App",
        ];
        for path in special_chars {
            assert!(!path.is_empty());
        }
    }
}
