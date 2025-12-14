use anyhow::{Context, Result};
use colored::*;
use std::collections::{HashMap, HashSet};
use std::env;

use crate::analyzer::{PathCategory, PathEntry, PathLocation, SystemAnalyzer};
use crate::registry::RegistryHelper;

#[derive(Debug, Clone)]
pub struct MigrationAction {
    pub action_type: ActionType,
    pub path: String,
    pub from_location: PathLocation,
    #[allow(dead_code)]
    pub to_location: Option<PathLocation>,
    pub reason: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ActionType {
    RemoveDuplicate,
    MoveToUser,
    AddQuotes,
}

pub struct PathMigrator {
    backup_dir: std::path::PathBuf,
}

impl PathMigrator {
    pub fn new() -> Result<Self> {
        let local_app_data =
            env::var("LOCALAPPDATA").context("Failed to get LOCALAPPDATA environment variable")?;

        let backup_dir = std::path::PathBuf::from(local_app_data)
            .join("spath")
            .join("backups");

        std::fs::create_dir_all(&backup_dir).context("Failed to create backup directory")?;

        Ok(Self { backup_dir })
    }

    pub fn plan_migration(
        &self,
        remove_duplicates: bool,
        move_user_paths: bool,
    ) -> Result<MigrationPlan> {
        let analyzer = SystemAnalyzer::new()?;
        let analysis = analyzer.analyze()?;

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

        // Group paths by normalized path
        for entry in entries {
            let normalized = entry.path.trim_matches('"').to_lowercase();
            path_locations.entry(normalized).or_default().push(entry);
        }

        // Find duplicates
        for (_normalized_path, locations) in path_locations {
            if locations.len() <= 1 {
                continue;
            }

            // Determine which location to keep
            let has_system = locations
                .iter()
                .any(|e| matches!(e.location, PathLocation::System));
            let has_user = locations
                .iter()
                .any(|e| matches!(e.location, PathLocation::User));

            if !has_system || !has_user {
                // Not a duplicate between SYSTEM and USER
                continue;
            }

            // Check if it's a user path
            let is_user_path = locations
                .iter()
                .any(|e| matches!(e.category, PathCategory::UserProgram));

            for entry in locations {
                let should_remove = if is_user_path {
                    // User path: keep in USER, remove from SYSTEM
                    matches!(entry.location, PathLocation::System)
                } else {
                    // System path: keep in SYSTEM, remove from USER
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
                        from_location: entry.location.clone(),
                        to_location: None,
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
                    to_location: Some(PathLocation::User),
                    reason: "User-specific path should be in USER PATH".to_string(),
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

    pub fn execute_migration(&self, plan: &MigrationPlan, dry_run: bool) -> Result<()> {
        if dry_run {
            return Ok(());
        }

        if plan.requires_admin {
            println!(
                "{}",
                "This migration requires administrator rights!"
                    .yellow()
                    .bold()
            );
            println!(
                "{}",
                "  Some changes will be skipped if not running as admin.".yellow()
            );
            println!();
        }

        // Create backup
        self.create_backup()?;

        // Group actions by location
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
                    // Add with quotes if needed
                    let path_to_add = if action.path.contains(' ') && !action.path.starts_with('"')
                    {
                        format!("\"{}\"", action.path)
                    } else {
                        action.path.clone()
                    };
                    user_additions.push(path_to_add);
                }
                _ => {}
            }
        }

        // Apply changes to USER PATH (doesn't require admin)
        if !user_removals.is_empty() || !user_additions.is_empty() {
            self.update_user_path(&user_removals, &user_additions)?;
        }

        // Apply changes to SYSTEM PATH (requires admin)
        if !system_removals.is_empty() {
            match self.update_system_path(&system_removals) {
                Ok(_) => {
                    println!("{}", "SYSTEM PATH updated successfully".green().bold());
                }
                Err(e) => {
                    println!(
                        "{}",
                        "✗ Failed to update SYSTEM PATH (requires admin rights)"
                            .red()
                            .bold()
                    );
                    println!("  Error: {}", e);
                    println!();
                    println!("{}", "  USER PATH was updated successfully.".green());
                    println!(
                        "{}",
                        "  Run as administrator to update SYSTEM PATH.".yellow()
                    );
                }
            }
        }

        Ok(())
    }

    fn create_backup(&self) -> Result<()> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_file = self
            .backup_dir
            .join(format!("path_backup_{}.json", timestamp));

        let user_path = RegistryHelper::read_user_path_raw()?;
        let system_path = RegistryHelper::read_system_path_raw().ok();

        let backup = serde_json::json!({
            "timestamp": timestamp,
            "user_path": user_path,
            "system_path": system_path,
        });

        std::fs::write(&backup_file, serde_json::to_string_pretty(&backup)?)?;
        println!(
            "{} {}",
            "Backup created:".green().bold(),
            backup_file.display()
        );
        println!();

        Ok(())
    }

    fn update_user_path(&self, removals: &[String], additions: &[String]) -> Result<()> {
        let current_path = RegistryHelper::read_user_path_raw()?;
        let mut paths = RegistryHelper::parse_path_string(&current_path);

        // Remove paths
        let removals_normalized: HashSet<String> = removals
            .iter()
            .map(|p| p.trim_matches('"').to_lowercase())
            .collect();

        paths.retain(|p| {
            let normalized = p.trim_matches('"').to_lowercase();
            !removals_normalized.contains(&normalized)
        });

        // Add new paths
        paths.extend(additions.iter().cloned());

        let new_path = RegistryHelper::join_paths(&paths);
        RegistryHelper::write_user_path(&new_path)?;

        println!("{}", "USER PATH updated successfully".green().bold());

        Ok(())
    }

    fn update_system_path(&self, removals: &[String]) -> Result<()> {
        let current_path = RegistryHelper::read_system_path_raw()?;
        let mut paths = RegistryHelper::parse_path_string(&current_path);

        // Remove paths
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

pub struct MigrationPlan {
    pub actions: Vec<MigrationAction>,
    pub requires_admin: bool,
}
