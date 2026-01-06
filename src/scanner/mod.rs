//! PATH scanner for security issues.
use crate::constants::{PROGRAM_FILES, PROGRAM_FILES_X86, WINDOWS_PATH};
use crate::models::{AuditStats, IssueLevel, PathIssue};
use crate::registry::RegistryHelper;
use crate::utils::{expand_env_vars, is_absolute_path};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;
use tracing::{debug, info, warn};

fn check_path_exploitable(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.starts_with(PROGRAM_FILES)
        || path_lower.starts_with(PROGRAM_FILES_X86)
        || path_lower.starts_with(WINDOWS_PATH)
}

pub struct ScanResults {
    pub paths: Vec<String>,
    pub issues: Vec<PathIssue>,
    pub audit: AuditStats,
}

pub struct PathScanner {
    path_var: String,
}

impl PathScanner {
    pub fn new(scan_system: bool) -> Result<Self> {
        let path_var = if scan_system {
            RegistryHelper::read_system_path_raw()
                .context("Failed to read SYSTEM PATH from registry")?
        } else {
            RegistryHelper::read_user_path_raw()
                .context("Failed to read USER PATH from registry")?
        };
        Ok(Self { path_var })
    }
    pub fn scan(&self) -> Result<ScanResults> {
        info!("Starting PATH scan");
        let paths = RegistryHelper::parse_path_string(&self.path_var);
        debug!("Found {} path entries to scan", paths.len());
        let mut issues = Vec::new();
        let mut audit = AuditStats {
            total_paths: paths.len(),
            ..Default::default()
        };
        let mut seen = HashSet::new();
        for path in &paths {
            self.scan_single_path(path, &mut issues, &mut audit, &mut seen);
        }
        info!(
            "Scan completed: {} issues found, {} critical",
            issues.len(),
            issues
                .iter()
                .filter(|i| matches!(i.level, IssueLevel::Critical))
                .count()
        );
        Ok(ScanResults {
            paths,
            issues,
            audit,
        })
    }
    fn scan_single_path(
        &self,
        path: &str,
        issues: &mut Vec<PathIssue>,
        audit: &mut AuditStats,
        seen: &mut HashSet<String>,
    ) {
        let trimmed = path.trim();
        let has_spaces = trimmed.contains(' ');
        let is_quoted = trimmed.starts_with('"');
        let path_to_check = self.resolve_path(trimmed);
        let exists = Path::new(&path_to_check).exists();
        let is_absolute = is_absolute_path(trimmed);
        self.update_audit_stats(audit, has_spaces, is_quoted, exists, is_absolute, trimmed);
        self.check_duplicate(path, trimmed, issues, seen);
        self.check_unquoted_spaces(path, trimmed, has_spaces, is_quoted, exists, issues);
        self.check_existence(path, exists, issues);
        self.check_relative_path(path, is_absolute, trimmed, issues);
    }
    fn resolve_path(&self, trimmed: &str) -> String {
        if trimmed.contains('%') {
            expand_env_vars(trimmed)
        } else {
            trimmed.trim_matches('"').to_string()
        }
    }
    fn update_audit_stats(
        &self,
        audit: &mut AuditStats,
        has_spaces: bool,
        is_quoted: bool,
        exists: bool,
        is_absolute: bool,
        trimmed: &str,
    ) {
        if has_spaces && !is_quoted {
            audit.unquoted_with_spaces += 1;
        }
        if !exists {
            audit.non_existent += 1;
        }
        if !is_absolute && !trimmed.is_empty() {
            audit.relative_paths += 1;
        }
        if has_spaces && is_quoted {
            audit.properly_quoted += 1;
        }
        if exists && is_absolute && (!has_spaces || is_quoted) {
            audit.valid_paths += 1;
        }
    }
    fn check_duplicate(
        &self,
        path: &str,
        trimmed: &str,
        issues: &mut Vec<PathIssue>,
        seen: &mut HashSet<String>,
    ) {
        if seen.contains(trimmed) {
            issues.push(PathIssue::warning(path, "Duplicate path entry"));
        }
        seen.insert(trimmed.to_string());
    }
    fn check_unquoted_spaces(
        &self,
        path: &str,
        trimmed: &str,
        has_spaces: bool,
        is_quoted: bool,
        exists: bool,
        issues: &mut Vec<PathIssue>,
    ) {
        if !has_spaces || is_quoted {
            if has_spaces && is_quoted && exists {
                issues.push(PathIssue::info(path, "Path is properly quoted"));
            }
            return;
        }
        if exists {
            if check_path_exploitable(trimmed) {
                warn!("Critical security issue found: {}", trimmed);
                issues.push(PathIssue::critical(path, "Path contains spaces without quotes and could be exploited by creating malicious files/directories"));
            } else {
                issues.push(PathIssue::info(path, "Path contains spaces but is not quoted. Consider adding quotes for better compatibility."));
            }
        } else {
            issues.push(PathIssue::warning(
                path,
                "Path contains spaces, is not quoted, and does not exist",
            ));
        }
    }
    fn check_existence(&self, path: &str, exists: bool, issues: &mut Vec<PathIssue>) {
        if !exists {
            issues.push(PathIssue::warning(path, "Path does not exist"));
        }
    }
    fn check_relative_path(
        &self,
        path: &str,
        is_absolute: bool,
        trimmed: &str,
        issues: &mut Vec<PathIssue>,
    ) {
        if !is_absolute && !trimmed.is_empty() {
            issues.push(PathIssue::warning(
                path,
                "Relative path detected - should use absolute paths",
            ));
        }
    }
}
