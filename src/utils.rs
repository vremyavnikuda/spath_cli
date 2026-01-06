//! Общие утилиты для работы с путями Windows.
use std::env;

/// Раскрывает переменные окружения Windows в строке пути.
pub fn expand_env_vars(path: &str) -> String {
    let mut result = path.to_string();
    while let Some(start) = result.find('%') {
        if let Some(end) = result[start + 1..].find('%') {
            let var_name = &result[start + 1..start + 1 + end];
            if let Ok(value) = env::var(var_name) {
                result = result.replace(&format!("%{}%", var_name), &value);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}

/// Проверяет, является ли путь абсолютным.
pub fn is_absolute_path(path: &str) -> bool {
    let trimmed = path.trim();
    trimmed.contains(':') || trimmed.starts_with('"') || trimmed.contains('%')
}
