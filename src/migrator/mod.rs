//! PATH migration for optimizing PATH structure.
use crate::analyzer::AnalysisResults;
use crate::backup::BackupManager;
use crate::models::{PathCategory, PathEntry, PathLocation};
use crate::registry::RegistryHelper;
use crate::utils::quote_if_needed;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MigrationAction {
    pub action_type: ActionType,
    pub path: String,
    pub from_location: PathLocation,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    RemoveDuplicate,
    MoveToUser,
    AddQuotes,
}

#[derive(Debug)]
pub struct MigrationResult {
    pub backup_path: PathBuf,
    pub user_path_updated: bool,
    pub system_path_updated: bool,
    pub system_path_error: Option<String>,
}

pub struct MigrationPlan {
    pub actions: Vec<MigrationAction>,
    pub requires_admin: bool,
}

pub struct PathMigrator {
    backup_manager: BackupManager,
}

impl PathMigrator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            backup_manager: BackupManager::new()?,
        })
    }
    pub fn plan_migration(
        &self,
        analysis: &AnalysisResults,
        remove_duplicates: bool,
        move_user_paths: bool,
    ) -> Result<MigrationPlan> {
        let mut actions = Vec::new();
        if remove_duplicates {
            actions.extend(self.plan_duplicate_removal(&analysis.entries)?);
        }
        if move_user_paths {
            actions.extend(self.plan_user_path_migration(&analysis.entries)?);
        }
        let requires_admin = move_user_paths || self.has_system_changes(&actions);
        Ok(MigrationPlan {
            actions,
            requires_admin,
        })
    }
    fn plan_duplicate_removal(&self, entries: &[PathEntry]) -> Result<Vec<MigrationAction>> {
        let mut actions = Vec::new();
        let mut path_locations: HashMap<String, Vec<&PathEntry>> = HashMap::new();
        for entry in entries {
            let normalized = entry.path.trim_matches('"').to_lowercase();
            path_locations.entry(normalized).or_default().push(entry);
        }
        for (_normalized_path, locations) in path_locations {
            if locations.len() <= 1 {
                continue;
            }
            let has_system = locations
                .iter()
                .any(|e| matches!(e.location, PathLocation::System));
            let has_user = locations
                .iter()
                .any(|e| matches!(e.location, PathLocation::User));
            if !has_system || !has_user {
                continue;
            }
            let is_user_path = locations
                .iter()
                .any(|e| matches!(e.category, PathCategory::UserProgram));
            for entry in locations {
                let should_remove = if is_user_path {
                    matches!(entry.location, PathLocation::System)
                } else {
                    matches!(entry.location, PathLocation::User)
                };
                if should_remove {
                    let keep_location = if is_user_path {
                        "USER PATH"
                    } else {
                        "SYSTEM PATH"
                    };
                    actions.push(MigrationAction {
                        action_type: ActionType::RemoveDuplicate,
                        path: entry.path.clone(),
                        from_location: entry.location,
                        reason: format!("Duplicate - already exists in {}", keep_location),
                    });
                }
            }
        }
        Ok(actions)
    }
    fn plan_user_path_migration(&self, entries: &[PathEntry]) -> Result<Vec<MigrationAction>> {
        let mut actions = Vec::new();
        for entry in entries {
            if matches!(entry.location, PathLocation::System)
                && matches!(entry.category, PathCategory::UserProgram)
            {
                actions.push(MigrationAction {
                    action_type: ActionType::MoveToUser,
                    path: entry.path.clone(),
                    from_location: PathLocation::System,
                    reason: "User-specific path should be in USER PATH".to_string(),
                });
            } else if entry.path.contains(' ') && !entry.path.starts_with('"') {
                actions.push(MigrationAction {
                    action_type: ActionType::AddQuotes,
                    path: entry.path.clone(),
                    from_location: entry.location,
                    reason: "Path contains spaces and should be quoted".to_string(),
                });
            }
        }
        Ok(actions)
    }
    fn has_system_changes(&self, actions: &[MigrationAction]) -> bool {
        actions
            .iter()
            .any(|a| matches!(a.from_location, PathLocation::System))
    }
    pub fn execute_migration(
        &self,
        plan: &MigrationPlan,
        dry_run: bool,
    ) -> Result<MigrationResult> {
        if dry_run {
            return Ok(MigrationResult {
                backup_path: PathBuf::new(),
                user_path_updated: false,
                system_path_updated: false,
                system_path_error: None,
            });
        }
        let backup_result = self.backup_manager.create()?;
        let (system_removals, user_removals, user_additions) = self.categorize_actions(plan);
        let user_path_updated = self.apply_user_changes(&user_removals, &user_additions)?;
        let (system_path_updated, system_path_error) = self.apply_system_changes(&system_removals);
        Ok(MigrationResult {
            backup_path: backup_result.path,
            user_path_updated,
            system_path_updated,
            system_path_error,
        })
    }
    fn categorize_actions(&self, plan: &MigrationPlan) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut system_removals = Vec::new();
        let mut user_removals = Vec::new();
        let mut user_additions = Vec::new();
        for action in &plan.actions {
            match (&action.action_type, &action.from_location) {
                (ActionType::RemoveDuplicate, PathLocation::System) => {
                    system_removals.push(action.path.clone());
                }
                (ActionType::RemoveDuplicate, PathLocation::User) => {
                    user_removals.push(action.path.clone());
                }
                (ActionType::MoveToUser, PathLocation::System) => {
                    system_removals.push(action.path.clone());
                    let path_to_add = quote_if_needed(&action.path);
                    user_additions.push(path_to_add);
                }
                (ActionType::AddQuotes, PathLocation::System) => {
                    system_removals.push(action.path.clone());
                    system_removals.push(format!("\"{}\"", action.path));
                }
                (ActionType::AddQuotes, PathLocation::User) => {
                    user_removals.push(action.path.clone());
                    user_additions.push(format!("\"{}\"", action.path));
                }
                _ => {}
            }
        }
        (system_removals, user_removals, user_additions)
    }
    fn apply_user_changes(&self, removals: &[String], additions: &[String]) -> Result<bool> {
        if removals.is_empty() && additions.is_empty() {
            return Ok(false);
        }
        let current_path = RegistryHelper::read_user_path_raw()?;
        let mut paths = RegistryHelper::parse_path_string(&current_path);
        let removals_normalized: HashSet<String> = removals
            .iter()
            .map(|p| p.trim_matches('"').to_lowercase())
            .collect();
        paths.retain(|p| {
            let normalized = p.trim_matches('"').to_lowercase();
            !removals_normalized.contains(&normalized)
        });
        paths.extend(additions.iter().cloned());
        let new_path = RegistryHelper::join_paths(&paths);
        RegistryHelper::write_user_path(&new_path)?;
        Ok(true)
    }
    fn apply_system_changes(&self, removals: &[String]) -> (bool, Option<String>) {
        if removals.is_empty() {
            return (false, None);
        }
        match self.update_system_path(removals) {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        }
    }
    fn update_system_path(&self, removals: &[String]) -> Result<()> {
        let current_path = RegistryHelper::read_system_path_raw()?;
        let mut paths = RegistryHelper::parse_path_string(&current_path);
        let removals_normalized: HashSet<String> = removals
            .iter()
            .map(|p| p.trim_matches('"').to_lowercase())
            .collect();
        paths.retain(|p| {
            let normalized = p.trim_matches('"').to_lowercase();
            !removals_normalized.contains(&normalized)
        });
        let new_path = RegistryHelper::join_paths(&paths);
        RegistryHelper::write_system_path(&new_path)?;
        Ok(())
    }
}
