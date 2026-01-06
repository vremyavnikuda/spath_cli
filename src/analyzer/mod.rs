//! System PATH analyzer.
use crate::constants::{PROGRAM_DATA, PROGRAM_FILES, PROGRAM_FILES_X86, USER_PATHS, WINDOWS_PATH};
use crate::models::{PathCategory, PathEntry, PathLocation};
use crate::registry::RegistryHelper;
use anyhow::{Context, Result};
use std::path::Path;

pub struct AnalysisResults {
    pub entries: Vec<PathEntry>,
}

pub struct SystemAnalyzer {
    current_username: String,
}

impl SystemAnalyzer {
    pub fn new() -> Result<Self> {
        let current_username =
            std::env::var("USERNAME").context("Failed to get current username")?;
        Ok(Self { current_username })
    }
    pub fn analyze(&self) -> Result<AnalysisResults> {
        let system_paths = RegistryHelper::read_system_path()?;
        let user_paths = RegistryHelper::read_user_path()?;
        let all_paths: Vec<String> = system_paths
            .iter()
            .chain(user_paths.iter())
            .cloned()
            .collect();
        let mut entries = Vec::new();
        let mut index = 0;
        for path in &system_paths {
            entries.push(self.analyze_path(path, index, PathLocation::System, &all_paths));
            index += 1;
        }
        for path in &user_paths {
            entries.push(self.analyze_path(path, index, PathLocation::User, &all_paths));
            index += 1;
        }
        Ok(AnalysisResults { entries })
    }
    fn analyze_path(
        &self,
        path: &str,
        index: usize,
        location: PathLocation,
        all_paths: &[String],
    ) -> PathEntry {
        let trimmed = path.trim_matches('"');
        let has_spaces = path.contains(' ');
        let is_quoted = path.starts_with('"') && path.ends_with('"');
        let exists = Path::new(trimmed).exists();
        let category = self.categorize_path(trimmed);
        let normalized = trimmed.to_lowercase();
        let is_duplicate = all_paths
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .any(|(_, p)| p.trim_matches('"').to_lowercase() == normalized);
        PathEntry {
            path: path.to_string(),
            index,
            location,
            category,
            exists,
            has_spaces,
            is_quoted,
            is_duplicate,
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
