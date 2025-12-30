use anyhow::{Context, Result};
use std::env;
use std::path::Path;

use crate::constants::{PROGRAM_DATA, PROGRAM_FILES, PROGRAM_FILES_X86, USER_PATHS, WINDOWS_PATH};
use crate::registry::RegistryHelper;

#[derive(Debug, Clone)]
pub enum PathLocation {
    System,
    User,
}

#[derive(Debug, Clone)]
pub enum PathCategory {
    SystemProgram,
    UserProgram,
    ProgramData,
    Ambiguous,
}

#[derive(Debug, Clone)]
pub struct PathEntry {
    pub path: String,
    pub location: PathLocation,
    pub category: PathCategory,
    pub has_spaces: bool,
    pub is_quoted: bool,
    pub exists: bool,
}

impl PathEntry {
    pub fn should_be_in_user_path(&self) -> bool {
        matches!(self.category, PathCategory::UserProgram)
            && matches!(self.location, PathLocation::System)
    }

    pub fn needs_quotes(&self) -> bool {
        self.has_spaces && !self.is_quoted
    }
}

pub struct SystemAnalyzer {
    current_username: String,
}

impl SystemAnalyzer {
    pub fn new() -> Result<Self> {
        let current_username = env::var("USERNAME").context("Failed to get current username")?;
        Ok(Self { current_username })
    }

    pub fn analyze(&self) -> Result<AnalysisResults> {
        let system_paths = self.read_system_path()?;
        let user_paths = self.read_user_path()?;
        let mut entries = Vec::new();
        for path in system_paths {
            let entry = self.analyze_path(&path, PathLocation::System);
            entries.push(entry);
        }
        for path in user_paths {
            let entry = self.analyze_path(&path, PathLocation::User);
            entries.push(entry);
        }
        Ok(AnalysisResults {
            entries,
            current_username: self.current_username.clone(),
        })
    }

    fn read_system_path(&self) -> Result<Vec<String>> {
        RegistryHelper::read_system_path()
    }

    fn read_user_path(&self) -> Result<Vec<String>> {
        RegistryHelper::read_user_path()
    }

    fn analyze_path(&self, path: &str, location: PathLocation) -> PathEntry {
        let trimmed = path.trim_matches('"');
        let has_spaces = path.contains(' ');
        let is_quoted = path.starts_with('"') && path.ends_with('"');
        let exists = Path::new(trimmed).exists();
        let category = self.categorize_path(trimmed);
        PathEntry {
            path: path.to_string(),
            location,
            category,
            has_spaces,
            is_quoted,
            exists,
        }
    }

    fn categorize_path(&self, path: &str) -> PathCategory {
        let lower = path.to_lowercase();
        if lower.starts_with(WINDOWS_PATH)
            || lower.starts_with(PROGRAM_FILES)
            || lower.starts_with(PROGRAM_FILES_X86)
        {
            return PathCategory::SystemProgram;
        }
        let user_path_prefix = format!("c:\\users\\{}", self.current_username.to_lowercase());
        if lower.contains(&user_path_prefix)
            || USER_PATHS.iter().any(|pattern| lower.contains(pattern))
        {
            return PathCategory::UserProgram;
        }
        if lower.starts_with(PROGRAM_DATA) {
            return PathCategory::ProgramData;
        }
        PathCategory::Ambiguous
    }
}

pub struct AnalysisResults {
    pub entries: Vec<PathEntry>,
    #[allow(dead_code)]
    pub current_username: String,
}
