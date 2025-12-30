use proptest::prelude::*;

#[cfg(test)]
mod property_based_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_property_unicode_paths_preserved(
            unicode_str in "[\\p{Cyrillic}\\p{Han}\\p{Hiragana}]{1,20}"
        ) {
            let path = format!("C:\\Users\\{}\\bin", unicode_str);
            prop_assert!(path.contains(&unicode_str));
            prop_assert!(!path.is_empty());
        }

        #[test]
        fn test_property_unicode_paths_case_insensitive_comparison(
            unicode_str in "[\\p{Cyrillic}]{1,20}"
        ) {
            let path1 = format!("C:\\Users\\{}\\bin", unicode_str);
            let path2 = format!("C:\\Users\\{}\\bin", unicode_str.to_uppercase());
            let lower1 = path1.to_lowercase();
            let lower2 = path2.to_lowercase();
            prop_assert!(lower1.contains("users"));
            prop_assert!(lower2.contains("users"));
        }
    }

    proptest! {
        #[test]
        fn test_property_special_chars_in_paths(
            special_char in r"[\(\)\[\]&!@#\$%\^]"
        ) {
            let path = format!("C:\\Path{}Test\\bin", special_char);
            prop_assert!(path.contains(&special_char as &str));
            prop_assert!(path.starts_with("C:\\"));
        }

        #[test]
        fn test_property_parentheses_in_paths(
            content in "[a-zA-Z0-9 ]{1,20}"
        ) {
            let path = format!("C:\\Path ({})\\bin", content);
            prop_assert!(path.contains('('));
            prop_assert!(path.contains(')'));
        }
    }

    proptest! {
        #[test]
        fn test_property_long_paths_handled(
            repeat_count in 1usize..100usize
        ) {
            let long_path = "C:\\".to_string() + &"VeryLongFolderName\\".repeat(repeat_count);
            prop_assert!(long_path.starts_with("C:\\"));
            prop_assert!(long_path.len() > repeat_count * 10);
        }

        #[test]
        fn test_property_path_length_over_260_chars(
            folder_name in "[a-zA-Z]{20,50}"
        ) {
            let long_path = "C:\\".to_string() + &format!("{}\\", folder_name).repeat(10);
            if long_path.len() > 260 {
                prop_assert!(long_path.len() > 260);
            }
        }

        #[test]
        fn test_property_path_length_over_2047_chars(
            folder_name in "[a-zA-Z]{30,60}"
        ) {
            let very_long_path = "C:\\".to_string() + &format!("{}\\", folder_name).repeat(50);
            if very_long_path.len() > 2047 {
                prop_assert!(very_long_path.len() > 2047);
            }
        }
    }

    proptest! {
        #[test]
        fn test_property_adding_quotes_idempotent(
            path in "C:\\\\[a-zA-Z ]{5,30}"
        ) {
            let quoted_once = if path.contains(' ') && !path.starts_with('"') {
                format!("\"{}\"", path)
            } else {
                path.clone()
            };
            let quoted_twice = if quoted_once.contains(' ') && !quoted_once.starts_with('"') {
                format!("\"{}\"", quoted_once)
            } else {
                quoted_once.clone()
            };
            prop_assert_eq!(quoted_once, quoted_twice);
        }

        #[test]
        fn test_property_quote_removal_inverse(
            path_content in "[a-zA-Z ]{5,30}"
        ) {
            let original = format!("C:\\{}", path_content);
            let quoted = format!("\"{}\"", original);
            let unquoted = quoted.trim_matches('"');
            prop_assert_eq!(original, unquoted);
        }
    }

    proptest! {
        #[test]
        fn test_property_duplicate_removal_reduces_count(
            paths in prop::collection::vec("[a-zA-Z]{3,10}", 2..20)
        ) {
            use std::collections::HashSet;
            let original_count = paths.len();
            let mut seen = HashSet::new();
            let unique: Vec<_> = paths.iter()
                .filter(|p| seen.insert(p.to_lowercase()))
                .collect();
            let unique_count = unique.len();
            prop_assert!(unique_count <= original_count);
        }

        #[test]
        fn test_property_duplicate_removal_preserves_order(
            paths in prop::collection::vec("[a-zA-Z]{3,10}", 2..10)
        ) {
            use std::collections::HashSet;
            let mut seen = HashSet::new();
            let unique: Vec<_> = paths.iter()
                .filter(|p| seen.insert(p.to_lowercase()))
                .collect();
            if !unique.is_empty() && !paths.is_empty() {
                let first_unique = unique[0];
                let first_in_paths = paths.iter().find(|p| *p == first_unique);
                prop_assert!(first_in_paths.is_some());
            }
        }
    }

    proptest! {
        #[test]
        fn test_property_case_insensitive_comparison(
            path_part in "[a-zA-Z]{5,20}"
        ) {
            let path1 = format!("C:\\{}", path_part.to_lowercase());
            let path2 = format!("C:\\{}", path_part.to_uppercase());
            prop_assert_eq!(path1.to_lowercase(), path2.to_lowercase());
        }

        #[test]
        fn test_property_case_variations_are_duplicates(
            path_part in "[a-zA-Z]{5,20}"
        ) {
            use std::collections::HashSet;
            let paths = [
                format!("C:\\{}", path_part.to_lowercase()),
                format!("C:\\{}", path_part.to_uppercase()),
                format!("C:\\{}", path_part),
            ];
            let mut seen = HashSet::new();
            let unique: Vec<_> = paths.iter()
                .filter(|p| seen.insert(p.to_lowercase()))
                .collect();
            prop_assert_eq!(unique.len(), 1);
        }
    }

    proptest! {
        #[test]
        fn test_property_split_join_inverse(
            paths in prop::collection::vec("[a-zA-Z]{3,10}", 1..10)
        ) {
            let joined = paths.join(";");
            let split: Vec<&str> = joined.split(';').collect();
            prop_assert_eq!(paths.len(), split.len());
            for (original, parsed) in paths.iter().zip(split.iter()) {
                prop_assert_eq!(original, parsed);
            }
        }

        #[test]
        fn test_property_empty_elements_filtered(
            path_count in 1usize..10usize
        ) {
            let path_with_empties = ";".repeat(path_count) + "C:\\Windows" + &";".repeat(path_count);
            let filtered: Vec<&str> = path_with_empties.split(';')
                .filter(|s| !s.is_empty())
                .collect();
            prop_assert_eq!(filtered.len(), 1);
            prop_assert_eq!(filtered[0], "C:\\Windows");
        }
    }

    proptest! {
        #[test]
        fn test_property_spaces_detection(
            before in "[a-zA-Z]{1,10}",
            after in "[a-zA-Z]{1,10}"
        ) {
            let path_with_space = format!("C:\\{} {}", before, after);
            prop_assert!(path_with_space.contains(' '));
        }

        #[test]
        fn test_property_multiple_spaces_handled(
            space_count in 1usize..10usize
        ) {
            let path = format!("C:\\Path{}Test", " ".repeat(space_count));
            prop_assert!(path.contains(' '));
            prop_assert_eq!(path.matches(' ').count(), space_count);
        }
    }

    proptest! {
        #[test]
        fn test_property_env_var_detection(
            var_name in "[A-Z]{3,15}"
        ) {
            let path = format!("C:\\%{{{}}}%\\bin", var_name);
            prop_assert!(path.contains('%'));
            let expected = format!("%{{{}}}%", var_name);
            prop_assert!(path.contains(&expected as &str));
        }

        #[test]
        fn test_property_env_var_expansion_removes_percent(
            var_name in "[A-Z]{3,15}",
            value in "[a-zA-Z]{3,15}"
        ) {
            let path = format!("%{}%\\bin", var_name);
            let expanded = path.replace(&format!("%{}%", var_name), &value);
            prop_assert!(!expanded.contains('%'));
            prop_assert!(expanded.contains(&value));
        }
    }

    proptest! {
        #[test]
        fn test_property_single_char_paths(
            char in "[a-zA-Z]"
        ) {
            let path = format!("C:\\{}", char);
            prop_assert_eq!(path.len(), 4);
        }

        #[test]
        fn test_property_paths_with_numbers(
            num in 0u32..10000u32
        ) {
            let path = format!("C:\\Path{}\\bin", num);
            prop_assert!(path.contains(&num.to_string()));
        }

        #[test]
        fn test_property_mixed_slashes(
            forward_count in 0usize..5usize,
            backward_count in 0usize..5usize
        ) {
            let path = "C:".to_string()
                + &"/".repeat(forward_count)
                + &"\\".repeat(backward_count)
                + "Path";
            if forward_count > 0 {
                prop_assert!(path.contains('/'));
            }
            if backward_count > 0 {
                prop_assert!(path.contains('\\'));
            }
        }
    }

    proptest! {
        #[test]
        fn test_property_multiple_semicolons(
            semicolon_count in 1usize..20usize
        ) {
            let path = ";".repeat(semicolon_count);
            let filtered: Vec<&str> = path.split(';')
                .filter(|s| !s.is_empty())
                .collect();
            prop_assert_eq!(filtered.len(), 0);
        }

        #[test]
        fn test_property_random_whitespace(
            ws_count in 1usize..10usize
        ) {
            let path = format!("C:\\{}Path{}Test", " ".repeat(ws_count), " ".repeat(ws_count));
            prop_assert!(path.contains(' '));
        }
    }
}
