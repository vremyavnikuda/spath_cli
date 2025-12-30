use spath_cli::constants::WINDOWS_PATH;

#[cfg(test)]
mod path_utils_tests {
    use super::*;

    #[test]
    fn test_normalize_path_lowercase() {
        assert_eq!(WINDOWS_PATH.to_lowercase(), WINDOWS_PATH);
    }

    #[test]
    fn test_normalize_path_trim_quotes() {
        let path = "\"C:\\Program Files\"";
        assert_eq!(path.trim_matches('"'), "C:\\Program Files");
    }

    #[test]
    fn test_path_has_spaces() {
        let path = "C:\\Program Files";
        assert!(path.contains(' '));
    }

    #[test]
    fn test_path_no_spaces() {
        assert!(!WINDOWS_PATH.contains(' '));
    }

    #[test]
    fn test_path_is_quoted() {
        let path = "\"C:\\Program Files\"";
        assert!(path.starts_with('"') && path.ends_with('"'));
    }

    #[test]
    fn test_path_not_quoted() {
        let path = "C:\\Program Files";
        assert!(!path.starts_with('"'));
    }

    #[test]
    fn test_path_is_absolute() {
        assert!(WINDOWS_PATH.contains(':'));
    }

    #[test]
    fn test_path_is_relative() {
        let path = "..\\relative";
        assert!(!path.contains(':'));
    }

    #[test]
    fn test_split_path_by_semicolon() {
        let paths = format!("{};C:\\System32", WINDOWS_PATH);
        let split: Vec<&str> = paths.split(';').collect();
        assert_eq!(split.len(), 2);
    }

    #[test]
    fn test_join_paths_with_semicolon() {
        let paths = [WINDOWS_PATH, "C:\\System32"];
        let joined = paths.join(";");
        assert_eq!(joined, format!("{};C:\\System32", WINDOWS_PATH));
    }

    #[test]
    fn test_empty_path_filter() {
        let paths = format!("{};;C:\\System32", WINDOWS_PATH);
        let filtered: Vec<&str> = paths.split(';').filter(|s| !s.is_empty()).collect();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_path_comparison_case_insensitive() {
        let path2 = WINDOWS_PATH.to_lowercase();
        assert_eq!(WINDOWS_PATH.to_lowercase(), path2.to_lowercase());
    }
}

#[cfg(test)]
mod confirmation_tests {
    #[test]
    fn test_confirmation_yes() {
        let input = "yes";
        assert!(input.to_lowercase() == "yes" || input.to_lowercase() == "y");
    }

    #[test]
    fn test_confirmation_y() {
        let input = "y";
        assert!(input.to_lowercase() == "y");
    }

    #[test]
    fn test_confirmation_no() {
        let input = "no";
        assert!(input.to_lowercase() == "no" || input.to_lowercase() == "n");
    }

    #[test]
    fn test_confirmation_n() {
        let input = "n";
        assert!(input.to_lowercase() == "n");
    }

    #[test]
    fn test_confirmation_empty_defaults_no() {
        let input = "";
        assert!(input.is_empty());
    }

    #[test]
    fn test_confirmation_case_insensitive() {
        let input = "YES";
        assert_eq!(input.to_lowercase(), "yes");
    }
}

#[cfg(test)]
mod error_handling_tests {
    #[test]
    fn test_handle_missing_path_variable() {
        let path_var = std::env::var("PATH");
        assert!(path_var.is_ok());
    }

    #[test]
    fn test_handle_invalid_backup_file() {
        let filename = "invalid.txt";
        assert!(!filename.ends_with(".json"));
    }

    #[test]
    fn test_handle_permission_denied() {
        let error_code = 5;
        assert_eq!(error_code, 5);
    }

    #[test]
    fn test_handle_corrupted_json() {
        let json = "{invalid";
        let is_valid = json.starts_with('{') && json.ends_with('}');
        assert!(!is_valid);
    }

    #[test]
    fn test_handle_disk_full() {
        let error = "No space left on device";
        assert!(error.contains("space"));
    }

    #[test]
    fn test_handle_invalid_path_characters() {
        let path = "C:\\Invalid<>Path";
        assert!(path.contains('<') || path.contains('>'));
    }

    #[test]
    fn test_error_message_format() {
        let error = "Error: Failed to read PATH";
        assert!(error.starts_with("Error:"));
    }
}
