//! Унифицированные модели данных для spath-cli.
use crate::constants::{MAX_SINGLE_PATH_LENGTH, USER_PATHS};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathLocation {
    System,
    User,
}

impl std::fmt::Display for PathLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathLocation::System => write!(f, "SYSTEM"),
            PathLocation::User => write!(f, "USER"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathCategory {
    SystemProgram,
    UserProgram,
    ProgramData,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct PathIssue {
    pub path: String,
    pub level: IssueLevel,
    pub message: String,
}

impl PathIssue {
    pub fn critical(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            level: IssueLevel::Critical,
            message: message.into(),
        }
    }
    pub fn warning(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            level: IssueLevel::Warning,
            message: message.into(),
        }
    }
    pub fn info(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            level: IssueLevel::Info,
            message: message.into(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AuditStats {
    pub total_paths: usize,
    pub unquoted_with_spaces: usize,
    pub non_existent: usize,
    pub relative_paths: usize,
    pub properly_quoted: usize,
    pub valid_paths: usize,
}

#[derive(Debug, Clone)]
pub struct PathEntry {
    pub path: String,
    pub index: usize,
    pub location: PathLocation,
    pub category: PathCategory,
    pub exists: bool,
    pub has_spaces: bool,
    pub is_quoted: bool,
    pub is_duplicate: bool,
}

impl Default for PathEntry {
    fn default() -> Self {
        Self {
            path: String::new(),
            index: 0,
            location: PathLocation::User,
            category: PathCategory::Ambiguous,
            exists: false,
            has_spaces: false,
            is_quoted: false,
            is_duplicate: false,
        }
    }
}

impl PathEntry {
    pub fn new(path: String, index: usize, location: PathLocation, all_paths: &[String]) -> Self {
        let trimmed = path.trim_matches('"');
        let exists = Path::new(trimmed).exists();
        let has_spaces = trimmed.contains(' ');
        let is_quoted = path.starts_with('"') && path.ends_with('"');
        let category = Self::categorize(trimmed);
        let normalized = trimmed.to_lowercase();
        let is_duplicate = all_paths
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .any(|(_, p)| p.trim_matches('"').to_lowercase() == normalized);
        Self {
            path,
            index,
            location,
            category,
            exists,
            has_spaces,
            is_quoted,
            is_duplicate,
        }
    }
    pub fn categorize(path: &str) -> PathCategory {
        let lower = path.to_lowercase();
        if lower.starts_with("c:\\windows")
            || lower.starts_with("c:\\program files")
            || lower.starts_with("c:\\program files (x86)")
        {
            return PathCategory::SystemProgram;
        }
        if lower.contains("\\users\\") || USER_PATHS.iter().any(|p| lower.contains(p)) {
            return PathCategory::UserProgram;
        }
        if lower.starts_with("c:\\programdata") {
            return PathCategory::ProgramData;
        }
        PathCategory::Ambiguous
    }
    pub fn should_be_in_user_path(&self) -> bool {
        matches!(self.category, PathCategory::UserProgram)
            && matches!(self.location, PathLocation::System)
    }
    pub fn needs_quotes(&self) -> bool {
        self.has_spaces && !self.is_quoted
    }
    pub fn has_issues(&self) -> bool {
        !self.exists || self.needs_quotes() || self.is_duplicate
    }
    pub fn is_user_specific(&self) -> bool {
        matches!(self.category, PathCategory::UserProgram)
    }
    pub fn get_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if !self.exists {
            warnings.push("Path does not exist".to_string());
        }
        if self.needs_quotes() {
            warnings.push("Contains spaces but not quoted".to_string());
        }
        if self.is_duplicate {
            warnings.push("Duplicate path".to_string());
        }
        if self.path.len() > MAX_SINGLE_PATH_LENGTH {
            warnings.push(format!(
                "Path exceeds {} characters",
                MAX_SINGLE_PATH_LENGTH
            ));
        }
        warnings
    }
}
