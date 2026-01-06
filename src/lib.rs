//! spath - Windows PATH security scanner and optimizer
//!
//! This library provides functionality to scan, analyze, and fix
//! Windows PATH environment variable security issues.

pub mod analyzer;
pub mod backup;
pub mod constants;
pub mod fixer;
pub mod formatter;
pub mod migrator;
pub mod models;
pub mod registry;
pub mod scanner;
pub mod security;
pub mod utils;
pub mod visualizer;
