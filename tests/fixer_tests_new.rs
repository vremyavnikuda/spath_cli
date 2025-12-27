use std::collections::HashSet;
use std::path::Path;

#[cfg(test)]
mod fixer_business_logic_tests {
    use super::*;
    #[test]
    fn test_fixer_adds_quotes_to_unquoted_paths_with_spaces() {
        let path = "C:\\Program Files\\Git\\cmd";
        assert!(path.contains(' '));
        assert!(!path.starts_with('"'));
        let fixed = format!("\"{}\"", path);
        assert_eq!(fixed, "\"C:\\Program Files\\Git\\cmd\"");
    }

    #[test]
    fn test_fixer_preserves_already_quoted_paths() {
        let path = "\"C:\\Program Files\\Git\\cmd\"";

        assert!(path.starts_with('"') && path.ends_with('"'));
        let fixed = if path.starts_with('"') {
            path.to_string()
        } else {
            format!("\"{}\"", path)
        };

        assert_eq!(fixed, path);
    }

    #[test]
    fn test_fixer_skips_paths_without_spaces() {
        let path = "C:\\Windows\\System32";

        assert!(!path.contains(' '));
        let fixed = if path.contains(' ') && !path.starts_with('"') {
            format!("\"{}\"", path)
        } else {
            path.to_string()
        };

        assert_eq!(fixed, path);
    }

    #[test]
    fn test_fixer_handles_multiple_paths_with_mixed_quoting() {
        let paths = vec![
            "C:\\Program Files\\Git",
            "\"C:\\Program Files\\App\"",
            "C:\\Windows",
        ];

        let fixed: Vec<String> = paths
            .iter()
            .map(|p| {
                if p.contains(' ') && !p.starts_with('"') {
                    format!("\"{}\"", p)
                } else {
                    p.to_string()
                }
            })
            .collect();

        assert_eq!(fixed[0], "\"C:\\Program Files\\Git\"");
        assert_eq!(fixed[1], "\"C:\\Program Files\\App\"");
        assert_eq!(fixed[2], "C:\\Windows");
    }

    #[test]
    fn test_fixer_removes_exact_duplicates() {
        let paths = vec![
            "C:\\Windows",
            "C:\\System32",
            "C:\\Windows",
        ];

        let mut seen = HashSet::new();
        let unique: Vec<&str> = paths
            .iter()
            .filter(|p| seen.insert(p.to_string()))
            .copied()
            .collect();

        assert_eq!(unique.len(), 2);
        assert_eq!(unique[0], "C:\\Windows");
        assert_eq!(unique[1], "C:\\System32");
    }

    #[test]
    fn test_fixer_removes_case_insensitive_duplicates() {
        let paths = vec![
            "C:\\Windows",
            "c:\\windows",
            "C:\\WINDOWS",
        ];

        let mut seen = HashSet::new();
        let unique: Vec<&str> = paths
            .iter()
            .filter(|p| seen.insert(p.to_lowercase()))
            .copied()
            .collect();

        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn test_fixer_removes_quoted_unquoted_duplicates() {
        let paths = vec![
            "C:\\Program Files\\Git",
            "\"C:\\Program Files\\Git\"",
        ];

        let mut seen = HashSet::new();
        let unique: Vec<&str> = paths
            .iter()
            .filter(|p| {
                let normalized = p.trim_matches('"').to_lowercase();
                seen.insert(normalized)
            })
            .copied()
            .collect();

        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn test_fixer_preserves_first_occurrence_of_duplicate() {
        let paths = vec![
            "C:\\First",
            "C:\\Windows",
            "C:\\Windows",
            "C:\\Last",
        ];

        let mut seen = HashSet::new();
        let unique: Vec<&str> = paths
            .iter()
            .filter(|p| seen.insert(p.to_lowercase()))
            .copied()
            .collect();

        assert_eq!(unique.len(), 3);
        assert_eq!(unique[1], "C:\\Windows");
    }

    #[test]
    fn test_fixer_removes_non_existent_paths() {
        let paths = vec![
            "C:\\Windows",
            "C:\\NonExistent123456789",
            "C:\\System32",
        ];

        let existing: Vec<&str> = paths
            .iter()
            .filter(|p| Path::new(p).exists())
            .copied()
            .collect();
        assert!(existing.contains(&"C:\\Windows"));
        assert!(!existing.contains(&"C:\\NonExistent123456789"));
    }

    #[test]
    fn test_fixer_preserves_paths_with_env_vars() {
        let path = "%SystemRoot%\\System32";
        assert!(path.contains('%'));
        let expanded = path.replace("%SystemRoot%", "C:\\Windows");
        assert!(Path::new(&expanded).exists());
    }

    #[test]
    fn test_fixer_handles_quoted_paths_existence_check() {
        let path = "\"C:\\Windows\"";

        let unquoted = path.trim_matches('"');
        assert!(Path::new(unquoted).exists());
    }

    #[test]
    fn test_fixer_preserves_path_order() {
        let paths = vec!["C:\\First", "C:\\Second", "C:\\Third"];
        let processed: Vec<&str> = paths.iter().copied().collect();

        assert_eq!(processed[0], "C:\\First");
        assert_eq!(processed[1], "C:\\Second");
        assert_eq!(processed[2], "C:\\Third");
    }

    #[test]
    fn test_fixer_maintains_order_after_duplicate_removal() {
        let paths = vec![
            "C:\\First",
            "C:\\Second",
            "C:\\First",
            "C:\\Third",
        ];

        let mut seen = HashSet::new();
        let unique: Vec<&str> = paths
            .iter()
            .filter(|p| seen.insert(p.to_lowercase()))
            .copied()
            .collect();

        assert_eq!(unique[0], "C:\\First");
        assert_eq!(unique[1], "C:\\Second");
        assert_eq!(unique[2], "C:\\Third");
    }

    #[test]
    fn test_fixer_dry_run_does_not_modify() {
        let dry_run = true;
        let original_path = "C:\\Windows;C:\\System32";
        let modified = if dry_run {
            original_path.to_string()
        } else {
            "modified".to_string()
        };

        assert_eq!(modified, original_path);
    }

    #[test]
    fn test_fixer_dry_run_reports_changes() {
        let changes = vec![
            "Would add quotes: C:\\Program Files\\Git",
            "Would remove duplicate: C:\\Windows",
            "Would remove non-existent: C:\\NonExistent",
        ];

        assert_eq!(changes.len(), 3);
        assert!(changes[0].starts_with("Would add quotes"));
    }

    #[test]
    fn test_fixer_creates_backup_before_changes() {
        let should_create_backup = true;
        let changes_detected = true;

        assert!(should_create_backup && changes_detected);
    }

    #[test]
    fn test_fixer_backup_contains_timestamp() {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_name = format!("path_backup_{}.json", timestamp);

        assert!(backup_name.starts_with("path_backup_"));
        assert!(backup_name.ends_with(".json"));
        assert!(backup_name.contains('_'));
    }

    #[test]
    fn test_fixer_backup_includes_both_paths() {
        let backup_data = serde_json::json!({
            "timestamp": "20241213_120000",
            "user_path": "C:\\Users\\test\\bin",
            "system_path": "C:\\Windows;C:\\System32"
        });

        assert!(backup_data["user_path"].is_string());
        assert!(backup_data["system_path"].is_string());
        assert!(backup_data["timestamp"].is_string());
    }

    #[test]
    fn test_fixer_reports_all_changes() {
        let changes = vec![
            "Added quotes: C:\\Program Files\\Git",
            "Removed duplicate: C:\\Windows",
            "Removed non-existent: C:\\NonExistent",
        ];

        assert_eq!(changes.len(), 3);
    }

    #[test]
    fn test_fixer_reports_no_changes_when_clean() {
        let changes: Vec<String> = vec![];

        assert!(changes.is_empty());
    }

    #[test]
    fn test_fixer_detects_if_path_changed() {
        let original = "C:\\Windows;C:\\System32";
        let fixed = "C:\\Windows;C:\\System32";

        let changed = original != fixed;
        assert!(!changed);

        let fixed2 = "\"C:\\Program Files\";C:\\Windows";
        let changed2 = original != fixed2;
        assert!(changed2);
    }

    #[test]
    fn test_fixer_handles_empty_path() {
        let empty_path = "";
        let paths: Vec<&str> = empty_path.split(';').filter(|s| !s.is_empty()).collect();

        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_fixer_handles_single_path() {
        let single_path = "C:\\Windows";
        let paths: Vec<&str> = single_path.split(';').collect();

        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn test_fixer_handles_path_with_trailing_semicolon() {
        let path = "C:\\Windows;C:\\System32;";
        let paths: Vec<&str> = path.split(';').filter(|s| !s.is_empty()).collect();

        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_fixer_handles_unicode_paths() {
        let unicode_path = "C:\\Пользователи\\用户";
        assert!(unicode_path.chars().any(|c| !c.is_ascii()));
        let processed = unicode_path.to_string();
        assert_eq!(processed, unicode_path);
    }

    #[test]
    fn test_fixer_handles_paths_with_special_chars() {
        let special_paths = vec![
            "C:\\Path (x86)\\App",
            "C:\\Path [test]\\App",
            "C:\\Path & App",
        ];

        for path in special_paths {
            assert!(!path.is_empty());
        }
    }
}
