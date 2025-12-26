use anyhow::{Context, Result};
use std::collections::HashSet;
use std::env;
use std::path::Path;

use crate::constants::{PROGRAM_FILES, PROGRAM_FILES_X86, WINDOWS_PATH};
use crate::registry::RegistryHelper;

/// Expands environment variables in a path string.
///
/// Supports Windows-style `%VAR%` syntax.
fn expand_env_vars(path: &str) -> String {
    let mut result = path.to_string();
    while let Some(start) = result.find('%') {
        if let Some(end) = result[start + 1..].find('%') {
            let var_name = &result[start + 1..start + 1 + end];
            if let Ok(value) = env::var(var_name) {
                let pattern = format!("%{}%", var_name);
                result = result.replace(&pattern, &value);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}

/// Checks if an unquoted path with spaces could be exploited.
///
/// Returns true if the path could be vulnerable to DLL hijacking or similar attacks.
///
/// For example, `"C:\Program Files\App\bin"` could be exploited by creating:
/// - `C:\Program.exe` (would be executed instead of `C:\Program Files\...`)
/// - `C:\Program Files\App.exe` (would be executed instead of `C:\Program Files\App\...`)
fn check_path_exploitable(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.starts_with(PROGRAM_FILES)
        || path_lower.starts_with(PROGRAM_FILES_X86)
        || path_lower.starts_with(WINDOWS_PATH)
}

#[derive(Debug, Clone)]
pub enum IssueLevel {
    Critical,
    Warning,
    Info,
}
#[derive(Debug, Clone)]
pub struct PathIssue {
    pub path: String,
    pub level: IssueLevel,
    pub message: String,
}

pub struct ScanResults {
    pub paths: Vec<String>,
    pub issues: Vec<PathIssue>,
    pub audit: AuditStats,
}

#[derive(Debug, Default)]
pub struct AuditStats {
    pub total_paths: usize,
    pub unquoted_with_spaces: usize,
    pub non_existent: usize,
    pub relative_paths: usize,
    pub properly_quoted: usize,
    pub valid_paths: usize,
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
        let paths = RegistryHelper::parse_path_string(&self.path_var);
        let mut issues = Vec::new();
        let mut audit = AuditStats {
            total_paths: paths.len(),
            ..Default::default()
        };
        let mut seen = HashSet::new();
        for path in &paths {
            let trimmed = path.trim();
            let has_spaces = trimmed.contains(' ');
            let is_quoted = trimmed.starts_with('"');
            let path_to_check = if trimmed.contains('%') {
                expand_env_vars(trimmed)
            } else {
                trimmed.trim_matches('"').to_string()
            };
            let exists = Path::new(&path_to_check).exists();
            let is_absolute =
                trimmed.contains(':') || trimmed.starts_with('"') || trimmed.contains('%');
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
            if seen.contains(trimmed) {
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Warning,
                    message: "Duplicate path entry".to_string(),
                });
            }
            seen.insert(trimmed.to_string());
            if has_spaces && !is_quoted {
                if exists {
                    let is_exploitable = check_path_exploitable(trimmed);
                    if is_exploitable {
                        issues.push(PathIssue {
                            path: path.clone(),
                            level: IssueLevel::Critical,
                            message: "Path contains spaces without quotes and could be exploited by creating malicious files/directories".to_string(),
                        });
                    } else {
                        issues.push(PathIssue {
                            path: path.clone(),
                            level: IssueLevel::Info,
                            message: "Path contains spaces but is not quoted. Consider adding quotes for better compatibility.".to_string(),
                        });
                    }
                } else {
                    issues.push(PathIssue {
                        path: path.clone(),
                        level: IssueLevel::Warning,
                        message: "Path contains spaces, is not quoted, and does not exist"
                            .to_string(),
                    });
                }
            } else if has_spaces && is_quoted && exists {
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Info,
                    message: "Path is properly quoted".to_string(),
                });
            }
            if !exists {
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Warning,
                    message: "Path does not exist".to_string(),
                });
            }
            if !is_absolute && !trimmed.is_empty() {
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Warning,
                    message: "Relative path detected - should use absolute paths".to_string(),
                });
            }
        }
        Ok(ScanResults {
            paths,
            issues,
            audit,
        })
    }
}
