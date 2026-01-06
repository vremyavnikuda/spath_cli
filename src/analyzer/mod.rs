//! System PATH analyzer.
use crate::models::{PathEntry, PathLocation};
use crate::registry::RegistryHelper;
use anyhow::Result;

pub struct AnalysisResults {
    pub entries: Vec<PathEntry>,
}

pub struct SystemAnalyzer;

impl SystemAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self)
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
            entries.push(PathEntry::new(
                path.clone(),
                index,
                PathLocation::System,
                &all_paths,
            ));
            index += 1;
        }
        for path in &user_paths {
            entries.push(PathEntry::new(
                path.clone(),
                index,
                PathLocation::User,
                &all_paths,
            ));
            index += 1;
        }
        Ok(AnalysisResults { entries })
    }
}
