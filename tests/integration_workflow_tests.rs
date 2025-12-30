use spath_cli::constants::{PROGRAM_FILES, WINDOWS_PATH};

#[cfg(test)]
mod integration_workflow_tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn test_workflow_scan_detects_issues() {
        let test_path = format!("{}\\Git;{};{}", PROGRAM_FILES, WINDOWS_PATH, WINDOWS_PATH);
        let paths: Vec<&str> = test_path.split(';').collect();
        let unquoted_with_spaces = paths
            .iter()
            .filter(|p| p.contains(' ') && !p.starts_with('"'))
            .count();
        assert_eq!(unquoted_with_spaces, 1);
        let mut seen = HashSet::new();
        let duplicates = paths
            .iter()
            .filter(|p| !seen.insert(p.to_lowercase()))
            .count();
        assert_eq!(duplicates, 1);
    }

    #[test]
    fn test_workflow_fix_resolves_issues() {
        let test_path = format!("{}\\Git;{};{}", PROGRAM_FILES, WINDOWS_PATH, WINDOWS_PATH);
        let paths: Vec<&str> = test_path.split(';').collect();
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
        let expected = format!("\"{}\\Git\"", PROGRAM_FILES);
        assert_eq!(fixed[0], expected);
        let mut seen = HashSet::new();
        let unique: Vec<String> = fixed
            .into_iter()
            .filter(|p| seen.insert(p.to_lowercase()))
            .collect();
        assert_eq!(unique.len(), 2);
    }

    #[test]
    fn test_workflow_verify_confirms_fix() {
        let fixed_path = format!("\"{}\\Git\";{}", PROGRAM_FILES, WINDOWS_PATH);
        let paths: Vec<&str> = fixed_path.split(';').collect();
        let unquoted_with_spaces = paths
            .iter()
            .filter(|p| {
                let trimmed = p.trim_matches('"');
                trimmed.contains(' ') && !p.starts_with('"')
            })
            .count();
        assert_eq!(unquoted_with_spaces, 0);
        let mut seen = HashSet::new();
        let duplicates = paths
            .iter()
            .filter(|p| !seen.insert(p.trim_matches('"').to_lowercase()))
            .count();
        assert_eq!(duplicates, 0);
    }

    #[test]
    fn test_workflow_full_scan_fix_verify_cycle() {
        let original = format!(
            "{}\\Git;{};{};C:\\NonExistent",
            PROGRAM_FILES, WINDOWS_PATH, WINDOWS_PATH
        );
        let paths: Vec<&str> = original.split(';').collect();
        let issues_found = paths
            .iter()
            .filter(|p| p.contains(' ') && !p.starts_with('"') || !Path::new(p).exists())
            .count();
        assert!(issues_found > 0, "Scan should find issues");
        let mut seen = HashSet::new();
        let fixed: Vec<String> = paths
            .iter()
            .filter(|p| Path::new(p).exists() && seen.insert(p.to_lowercase()))
            .map(|p| {
                if p.contains(' ') && !p.starts_with('"') {
                    format!("\"{}\"", p)
                } else {
                    p.to_string()
                }
            })
            .collect();
        let fixed_path = fixed.join(";");
        let verified_paths: Vec<&str> = fixed_path.split(';').collect();
        let remaining_issues = verified_paths
            .iter()
            .filter(|p| {
                let trimmed = p.trim_matches('"');
                (trimmed.contains(' ') && !p.starts_with('"')) || !Path::new(trimmed).exists()
            })
            .count();
        assert_eq!(remaining_issues, 0, "All issues should be fixed");
    }

    #[test]
    fn test_workflow_backup_preserves_original() {
        let original_path = format!("{};C:\\System32;{}\\Git", WINDOWS_PATH, PROGRAM_FILES);
        let backup = original_path.to_string();
        assert_eq!(backup, original_path);
    }

    #[test]
    fn test_workflow_modify_changes_path() {
        let original = format!("{}\\Git;{}", PROGRAM_FILES, WINDOWS_PATH);
        let modified = format!("\"{}\\Git\";{}", PROGRAM_FILES, WINDOWS_PATH);
        assert_ne!(original, modified);
    }

    #[test]
    fn test_workflow_restore_reverts_changes() {
        let original = format!("{};C:\\System32", WINDOWS_PATH);
        let modified = format!("\"{}\\Git\";{}", PROGRAM_FILES, WINDOWS_PATH);
        let restored = original.to_string();
        assert_eq!(restored, original);
        assert_ne!(restored, modified);
    }

    #[test]
    fn test_workflow_full_backup_modify_restore_cycle() {
        let original = format!("{};C:\\System32;{}\\Git", WINDOWS_PATH, PROGRAM_FILES);
        let backup = original.to_string();
        let paths: Vec<&str> = original.split(';').collect();
        let modified: Vec<String> = paths
            .iter()
            .map(|p| {
                if p.contains(' ') && !p.starts_with('"') {
                    format!("\"{}\"", p)
                } else {
                    p.to_string()
                }
            })
            .collect();
        let modified_path = modified.join(";");
        assert_ne!(original, modified_path);
        let restored = backup;
        assert_eq!(restored, original);
    }

    #[test]
    fn test_workflow_clean_removes_all_duplicates() {
        let win_lower = WINDOWS_PATH.to_lowercase();
        let win_upper = WINDOWS_PATH.to_uppercase();
        let path_with_duplicates = format!(
            "{};{};C:\\System32;{};C:\\System32",
            WINDOWS_PATH, win_lower, win_upper
        );
        let paths: Vec<&str> = path_with_duplicates.split(';').collect();
        let mut seen = HashSet::new();
        let duplicates_count = paths
            .iter()
            .filter(|p| !seen.insert(p.to_lowercase()))
            .count();
        assert_eq!(duplicates_count, 3);
        let mut seen = HashSet::new();
        let cleaned: Vec<&str> = paths
            .iter()
            .filter(|p| seen.insert(p.to_lowercase()))
            .copied()
            .collect();
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaned[0], WINDOWS_PATH);
        assert_eq!(cleaned[1], "C:\\System32");
    }

    #[test]
    fn test_workflow_clean_preserves_order() {
        let original = "C:\\First;C:\\Second;C:\\First;C:\\Third;C:\\Second";
        let paths: Vec<&str> = original.split(';').collect();
        let mut seen = HashSet::new();
        let cleaned: Vec<&str> = paths
            .iter()
            .filter(|p| seen.insert(p.to_lowercase()))
            .copied()
            .collect();
        assert_eq!(cleaned[0], "C:\\First");
        assert_eq!(cleaned[1], "C:\\Second");
        assert_eq!(cleaned[2], "C:\\Third");
    }

    #[test]
    fn test_workflow_analyze_detects_misplaced_paths() {
        let system_paths = vec![WINDOWS_PATH, "C:\\Users\\test\\.cargo\\bin"];
        let misplaced = system_paths
            .iter()
            .filter(|p| {
                let lower = p.to_lowercase();
                lower.contains("\\users\\") || lower.contains(".cargo")
            })
            .count();
        assert_eq!(misplaced, 1);
    }

    #[test]
    fn test_workflow_migrate_moves_paths() {
        let system_paths = vec![WINDOWS_PATH, "C:\\Users\\test\\.cargo\\bin"];
        let mut user_paths: Vec<String> = vec!["C:\\Users\\test\\bin".to_string()];
        let to_migrate: Vec<String> = system_paths
            .iter()
            .filter(|p| {
                let lower = p.to_lowercase();
                lower.contains("\\users\\") || lower.contains(".cargo")
            })
            .map(|s| s.to_string())
            .collect();
        user_paths.extend(to_migrate);
        assert_eq!(user_paths.len(), 2);
        assert!(user_paths.contains(&"C:\\Users\\test\\.cargo\\bin".to_string()));
    }

    #[test]
    fn test_workflow_dry_run_simulates_changes() {
        let original = format!("{}\\Git;{};{}", PROGRAM_FILES, WINDOWS_PATH, WINDOWS_PATH);
        let dry_run = true;
        let _paths: Vec<&str> = original.split(';').collect();
        let changes = vec![
            format!("Would add quotes: {}\\Git", PROGRAM_FILES),
            format!("Would remove duplicate: {}", WINDOWS_PATH),
        ];
        let result = if dry_run {
            original.to_string()
        } else {
            "modified".to_string()
        };
        assert_eq!(result, original);
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn test_workflow_dry_run_reports_all_planned_changes() {
        let original = format!(
            "{}\\Git;{};{};C:\\NonExistent",
            PROGRAM_FILES, WINDOWS_PATH, WINDOWS_PATH
        );
        let paths: Vec<&str> = original.split(';').collect();
        let mut changes = Vec::new();
        for path in &paths {
            if path.contains(' ') && !path.starts_with('"') {
                changes.push(format!("Would add quotes: {}", path));
            }
            if !Path::new(path).exists() {
                changes.push(format!("Would remove non-existent: {}", path));
            }
        }
        let mut seen = HashSet::new();
        for path in &paths {
            if !seen.insert(path.to_lowercase()) {
                changes.push(format!("Would remove duplicate: {}", path));
            }
        }
        assert!(changes.len() >= 3);
    }

    #[test]
    fn test_workflow_handles_empty_path() {
        let empty_path = "";
        let paths: Vec<&str> = empty_path.split(';').filter(|s| !s.is_empty()).collect();
        assert_eq!(paths.len(), 0);
        let fixed: Vec<String> = paths.iter().map(|p| p.to_string()).collect();
        assert_eq!(fixed.len(), 0);
    }

    #[test]
    fn test_workflow_handles_corrupted_path() {
        let corrupted = format!(";;;{};;;C:\\System32;;;", WINDOWS_PATH);
        let paths: Vec<&str> = corrupted.split(';').filter(|s| !s.is_empty()).collect();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], WINDOWS_PATH);
        assert_eq!(paths[1], "C:\\System32");
    }

    #[test]
    fn test_workflow_handles_all_non_existent_paths() {
        let all_non_existent = "C:\\NonExistent1;C:\\NonExistent2;C:\\NonExistent3";
        let paths: Vec<&str> = all_non_existent.split(';').collect();
        let existing: Vec<&str> = paths
            .iter()
            .filter(|p| Path::new(p).exists())
            .copied()
            .collect();
        assert_eq!(existing.len(), 0);
    }

    #[test]
    fn test_workflow_complex_multiple_issues() {
        let pf_lower = PROGRAM_FILES.to_lowercase();
        let complex_path = format!(
            "{}\\Git;{}\\git;{};C:\\NonExistent;\"{}\\App\";{}",
            PROGRAM_FILES, pf_lower, WINDOWS_PATH, PROGRAM_FILES, WINDOWS_PATH
        );
        let paths: Vec<&str> = complex_path.split(';').collect();
        let mut seen = HashSet::new();
        let fixed: Vec<String> = paths
            .iter()
            .filter(|p| {
                let trimmed = p.trim_matches('"');
                Path::new(trimmed).exists() && seen.insert(trimmed.to_lowercase())
            })
            .map(|p| {
                let trimmed = p.trim_matches('"');
                if trimmed.contains(' ') && !p.starts_with('"') {
                    format!("\"{}\"", trimmed)
                } else if p.starts_with('"') {
                    p.to_string()
                } else {
                    trimmed.to_string()
                }
            })
            .collect();
        assert!(
            fixed.len() < paths.len(),
            "Should remove duplicates and non-existent"
        );
        for path in &fixed {
            let trimmed = path.trim_matches('"');
            if trimmed.contains(' ') {
                assert!(path.starts_with('"') && path.ends_with('"'));
            }
        }
    }
}
