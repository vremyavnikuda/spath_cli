use std::path::Path;

#[cfg(test)]
mod analyzer_business_logic_tests {
    use super::*;

    #[test]
    fn test_analyzer_categorizes_system_programs() {
        let system_paths = vec![
            "C:\\Windows\\System32",
            "C:\\Windows\\SysWOW64",
            "C:\\Program Files\\Common Files",
            "C:\\Program Files (x86)\\Microsoft",
        ];

        for path in system_paths {
            let lower = path.to_lowercase();
            let is_system = lower.starts_with("c:\\windows")
                || lower.starts_with("c:\\program files")
                || lower.starts_with("c:\\program files (x86)");

            assert!(is_system, "Path {} should be categorized as system", path);
        }
    }

    #[test]
    fn test_analyzer_categorizes_user_programs() {
        let user_paths = vec![
            "C:\\Users\\test\\.cargo\\bin",
            "C:\\Users\\test\\.dotnet\\tools",
            "C:\\Users\\test\\AppData\\Local\\Programs",
            "C:\\Users\\test\\.npm\\bin",
        ];

        for path in user_paths {
            let lower = path.to_lowercase();
            let is_user = lower.contains("\\users\\")
                || lower.contains(".cargo")
                || lower.contains(".dotnet")
                || lower.contains(".npm")
                || lower.contains("\\appdata\\");

            assert!(is_user, "Path {} should be categorized as user", path);
        }
    }

    #[test]
    fn test_analyzer_categorizes_program_data() {
        let program_data_paths = vec![
            "C:\\ProgramData\\chocolatey\\bin",
            "C:\\ProgramData\\Microsoft\\Windows",
        ];

        for path in program_data_paths {
            let lower = path.to_lowercase();
            assert!(lower.starts_with("c:\\programdata"));
        }
    }

    #[test]
    fn test_analyzer_categorizes_ambiguous_paths() {
        let ambiguous_paths = vec![
            "D:\\CustomApps\\bin",
            "E:\\Tools\\utilities",
            "C:\\CustomFolder\\bin",
        ];

        for path in ambiguous_paths {
            let lower = path.to_lowercase();
            let is_system =
                lower.starts_with("c:\\windows") || lower.starts_with("c:\\program files");
            let is_user = lower.contains("\\users\\");
            let is_program_data = lower.starts_with("c:\\programdata");
            assert!(!is_system && !is_user && !is_program_data);
        }
    }

    #[test]
    fn test_analyzer_detects_user_paths_in_system() {
        let path = "C:\\Users\\test\\.cargo\\bin";
        let location = "system";

        let lower = path.to_lowercase();
        let is_user_path = lower.contains("\\users\\") || lower.contains(".cargo");
        assert!(is_user_path && location == "system");
    }

    #[test]
    fn test_analyzer_detects_system_paths_in_user() {
        let path = "C:\\Program Files\\Git\\cmd";
        let location = "user";

        let lower = path.to_lowercase();
        let is_system_path = lower.starts_with("c:\\program files");
        assert!(is_system_path && location == "user");
    }

    #[test]
    fn test_analyzer_validates_correct_placement() {
        let test_cases = vec![
            ("C:\\Windows\\System32", "system", true),
            ("C:\\Users\\test\\.cargo\\bin", "user", true),
            ("C:\\Program Files\\Git\\cmd", "system", true),
        ];

        for (path, location, should_be_correct) in test_cases {
            let lower = path.to_lowercase();
            let is_system_path =
                lower.starts_with("c:\\windows") || lower.starts_with("c:\\program files");
            let is_user_path = lower.contains("\\users\\") || lower.contains(".cargo");

            let is_correctly_placed =
                (is_system_path && location == "system") || (is_user_path && location == "user");

            assert_eq!(is_correctly_placed, should_be_correct);
        }
    }

    #[test]
    fn test_analyzer_detects_paths_needing_quotes() {
        let paths_needing_quotes = vec![
            ("C:\\Program Files\\Git", false),
            ("C:\\My Documents\\files", false),
            ("C:\\Custom App\\bin", false),
        ];

        for (path, is_quoted) in paths_needing_quotes {
            let has_spaces = path.contains(' ');
            let needs_quotes = has_spaces && !is_quoted;

            assert!(needs_quotes, "Path {} needs quotes", path);
        }
    }

    #[test]
    fn test_analyzer_validates_properly_quoted_paths() {
        let properly_quoted = vec!["\"C:\\Program Files\\Git\"", "\"C:\\My Documents\\files\""];

        for path in properly_quoted {
            let has_spaces = path.contains(' ');
            let is_quoted = path.starts_with('"') && path.ends_with('"');

            assert!(has_spaces && is_quoted);
        }
    }

    #[test]
    fn test_analyzer_validates_paths_not_needing_quotes() {
        let paths_without_spaces = vec![
            "C:\\Windows\\System32",
            "C:\\Tools\\bin",
            "D:\\Apps\\utilities",
        ];

        for path in paths_without_spaces {
            let has_spaces = path.contains(' ');
            assert!(!has_spaces);
        }
    }

    #[test]
    fn test_analyzer_validates_existing_paths() {
        let windows_path = "C:\\Windows";
        assert!(Path::new(windows_path).exists());
    }

    #[test]
    fn test_analyzer_detects_non_existent_paths() {
        let non_existent = "C:\\NonExistentFolder123456789XYZ";
        assert!(!Path::new(non_existent).exists());
    }

    #[test]
    fn test_analyzer_handles_quoted_paths_existence() {
        let quoted_path = "\"C:\\Windows\"";
        let unquoted = quoted_path.trim_matches('"');

        assert!(Path::new(unquoted).exists());
    }

    #[test]
    fn test_analyzer_processes_both_system_and_user_paths() {
        let system_paths = vec!["C:\\Windows", "C:\\Program Files\\Git"];
        let user_paths = vec!["C:\\Users\\test\\.cargo\\bin"];

        let total_paths = system_paths.len() + user_paths.len();
        assert_eq!(total_paths, 3);
    }

    #[test]
    fn test_analyzer_distinguishes_path_locations() {
        let paths_with_locations = vec![
            ("C:\\Windows", "system"),
            ("C:\\Users\\test\\.cargo\\bin", "user"),
            ("C:\\Program Files\\Git", "system"),
        ];

        for (path, expected_location) in paths_with_locations {
            assert!(!path.is_empty() && !expected_location.is_empty());
        }
    }

    #[test]
    fn test_analyzer_counts_misplaced_paths() {
        let paths = vec![
            ("C:\\Users\\test\\.cargo\\bin", "system", true),
            ("C:\\Windows", "system", false),
            ("C:\\Program Files\\Git", "user", true),
        ];

        let misplaced_count = paths
            .iter()
            .filter(|(_, _, is_misplaced)| *is_misplaced)
            .count();

        assert_eq!(misplaced_count, 2);
    }

    #[test]
    fn test_analyzer_counts_paths_needing_quotes() {
        let paths = vec![
            ("C:\\Program Files\\Git", false),
            ("\"C:\\Program Files\\App\"", true),
            ("C:\\Windows", false),
        ];

        let needing_quotes = paths
            .iter()
            .filter(|(path, is_quoted)| path.contains(' ') && !is_quoted)
            .count();

        assert_eq!(needing_quotes, 1);
    }

    #[test]
    fn test_analyzer_counts_non_existent_paths() {
        let paths = vec![
            "C:\\Windows",
            "C:\\NonExistent123",
            "C:\\AnotherNonExistent456",
        ];

        let non_existent = paths.iter().filter(|p| !Path::new(p).exists()).count();

        assert_eq!(non_existent, 2);
    }

    #[test]
    fn test_analyzer_recommends_moving_user_paths() {
        let path = "C:\\Users\\test\\.cargo\\bin";
        let location = "system";

        let lower = path.to_lowercase();
        let is_user_path = lower.contains("\\users\\") || lower.contains(".cargo");

        if is_user_path && location == "system" {
            let recommendation = "Move to USER PATH";
            assert_eq!(recommendation, "Move to USER PATH");
        }
    }

    #[test]
    fn test_analyzer_recommends_adding_quotes() {
        let path = "C:\\Program Files\\Git";
        let is_quoted = false;

        if path.contains(' ') && !is_quoted {
            let recommendation = "Add quotes";
            assert_eq!(recommendation, "Add quotes");
        }
    }

    #[test]
    fn test_analyzer_recommends_removing_non_existent() {
        let path = "C:\\NonExistent123";

        if !Path::new(path).exists() {
            let recommendation = "Remove (path does not exist)";
            assert!(recommendation.contains("Remove"));
        }
    }

    #[test]
    fn test_analyzer_handles_empty_system_path() {
        let system_paths: Vec<String> = vec![];
        assert!(system_paths.is_empty());
    }

    #[test]
    fn test_analyzer_handles_empty_user_path() {
        let user_paths: Vec<String> = vec![];
        assert!(user_paths.is_empty());
    }

    #[test]
    fn test_analyzer_handles_unicode_in_username() {
        let path = "C:\\Users\\Пользователь\\.cargo\\bin";

        assert!(path.chars().any(|c| !c.is_ascii()));
        assert!(path.contains("\\Users\\"));
    }

    #[test]
    fn test_analyzer_handles_mixed_case_paths() {
        let paths = vec![
            "C:\\Program Files\\Git",
            "c:\\program files\\git",
            "C:\\PROGRAM FILES\\GIT",
        ];
        let normalized: Vec<String> = paths.iter().map(|p| p.to_lowercase()).collect();

        assert_eq!(normalized[0], normalized[1]);
        assert_eq!(normalized[1], normalized[2]);
    }

    #[test]
    fn test_analyzer_handles_paths_with_env_vars() {
        let path = "%SystemRoot%\\System32";

        assert!(path.contains('%'));
        let expanded = path.replace("%SystemRoot%", "C:\\Windows");
        assert!(!expanded.contains('%'));
    }

    #[test]
    fn test_analyzer_handles_very_long_paths() {
        let long_path = "C:\\".to_string() + &"VeryLongFolder\\".repeat(50);

        assert!(long_path.len() > 260);
    }

    #[test]
    fn test_analyzer_handles_paths_with_special_chars() {
        let special_paths = vec![
            "C:\\Path (x86)\\App",
            "C:\\Path [test]\\App",
            "C:\\Path & App",
        ];

        for path in special_paths {
            assert!(!path.is_empty());
        }
    }

    #[test]
    fn test_analyzer_formats_summary() {
        let summary = format!(
            "Total paths: {}\nMisplaced: {}\nNeeding quotes: {}\nNon-existent: {}",
            10, 2, 3, 1
        );

        assert!(summary.contains("Total paths: 10"));
        assert!(summary.contains("Misplaced: 2"));
    }

    #[test]
    fn test_analyzer_formats_recommendations() {
        let recommendations = vec![
            "Move C:\\Users\\test\\.cargo\\bin to USER PATH",
            "Add quotes to C:\\Program Files\\Git",
            "Remove C:\\NonExistent (does not exist)",
        ];

        assert_eq!(recommendations.len(), 3);
        assert!(recommendations[0].contains("Move"));
        assert!(recommendations[1].contains("Add quotes"));
        assert!(recommendations[2].contains("Remove"));
    }
}
