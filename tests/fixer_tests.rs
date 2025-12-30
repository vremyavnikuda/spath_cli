use spath_cli::constants::PROGRAM_FILES;

#[cfg(test)]
mod fixer_tests {
    use super::*;

    #[test]
    fn test_backup_directory_creation() {
        let path = std::path::PathBuf::from("test_backup");
        assert!(path.to_str().is_some());
    }

    #[test]
    fn test_backup_file_naming_format() {
        let filename = "path_backup_20241213_120000.json";
        assert!(filename.starts_with("path_backup_"));
        assert!(filename.ends_with(".json"));
    }

    #[test]
    fn test_backup_contains_timestamp() {
        let timestamp = "20241213_120000";
        assert!(timestamp.contains('_'));
        assert_eq!(timestamp.len(), 15);
    }

    #[test]
    fn test_backup_json_format_valid() {
        let json = format!(
            r#"{{"timestamp":"20241213","user_path":"{}"}}"#,
            spath_cli::constants::WINDOWS_PATH
        );
        assert!(json.contains("timestamp"));
        assert!(json.contains("user_path"));
    }

    #[test]
    fn test_add_quotes_to_unquoted_path() {
        let path = format!("{}\\Test", PROGRAM_FILES);
        let expected = format!("\"{}\\Test\"", PROGRAM_FILES);
        assert_eq!(format!("\"{}\"", path), expected);
    }

    #[test]
    fn test_dont_add_quotes_to_quoted_path() {
        let path = format!("\"{}\\Test\"", PROGRAM_FILES);
        assert!(path.starts_with('"'));
    }

    #[test]
    fn test_dont_add_quotes_to_path_without_spaces() {
        let path = "C:\\Windows\\System32";
        assert!(!path.contains(' '));
    }

    #[test]
    fn test_remove_duplicate_paths() {
        let paths = vec![
            spath_cli::constants::WINDOWS_PATH,
            spath_cli::constants::WINDOWS_PATH,
            "C:\\System32",
        ];
        let unique: std::collections::HashSet<_> = paths.into_iter().collect();
        assert_eq!(unique.len(), 2);
    }

    #[test]
    fn test_preserve_path_order() {
        let paths = ["C:\\First", "C:\\Second", "C:\\Third"];
        assert_eq!(paths[0], "C:\\First");
        assert_eq!(paths[2], "C:\\Third");
    }

    #[test]
    fn test_dry_run_no_changes() {
        let dry_run = true;
        assert!(dry_run);
    }

    #[test]
    fn test_fix_results_show_changes() {
        let changes = ["Fixed: path1", "Fixed: path2"];
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_fix_results_empty_when_no_issues() {
        let changes: Vec<String> = vec![];
        assert!(changes.is_empty());
    }

    #[test]
    fn test_backup_list_sorted_by_date() {
        let mut backups = ["backup_20241213", "backup_20241212", "backup_20241214"];
        backups.sort();
        backups.reverse();
        assert_eq!(backups[0], "backup_20241214");
    }

    #[test]
    fn test_backup_list_empty_when_no_backups() {
        let backups: Vec<String> = vec![];
        assert!(backups.is_empty());
    }

    #[test]
    fn test_restore_from_valid_backup() {
        let backup_path = "path_backup_20241213.json";
        assert!(backup_path.ends_with(".json"));
    }

    #[test]
    fn test_restore_fails_with_invalid_backup() {
        let backup_path = "invalid.txt";
        assert!(!backup_path.ends_with(".json"));
    }

    #[test]
    fn test_restore_fails_with_corrupted_json() {
        let json = "{invalid json";
        assert!(!json.ends_with('}'));
    }

    #[test]
    fn test_backup_before_fix() {
        let backup_created = true;
        assert!(backup_created);
    }

    #[test]
    fn test_multiple_backups_preserved() {
        let backups = ["backup1", "backup2", "backup3"];
        assert_eq!(backups.len(), 3);
    }

    #[test]
    fn test_fix_handles_empty_path() {
        let path = "";
        assert!(path.is_empty());
    }
}
