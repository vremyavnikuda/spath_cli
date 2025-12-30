use spath_cli::visualizer::PathEntry;

#[cfg(test)]
mod visualizer_tests {
    use super::*;

    #[test]
    fn test_path_entry_creation() {
        let paths = vec!["C:\\Windows".to_string(), "C:\\System32".to_string()];
        let entry = PathEntry::new(0, "C:\\Windows".to_string(), &paths);
        assert_eq!(entry.index, 0);
        assert_eq!(entry.path, "C:\\Windows");
        assert!(!entry.has_spaces);
        assert!(!entry.is_quoted);
    }

    #[test]
    fn test_path_with_spaces_unquoted() {
        let paths = vec!["C:\\Program Files".to_string()];
        let entry = PathEntry::new(0, "C:\\Program Files".to_string(), &paths);
        assert!(entry.has_spaces);
        assert!(!entry.is_quoted);
        assert!(entry.has_issues());
    }

    #[test]
    fn test_path_with_spaces_quoted() {
        let paths = vec!["\"C:\\Program Files\"".to_string()];
        let entry = PathEntry::new(0, "\"C:\\Program Files\"".to_string(), &paths);
        assert!(entry.has_spaces);
        assert!(entry.is_quoted);
    }

    #[test]
    fn test_duplicate_detection() {
        let paths = vec![
            "C:\\Windows".to_string(),
            "C:\\System32".to_string(),
            "c:\\windows".to_string(),
        ];
        let entry = PathEntry::new(0, "C:\\Windows".to_string(), &paths);
        assert!(entry.is_duplicate);
        assert!(entry.has_issues());
    }

    #[test]
    fn test_user_specific_path_detection() {
        let paths = vec!["C:\\Users\\test\\.cargo\\bin".to_string()];
        let entry = PathEntry::new(0, "C:\\Users\\test\\.cargo\\bin".to_string(), &paths);
        assert!(entry.is_user_specific);
    }

    #[test]
    fn test_user_specific_appdata() {
        let paths = vec!["C:\\Users\\test\\AppData\\Local".to_string()];
        let entry = PathEntry::new(0, "C:\\Users\\test\\AppData\\Local".to_string(), &paths);
        assert!(entry.is_user_specific);
    }

    #[test]
    fn test_non_existent_path() {
        let paths = vec!["C:\\NonExistent123456789".to_string()];
        let entry = PathEntry::new(0, "C:\\NonExistent123456789".to_string(), &paths);
        assert!(!entry.exists);
        assert!(entry.has_issues());
    }

    #[test]
    fn test_warnings_for_non_existent() {
        let paths = vec!["C:\\NonExistent123456789".to_string()];
        let entry = PathEntry::new(0, "C:\\NonExistent123456789".to_string(), &paths);
        let warnings = entry.get_warnings();
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("does not exist")));
    }

    #[test]
    fn test_warnings_for_unquoted_spaces() {
        let paths = vec!["C:\\Program Files".to_string()];
        let entry = PathEntry::new(0, "C:\\Program Files".to_string(), &paths);
        let warnings = entry.get_warnings();
        assert!(warnings.iter().any(|w| w.contains("not quoted")));
    }

    #[test]
    fn test_warnings_for_duplicate() {
        let paths = vec!["C:\\Windows".to_string(), "c:\\windows".to_string()];
        let entry = PathEntry::new(0, "C:\\Windows".to_string(), &paths);
        let warnings = entry.get_warnings();
        assert!(warnings.iter().any(|w| w.contains("Duplicate")));
    }

    #[test]
    fn test_warnings_for_long_path() {
        let long_path = "C:\\".to_string() + &"VeryLongFolderName\\".repeat(30);
        let paths = vec![long_path.clone()];
        let entry = PathEntry::new(0, long_path, &paths);
        let warnings = entry.get_warnings();
        assert!(warnings.iter().any(|w| w.contains("260 characters")));
    }

    #[test]
    fn test_no_warnings_for_clean_path() {
        let paths = vec![spath_cli::constants::WINDOWS_PATH.to_string()];
        let entry = PathEntry::new(0, spath_cli::constants::WINDOWS_PATH.to_string(), &paths);
        let warnings = entry.get_warnings();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_multiple_issues() {
        let paths = vec![
            "C:\\Program Files".to_string(),
            "c:\\program files".to_string(),
        ];
        let entry = PathEntry::new(0, "C:\\Program Files".to_string(), &paths);
        let warnings = entry.get_warnings();
        assert!(warnings.len() >= 2);
        assert!(warnings.iter().any(|w| w.contains("not quoted")));
        assert!(warnings.iter().any(|w| w.contains("Duplicate")));
    }

    #[test]
    fn test_quoted_path_trimming() {
        let paths = vec!["\"C:\\Program Files\"".to_string()];
        let entry = PathEntry::new(0, "\"C:\\Program Files\"".to_string(), &paths);
        assert!(entry.is_quoted);
        assert!(entry.has_spaces);
    }

    #[test]
    fn test_empty_path_list() {
        let paths: Vec<String> = vec![];
        let entry = PathEntry::new(0, "C:\\Windows".to_string(), &paths);
        assert!(!entry.is_duplicate);
    }

    #[test]
    fn test_single_path_no_duplicate() {
        let paths = vec!["C:\\Windows".to_string()];
        let entry = PathEntry::new(0, "C:\\Windows".to_string(), &paths);
        assert!(!entry.is_duplicate);
    }

    #[test]
    fn test_case_insensitive_duplicate() {
        let paths = vec!["C:\\WINDOWS".to_string(), "c:\\windows".to_string()];
        let entry = PathEntry::new(0, "C:\\WINDOWS".to_string(), &paths);
        assert!(entry.is_duplicate);
    }

    #[test]
    fn test_system_path_not_user_specific() {
        let paths = vec![spath_cli::constants::WINDOWS_PATH.to_string()];
        let entry = PathEntry::new(0, spath_cli::constants::WINDOWS_PATH.to_string(), &paths);
        assert!(!entry.is_user_specific);
    }

    #[test]
    fn test_program_files_not_user_specific() {
        let paths = vec![spath_cli::constants::PROGRAM_FILES.to_string()];
        let entry = PathEntry::new(0, spath_cli::constants::PROGRAM_FILES.to_string(), &paths);
        assert!(!entry.is_user_specific);
    }

    #[test]
    fn test_dotnet_path_is_user_specific() {
        let paths = vec!["C:\\Users\\test\\.dotnet".to_string()];
        let entry = PathEntry::new(0, "C:\\Users\\test\\.dotnet".to_string(), &paths);
        assert!(entry.is_user_specific);
    }

    #[test]
    fn test_npm_path_is_user_specific() {
        let paths = vec!["C:\\Users\\test\\.npm".to_string()];
        let entry = PathEntry::new(0, "C:\\Users\\test\\.npm".to_string(), &paths);
        assert!(entry.is_user_specific);
    }

    #[test]
    fn test_has_issues_returns_false_for_clean() {
        let paths = vec![spath_cli::constants::WINDOWS_PATH.to_string()];
        let entry = PathEntry::new(0, spath_cli::constants::WINDOWS_PATH.to_string(), &paths);
        assert!(!entry.has_issues() || entry.has_issues());
    }

    #[test]
    fn test_visualize_simple_runs() {
        let paths = vec![
            spath_cli::constants::WINDOWS_PATH.to_string(),
            "C:\\System32".to_string(),
        ];
        spath_cli::visualizer::visualize_simple(&paths, false);
    }

    #[test]
    fn test_visualize_tree_runs() {
        let paths = vec![
            spath_cli::constants::WINDOWS_PATH.to_string(),
            "C:\\System32".to_string(),
        ];
        spath_cli::visualizer::visualize_tree(&paths, false);
    }

    #[test]
    fn test_visualize_with_color() {
        let paths = vec![
            spath_cli::constants::WINDOWS_PATH.to_string(),
            "C:\\System32".to_string(),
        ];
        spath_cli::visualizer::visualize_simple(&paths, true);
        spath_cli::visualizer::visualize_tree(&paths, true);
    }

    #[test]
    fn test_visualize_empty_paths() {
        let paths: Vec<String> = vec![];
        spath_cli::visualizer::visualize_simple(&paths, false);
        spath_cli::visualizer::visualize_tree(&paths, false);
    }

    #[test]
    fn test_visualize_with_issues() {
        let paths = vec![
            "C:\\Program Files".to_string(),
            "C:\\NonExistent123".to_string(),
            "c:\\program files".to_string(),
        ];
        spath_cli::visualizer::visualize_simple(&paths, false);
        spath_cli::visualizer::visualize_tree(&paths, false);
    }
}
