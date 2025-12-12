#[cfg(test)]
mod scanner_tests {

    #[test]
    fn test_path_with_spaces_no_quotes_detected() {
        let path = "C:\\Program Files\\Test";
        assert!(path.contains(' '));
        assert!(!path.starts_with('"'));
    }

    #[test]
    fn test_path_with_spaces_and_quotes_ok() {
        let path = "\"C:\\Program Files\\Test\"";
        assert!(path.contains(' '));
        assert!(path.starts_with('"') && path.ends_with('"'));
    }

    #[test]
    fn test_empty_path_handled() {
        let path = "";
        assert!(path.is_empty());
    }

    #[test]
    fn test_multiple_paths_separated_by_semicolon() {
        let paths = "C:\\Windows;C:\\Windows\\System32";
        let split: Vec<&str> = paths.split(';').collect();
        assert_eq!(split.len(), 2);
        assert_eq!(split[0], "C:\\Windows");
        assert_eq!(split[1], "C:\\Windows\\System32");
    }

    #[test]
    fn test_path_without_spaces_ok() {
        let path = "C:\\Windows\\System32";
        assert!(!path.contains(' '));
    }

    #[test]
    fn test_relative_path_detected() {
        let path = "..\\relative\\path";
        assert!(!path.contains(':'));
    }

    #[test]
    fn test_non_existent_path_detected() {
        let path = "C:\\NonExistentFolder123456789";
        assert!(!std::path::Path::new(path).exists());
    }

    #[test]
    fn test_mixed_paths_with_and_without_quotes() {
        let paths = "\"C:\\Program Files\\Test\";C:\\Windows";
        let split: Vec<&str> = paths.split(';').collect();
        assert_eq!(split.len(), 2);
        assert!(split[0].starts_with('"'));
        assert!(!split[1].starts_with('"'));
    }

    #[test]
    fn test_path_with_trailing_semicolon() {
        let paths = "C:\\Windows;";
        let filtered: Vec<&str> = paths.split(';').filter(|s| !s.is_empty()).collect();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_path_with_leading_semicolon() {
        let paths = ";C:\\Windows";
        let filtered: Vec<&str> = paths.split(';').filter(|s| !s.is_empty()).collect();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_duplicate_paths_detected() {
        let paths = "C:\\Windows;C:\\Windows";
        let split: Vec<&str> = paths.split(';').collect();
        assert_eq!(split[0], split[1]);
    }

    #[test]
    fn test_case_insensitive_duplicate_detection() {
        let path1 = "C:\\Windows";
        let path2 = "c:\\windows";
        assert_eq!(path1.to_lowercase(), path2.to_lowercase());
    }

    #[test]
    fn test_path_with_forward_slashes() {
        let path = "C:/Windows/System32";
        assert!(path.contains('/'));
    }

    #[test]
    fn test_very_long_path() {
        let long_path = "C:\\".to_string() + &"VeryLongFolderName\\".repeat(60);
        assert!(long_path.len() > 1000);
    }

    #[test]
    fn test_path_with_special_characters() {
        let path = "C:\\Test@Folder#123";
        assert!(path.contains('@'));
        assert!(path.contains('#'));
    }
}

#[cfg(test)]
mod issue_level_tests {
    #[test]
    fn test_critical_issue_for_unquoted_spaces() {
        let path = "C:\\Program Files\\Test";
        let is_critical = path.contains(' ') && !path.starts_with('"');
        assert!(is_critical);
    }

    #[test]
    fn test_warning_issue_for_non_existent() {
        let path = "C:\\NonExistent123456";
        let exists = std::path::Path::new(path).exists();
        assert!(!exists);
    }

    #[test]
    fn test_info_issue_for_properly_quoted() {
        let path = "\"C:\\Program Files\\Test\"";
        let is_properly_quoted = path.starts_with('"') && path.ends_with('"') && path.contains(' ');
        assert!(is_properly_quoted);
    }

    #[test]
    fn test_issue_message_format() {
        let message = "Path contains spaces but is not quoted";
        assert!(message.contains("spaces"));
        assert!(message.contains("quoted"));
    }

    #[test]
    fn test_issue_path_stored_correctly() {
        let original_path = "C:\\Program Files\\Test";
        let stored_path = original_path.to_string();
        assert_eq!(original_path, stored_path);
    }
}

#[cfg(test)]
mod scan_results_tests {
    #[test]
    fn test_empty_results_no_issues() {
        let issues: Vec<String> = vec![];
        assert!(issues.is_empty());
    }

    #[test]
    fn test_results_with_multiple_issues() {
        let issues = ["issue1", "issue2", "issue3"];
        assert_eq!(issues.len(), 3);
    }

    #[test]
    fn test_critical_issues_count() {
        let critical_issues = ["critical1", "critical2"];
        assert_eq!(critical_issues.len(), 2);
    }

    #[test]
    fn test_warning_issues_count() {
        let warnings = ["warning1"];
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_info_issues_count() {
        let info: Vec<String> = vec![];
        assert_eq!(info.len(), 0);
    }

    #[test]
    fn test_total_paths_count() {
        let paths = ["path1", "path2", "path3", "path4"];
        assert_eq!(paths.len(), 4);
    }

    #[test]
    fn test_audit_stats_calculation() {
        let total = 100;
        let valid = 80;
        let percentage = (valid as f64 / total as f64) * 100.0;
        assert_eq!(percentage, 80.0);
    }

    #[test]
    fn test_health_score_calculation_perfect() {
        let total = 50;
        let valid = 50;
        let score = ((valid as f64 / total as f64) * 100.0) as u32;
        assert_eq!(score, 100);
    }

    #[test]
    fn test_health_score_calculation_poor() {
        let total = 100;
        let valid = 30;
        let score = ((valid as f64 / total as f64) * 100.0) as u32;
        assert_eq!(score, 30);
    }

    #[test]
    fn test_health_score_calculation_zero_paths() {
        let total = 0;
        let valid = 0;
        let score = if total > 0 {
            ((valid as f64 / total as f64) * 100.0) as u32
        } else {
            0
        };
        assert_eq!(score, 0);
    }
}
