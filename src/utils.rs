use crate::constants::{PROGRAM_DATA, PROGRAM_FILES, PROGRAM_FILES_X86, USER_PATHS, WINDOWS_PATH};
use crate::models::PathCategory;
use std::env;

pub fn categorize_path(path: &str) -> PathCategory {
    let lower = path.to_lowercase();
    if lower.starts_with(WINDOWS_PATH)
        || lower.starts_with(PROGRAM_FILES)
        || lower.starts_with(PROGRAM_FILES_X86)
    {
        return PathCategory::SystemProgram;
    }
    if let Ok(username) = env::var("USERNAME") {
        let user_path_prefix = format!("c:\\users\\{}", username.to_lowercase());
        if lower.contains(&user_path_prefix) {
            return PathCategory::UserProgram;
        }
    }
    if lower.contains("\\users\\") || USER_PATHS.iter().any(|p| lower.contains(p)) {
        return PathCategory::UserProgram;
    }
    if lower.starts_with(PROGRAM_DATA) {
        return PathCategory::ProgramData;
    }
    PathCategory::Ambiguous
}

pub fn expand_env_vars(path: &str) -> String {
    let mut result = path.to_string();
    while let Some(start) = result.find('%') {
        if let Some(end) = result[start + 1..].find('%') {
            let var_name = &result[start + 1..start + 1 + end];
            if let Ok(value) = env::var(var_name) {
                result = result.replace(&format!("%{}%", var_name), &value);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}

pub fn is_absolute_path(path: &str) -> bool {
    let trimmed = path.trim();
    trimmed.contains(':') || trimmed.starts_with('"') || trimmed.contains('%')
}

pub fn quote_if_needed(path: &str) -> String {
    if path.contains(' ') && !path.starts_with('"') {
        format!("\"{}\"", path)
    } else {
        path.to_string()
    }
}
