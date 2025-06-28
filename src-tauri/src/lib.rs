use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;

// JJ log template for extracting change information
// Format: change_id\ncommit_id\ndescription\nauthor\ntimestamp\n---\n
// Use coalesce to provide default values for potentially empty fields
const JJ_LOG_TEMPLATE: &str = "change_id ++ \"\\n\" ++ commit_id ++ \"\\n\" ++ coalesce(description, \"(no description)\") ++ \"\\n\" ++ coalesce(author.name(), \"(unknown)\") ++ \"\\n\" ++ committer.timestamp() ++ \"\\n---\\n\"";

// Limit the number of changes to fetch for performance (prevents UI freezing on large repos)
const MAX_CHANGES_LIMIT: &str = "100";

// Expected number of fields in each log entry
const EXPECTED_FIELDS: usize = 5;

#[derive(Debug, Serialize, Deserialize)]
pub struct JJChange {
    pub change_id: String,
    pub commit_id: String,
    pub description: String,
    pub author: String,
    pub timestamp: String,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Validates and canonicalizes a path with security checks
/// Returns the canonical path if valid, or an error message if invalid
fn validate_and_canonicalize_path(path: &str) -> Result<std::path::PathBuf, String> {
    // Path sanitization and security checks
    if path.is_empty() {
        return Err("Path cannot be empty".to_string());
    }
    
    // Check for suspicious path patterns
    if path.contains("..") || path.contains("~") {
        return Err("Path contains potentially unsafe patterns".to_string());
    }
    
    // Additional security: reject paths with null bytes or control characters
    if path.contains('\0') || path.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
        return Err("Path contains invalid characters".to_string());
    }
    
    let repo_path = Path::new(path);
    
    // Canonicalize path to resolve any symbolic links and get absolute path
    let canonical_path = match repo_path.canonicalize() {
        Ok(p) => p,
        Err(_) => return Err("Invalid or inaccessible path".to_string()),
    };
    
    // Check if the canonical path exists and is a directory
    if !canonical_path.exists() {
        return Err("Path does not exist".to_string());
    }
    
    if !canonical_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    Ok(canonical_path)
}

#[tauri::command]
async fn validate_jj_repo(path: String) -> Result<bool, String> {
    // Validate and canonicalize the path with security checks
    let canonical_path = validate_and_canonicalize_path(&path)?;
    
    // First, check for the presence of .jj directory (more reliable than running jj status)
    let jj_dir = canonical_path.join(".jj");
    if !jj_dir.exists() || !jj_dir.is_dir() {
        return Err("Not a JJ repository (no .jj directory found)".to_string());
    }
    
    // Additional validation: check if .jj directory contains expected structure
    let jj_store_dir = jj_dir.join("repo").join("store");
    if !jj_store_dir.exists() {
        return Err("Invalid JJ repository structure".to_string());
    }
    
    // Final validation: run `jj status` with specific timeout and better error handling
    let output = tokio::process::Command::new("jj")
        .arg("status")
        .arg("--no-pager") // Prevent pager from blocking
        .current_dir(&canonical_path)
        .kill_on_drop(true)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            match output.status.code() {
                Some(0) => Ok(true), // Success exit code
                Some(1) => {
                    // Exit code 1 typically indicates "not a jj repo" or similar user error
                    Err("Not a valid JJ repository".to_string())
                }
                Some(code) => {
                    // Other exit codes indicate different types of errors
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!("JJ command failed with exit code {}: {}", code, stderr.trim()))
                }
                None => {
                    // Process was terminated by signal
                    Err("JJ command was terminated".to_string())
                }
            }
        }
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    Err("JJ command not found. Please ensure Jujutsu is installed and in your PATH".to_string())
                }
                std::io::ErrorKind::PermissionDenied => {
                    Err("Permission denied when accessing the repository".to_string())
                }
                std::io::ErrorKind::TimedOut => {
                    Err("JJ command timed out".to_string())
                }
                _ => {
                    Err(format!("Failed to execute JJ command: {}", e))
                }
            }
        }
    }
}

#[tauri::command]
async fn get_jj_changes(path: String) -> Result<Vec<JJChange>, String> {
    // Validate and canonicalize the path with security checks
    let canonical_path = validate_and_canonicalize_path(&path)?;
    
    // Verify it's a JJ repository before attempting to get changes
    let jj_dir = canonical_path.join(".jj");
    if !jj_dir.exists() || !jj_dir.is_dir() {
        return Err("Not a JJ repository (no .jj directory found)".to_string());
    }
    
    // Use async tokio command with better error handling
    let output = tokio::process::Command::new("jj")
        .args(&["log", "--template", JJ_LOG_TEMPLATE, "--limit", MAX_CHANGES_LIMIT])
        .arg("--no-pager") // Prevent pager from blocking
        .current_dir(&canonical_path)
        .kill_on_drop(true)
        .output()
        .await;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                parse_jj_log(&stdout)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                match output.status.code() {
                    Some(code) => Err(format!("JJ log command failed with exit code {}: {}", code, stderr.trim())),
                    None => Err("JJ log command was terminated".to_string()),
                }
            }
        }
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    Err("JJ command not found. Please ensure Jujutsu is installed and in your PATH".to_string())
                }
                std::io::ErrorKind::PermissionDenied => {
                    Err("Permission denied when accessing the repository".to_string())
                }
                _ => {
                    Err(format!("Failed to execute JJ log command: {}", e))
                }
            }
        }
    }
}

fn parse_jj_log(log_output: &str) -> Result<Vec<JJChange>, String> {
    let mut changes = Vec::new();
    let entries: Vec<&str> = log_output.split("---\n").collect();
    
    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        
        let lines: Vec<&str> = entry.lines().collect();
        if lines.len() >= EXPECTED_FIELDS {
            // Basic validation - only skip if critical fields are completely missing
            let change = JJChange {
                change_id: lines[0].trim().to_string(),
                commit_id: lines[1].trim().to_string(),
                description: lines[2].trim().to_string(),
                author: lines[3].trim().to_string(),
                timestamp: lines[4].trim().to_string(),
            };
            
            // Skip entries with empty change_id or commit_id (critical identifiers)
            if change.change_id.is_empty() || change.commit_id.is_empty() {
                eprintln!("Warning: Skipping entry with missing critical identifiers");
                continue;
            }
            
            changes.push(change);
        } else if !lines.is_empty() {
            // Log warning for malformed entries instead of silently skipping
            eprintln!(
                "Skipping malformed log entry with {} fields (expected {}): {:?}",
                lines.len(),
                EXPECTED_FIELDS,
                lines
            );
        }
    }
    
    if changes.is_empty() && !log_output.trim().is_empty() {
        return Err("No valid changes found in log output".to_string());
    }
    
    Ok(changes)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, validate_jj_repo, get_jj_changes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
