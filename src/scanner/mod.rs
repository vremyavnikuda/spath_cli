use anyhow::{Context, Result};
use std::env;
use std::path::Path;

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
    pub fn new() -> Result<Self> {
        let path_var = env::var("PATH").context("Failed to read PATH environment variable")?;

        Ok(Self { path_var })
    }

    pub fn scan(&self) -> Result<ScanResults> {
        let paths: Vec<String> = self
            .path_var
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let mut issues = Vec::new();
        let mut audit = AuditStats {
            total_paths: paths.len(),
            ..Default::default()
        };

        for path in &paths {
            let has_spaces = path.contains(' ');
            let is_quoted = path.starts_with('"');
            let exists = Path::new(path.trim_matches('"')).exists();
            let is_absolute = path.contains(':') || path.starts_with('"');

            // Track statistics
            if has_spaces && !is_quoted {
                audit.unquoted_with_spaces += 1;
            }

            if !exists {
                audit.non_existent += 1;
            }

            if !is_absolute && !path.is_empty() {
                audit.relative_paths += 1;
            }

            if has_spaces && is_quoted {
                audit.properly_quoted += 1;
            }

            // Valid path: exists, absolute, and if has spaces then quoted
            if exists && is_absolute && (!has_spaces || is_quoted) {
                audit.valid_paths += 1;
            }

            // Check for spaces without quotes
            if has_spaces && !is_quoted {
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Critical,
                    message: "Path contains spaces but is not quoted. This can be exploited!"
                        .to_string(),
                });
            } else if has_spaces && is_quoted && exists {
                // Properly quoted path with spaces - good!
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Info,
                    message: "Path is properly quoted".to_string(),
                });
            }

            // Check if path exists
            if !exists {
                issues.push(PathIssue {
                    path: path.clone(),
                    level: IssueLevel::Warning,
                    message: "Path does not exist".to_string(),
                });
            }

            // Check for relative paths
            if !is_absolute && !path.is_empty() {
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
