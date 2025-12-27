//! spath - Windows PATH security scanner and optimizer
//!
//! This library provides functionality to scan, analyze, and fix
//! Windows PATH environment variable security issues.

pub mod analyzer;
pub mod constants;
pub mod fixer;
pub mod formatter;
pub mod migrator;
pub mod registry;
pub mod scanner;
pub mod security;
