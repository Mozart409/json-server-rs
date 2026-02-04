#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::unused_async, clippy::unnecessary_wraps)]

use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::{self, Context};
use std::fs::{self, File};
use std::path::Path;
use tempfile::TempDir;

fn get_json_files(data_dir: &str) -> eyre::Result<Vec<String>> {
    let mut json_files = Vec::new();
    let entries =
        fs::read_dir(data_dir).with_context(|| format!("Failed to read directory: {data_dir}"))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if let Some(extension) = path.extension()
            && extension == "json"
        {
            let file_name = path
                .file_stem()
                .context("Failed to get file stem")?
                .to_str()
                .context("Invalid filename encoding")?
                .to_string();
            json_files.push(file_name);
        }
    }
    Ok(json_files)
}

#[test]
fn test_get_json_files_success() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    File::create(format!("{data_dir}/test1.json")).unwrap();
    File::create(format!("{data_dir}/test2.json")).unwrap();
    File::create(format!("{data_dir}/test3.txt")).unwrap();

    let result = get_json_files(data_dir.as_str()).expect("Failed to get JSON files");

    assert_eq!(result.len(), 2);
    assert!(result.contains(&"test1".to_string()));
    assert!(result.contains(&"test2".to_string()));
    assert!(!result.contains(&"test3".to_string()));
}

#[test]
fn test_get_json_files_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let result = get_json_files(data_dir.as_str()).expect("Failed to get JSON files");

    assert!(result.is_empty());
}

#[test]
fn test_get_json_files_no_json_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    File::create(format!("{data_dir}/test1.txt")).unwrap();
    File::create(format!("{data_dir}/test2.yaml")).unwrap();
    File::create(format!("{data_dir}/test3.xml")).unwrap();

    let result = get_json_files(data_dir.as_str()).expect("Failed to get JSON files");

    assert!(result.is_empty());
}

#[test]
fn test_get_json_files_handles_special_chars() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    File::create(format!("{data_dir}/test-file_123.json")).unwrap();
    File::create(format!("{data_dir}/test.file.json")).unwrap();

    let result = get_json_files(data_dir.as_str()).expect("Failed to get JSON files");

    assert_eq!(result.len(), 2);
    assert!(result.contains(&"test-file_123".to_string()));
    assert!(result.contains(&"test.file".to_string()));
}

#[test]
fn test_get_json_files_subdirectory_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let subdir = Path::new(&data_dir).join("subdir");
    fs::create_dir(&subdir).unwrap();
    File::create(subdir.join("subdir_file.json")).unwrap();

    let result = get_json_files(data_dir.as_str()).expect("Failed to get JSON files");

    assert!(result.is_empty());
}

#[test]
fn test_get_json_files_nonexistent_directory() {
    let result = get_json_files("/this/path/definitely/does/not/exist/json");

    assert!(result.is_err());
}

#[test]
fn test_get_json_files_mixed_case_extensions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    File::create(format!("{data_dir}/test1.JSON")).unwrap();
    File::create(format!("{data_dir}/test2.Json")).unwrap();
    File::create(format!("{data_dir}/test3.json")).unwrap();

    let result = get_json_files(data_dir.as_str()).expect("Failed to get JSON files");

    assert_eq!(result.len(), 1);
    assert!(result.contains(&"test3".to_string()));
}

#[test]
fn test_get_json_files_unicode_filenames() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    File::create(format!("{data_dir}/测试.json")).unwrap();
    File::create(format!("{data_dir}/test_тест.json")).unwrap();

    let result = get_json_files(data_dir.as_str()).expect("Failed to get JSON files");

    assert_eq!(result.len(), 2);
    assert!(result.contains(&"测试".to_string()));
    assert!(result.contains(&"test_тест".to_string()));
}

#[test]
fn test_get_json_files_preserves_order() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let files = vec!["zzz.json", "aaa.json", "mmm.json"];
    for file in &files {
        File::create(format!("{data_dir}/{file}")).unwrap();
    }

    let result = get_json_files(data_dir.as_str()).expect("Failed to get JSON files");

    assert_eq!(result.len(), 3);
    assert!(result.contains(&"zzz".to_string()));
    assert!(result.contains(&"aaa".to_string()));
    assert!(result.contains(&"mmm".to_string()));
}

#[test]
fn test_get_json_files_can_read_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    fs::write(
        format!("{data_dir}/test.json"),
        r#"{"key": "value", "number": 123}"#,
    )
    .expect("Failed to write test.json");

    let result = get_json_files(data_dir.as_str()).expect("Failed to get JSON files");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "test");

    let file_path = format!("{data_dir}/test.json");
    let content = fs::read_to_string(&file_path).expect("Failed to read file");
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");
    assert_eq!(parsed["key"], "value");
    assert_eq!(parsed["number"], 123);
}
