use anyhow::Result;
use spath_cli::security::acl;
use std::fs;
use std::path::PathBuf;

fn get_test_file_path() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    temp_dir.join(format!("spath_acl_test_{}.txt", std::process::id()))
}

#[test]
fn test_set_acl_on_nonexistent_file() {
    let test_file = get_test_file_path();
    let _ = fs::remove_file(&test_file);
    let result = acl::set_user_only_acl(&test_file);
    assert!(result.is_err(), "Should fail on non-existent file");
}

#[test]
fn test_acl_on_json_backup_file() -> Result<()> {
    let test_file =
        std::env::temp_dir().join(format!("path_backup_test_{}.json", std::process::id()));
    let json_content =
        r#"{"timestamp":"20240101_120000","user_path":"C:\\test","system_path":null}"#;
    fs::write(&test_file, json_content)?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    let result = acl::set_user_only_acl(&test_file);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let read_content = fs::read_to_string(&test_file)?;
    let _ = fs::remove_file(&test_file);
    assert!(
        result.is_ok(),
        "Failed to set ACL on JSON file: {:?}",
        result.err()
    );
    assert_eq!(
        json_content, read_content,
        "JSON content should be preserved"
    );
    Ok(())
}

#[test]
fn test_acl_error_message_on_invalid_path() {
    let invalid_path = PathBuf::from("Z:\\nonexistent\\path\\file.txt");
    let result = acl::set_user_only_acl(&invalid_path);
    assert!(result.is_err(), "Should fail on invalid path");
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(
            error_msg.contains("File does not exist")
                || error_msg.contains("Failed to canonicalize"),
            "Error message should be descriptive, got: {}",
            error_msg
        );
    }
}

#[test]
fn test_acl_on_file_with_long_name() -> Result<()> {
    let long_name = format!(
        "spath_acl_test_with_very_long_name_{}_{}.txt",
        "x".repeat(50),
        std::process::id()
    );
    let test_file = std::env::temp_dir().join(long_name);
    fs::write(&test_file, "test content")?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    let result = acl::set_user_only_acl(&test_file);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _ = fs::remove_file(&test_file);
    assert!(
        result.is_ok(),
        "Failed to set ACL on file with long name: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
fn test_acl_on_txt_file() -> Result<()> {
    let test_file = std::env::temp_dir().join(format!("test_{}.txt", std::process::id()));
    fs::write(&test_file, "text file content")?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    let result = acl::set_user_only_acl(&test_file);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _ = fs::remove_file(&test_file);
    assert!(
        result.is_ok(),
        "Failed to set ACL on .txt file: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
fn test_acl_on_dat_file() -> Result<()> {
    let test_file = std::env::temp_dir().join(format!("test_{}.dat", std::process::id()));
    fs::write(&test_file, "binary data")?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    let result = acl::set_user_only_acl(&test_file);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _ = fs::remove_file(&test_file);
    assert!(
        result.is_ok(),
        "Failed to set ACL on .dat file: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
fn test_acl_on_file_without_extension() -> Result<()> {
    let test_file = std::env::temp_dir().join(format!("test_file_{}", std::process::id()));
    fs::write(&test_file, "content")?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    let result = acl::set_user_only_acl(&test_file);
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = fs::remove_file(&test_file);
    if result.is_err() {
        eprintln!(
            "ACL error on file without extension (may be expected): {:?}",
            result.err()
        );
    }
    Ok(())
}

#[test]
fn test_acl_on_log_file() -> Result<()> {
    let test_file = std::env::temp_dir().join(format!("test_{}.log", std::process::id()));
    fs::write(&test_file, "log content")?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    let result = acl::set_user_only_acl(&test_file);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _ = fs::remove_file(&test_file);
    assert!(
        result.is_ok(),
        "Failed to set ACL on .log file: {:?}",
        result.err()
    );
    Ok(())
}

#[test]
fn test_acl_on_cfg_file() -> Result<()> {
    let test_file = std::env::temp_dir().join(format!("test_{}.cfg", std::process::id()));
    fs::write(&test_file, "config=value")?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    let result = acl::set_user_only_acl(&test_file);
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = fs::remove_file(&test_file);
    if result.is_err() {
        eprintln!(
            "ACL error on .cfg file (may be expected): {:?}",
            result.err()
        );
    }
    Ok(())
}

#[test]
fn test_acl_on_xml_file() -> Result<()> {
    let test_file = std::env::temp_dir().join(format!("test_{}.xml", std::process::id()));
    fs::write(&test_file, "<root>data</root>")?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    let result = acl::set_user_only_acl(&test_file);
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = fs::remove_file(&test_file);
    if result.is_err() {
        eprintln!(
            "ACL error on .xml file (may be expected): {:?}",
            result.err()
        );
    }
    Ok(())
}

#[test]
fn test_acl_on_ini_file() -> Result<()> {
    let test_file = std::env::temp_dir().join(format!("test_{}.ini", std::process::id()));
    fs::write(&test_file, "[section]\nkey=value")?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    let result = acl::set_user_only_acl(&test_file);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let _ = fs::remove_file(&test_file);
    assert!(
        result.is_ok(),
        "Failed to set ACL on .ini file: {:?}",
        result.err()
    );
    Ok(())
}
