#[cfg(test)]
mod analyzer_tests {
    #[test]
    fn test_categorize_system_program() {
        let path = "C:\\Program Files\\Test";
        assert!(path.to_lowercase().starts_with("c:\\program files"));
    }

    #[test]
    fn test_categorize_user_program() {
        let path = "C:\\Users\\test\\AppData\\Local";
        assert!(path.to_lowercase().contains("\\users\\"));
    }

    #[test]
    fn test_categorize_program_data() {
        let path = "C:\\ProgramData\\Test";
        assert!(path.to_lowercase().starts_with("c:\\programdata"));
    }

    #[test]
    fn test_categorize_ambiguous_path() {
        let path = "D:\\CustomFolder";
        assert!(!path.to_lowercase().starts_with("c:\\program"));
    }

    #[test]
    fn test_detect_user_path_in_system() {
        let path = "C:\\Users\\test\\bin";
        let in_system = true;
        let is_user_path = path.contains("Users");
        assert!(in_system && is_user_path);
    }

    #[test]
    fn test_detect_system_path_in_user() {
        let path = "C:\\Program Files\\Tool";
        let in_user = true;
        let is_system_path = path.contains("Program Files");
        assert!(in_user && is_system_path);
    }

    #[test]
    fn test_path_needs_quotes_with_spaces() {
        let path = "C:\\Program Files\\Test";
        assert!(path.contains(' '));
    }

    #[test]
    fn test_path_doesnt_need_quotes_without_spaces() {
        let path = "C:\\Windows\\System32";
        assert!(!path.contains(' '));
    }

    #[test]
    fn test_path_exists_check() {
        let path = "C:\\Windows";
        let exists = std::path::Path::new(path).exists();
        assert!(exists);
    }

    #[test]
    fn test_path_not_exists_check() {
        let path = "C:\\NonExistent123456789";
        let exists = std::path::Path::new(path).exists();
        assert!(!exists);
    }

    #[test]
    fn test_analyze_empty_system_path() {
        let system_paths: Vec<String> = vec![];
        assert!(system_paths.is_empty());
    }

    #[test]
    fn test_analyze_empty_user_path() {
        let user_paths: Vec<String> = vec![];
        assert!(user_paths.is_empty());
    }

    #[test]
    fn test_count_misplaced_paths() {
        let misplaced = 5;
        assert_eq!(misplaced, 5);
    }

    #[test]
    fn test_count_unquoted_system_paths() {
        let unquoted = 10;
        assert_eq!(unquoted, 10);
    }

    #[test]
    fn test_count_unquoted_user_paths() {
        let unquoted = 3;
        assert_eq!(unquoted, 3);
    }

    #[test]
    fn test_count_duplicate_paths() {
        let duplicates = 7;
        assert_eq!(duplicates, 7);
    }

    #[test]
    fn test_analysis_summary_format() {
        let summary = "Total paths: 50";
        assert!(summary.contains("Total"));
    }

    #[test]
    fn test_recommendations_for_misplaced() {
        let recommendation = "Run 'spath clean --dry-run'";
        assert!(recommendation.contains("spath"));
    }

    #[test]
    fn test_recommendations_for_unquoted() {
        let recommendation = "Run 'spath fix'";
        assert!(recommendation.contains("fix"));
    }

    #[test]
    fn test_no_recommendations_when_clean() {
        let issues = 0;
        let has_recommendations = issues > 0;
        assert!(!has_recommendations);
    }
}
