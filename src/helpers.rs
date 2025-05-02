use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::time::UNIX_EPOCH;
use std::os::unix::fs::PermissionsExt;
use chrono;
use filetime::FileTime;
use colored::*;

use crate::errors::CrateResult;

pub fn ls() -> CrateResult<()> {
    let entries = fs::read_dir(".")?;

    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string(); // Convert to an owned String
        
        // Colorize output based on the type
        if metadata.is_dir() {
            println!("{}/", name.blue().bold());
        } else if metadata.permissions().mode() & 0o111 != 0 {
            // Executable file
            println!("{}", name.green());
        } else if name.ends_with(".rs") || name.ends_with(".toml") || 
                  name.ends_with(".json") || name.ends_with(".md") {
            // Source code and documentation files
            println!("{}", name.yellow());
        } else {
            println!("{}", name);
        }
    }

    Ok(())
}

pub fn ls_detailed() -> CrateResult<()> {
    let entries = fs::read_dir(".")?;
    
    println!("{} {} {} {} {}", 
        "Type ".bright_cyan().bold(),
        "Permissions".bright_cyan().bold(),
        "Size      ".bright_cyan().bold(),
        "Modified            ".bright_cyan().bold(),
        "Name".bright_cyan().bold());
    println!("{}", "─".repeat(80).bright_black());

    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string(); // Convert to an owned String
        
        // Format the file type with appropriate color
        let file_type = if metadata.is_dir() { 
            "DIR ".blue().bold() 
        } else if metadata.is_file() { 
            "FILE".normal() 
        } else { 
            "LINK".purple().bold() 
        };
        
        // Format permissions
        let mode = metadata.permissions().mode();
        let permissions = format!(
            "{}{}{}{}{}{}{}{}{}",
            if mode & 0o400 != 0 { "r".green() } else { "-".normal() },
            if mode & 0o200 != 0 { "w".green() } else { "-".normal() },
            if mode & 0o100 != 0 { "x".green() } else { "-".normal() },
            if mode & 0o040 != 0 { "r".yellow() } else { "-".normal() },
            if mode & 0o020 != 0 { "w".yellow() } else { "-".normal() },
            if mode & 0o010 != 0 { "x".yellow() } else { "-".normal() },
            if mode & 0o004 != 0 { "r".red() } else { "-".normal() },
            if mode & 0o002 != 0 { "w".red() } else { "-".normal() },
            if mode & 0o001 != 0 { "x".red() } else { "-".normal() },
        );
        
        // Format size with units
        let size = metadata.len();
        let size_str = if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        };
        
        let modified = metadata.modified()?;
        let modified_since_epoch = modified.duration_since(UNIX_EPOCH)?.as_secs();
        let modified_time = chrono::DateTime::<chrono::Utc>::from_timestamp(modified_since_epoch as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // Format name with color based on type
        let colored_name = if metadata.is_dir() {
            name.blue().bold()
        } else if metadata.permissions().mode() & 0o111 != 0 {
            // Executable file
            name.green()
        } else if name.ends_with(".rs") || name.ends_with(".toml") || 
                  name.ends_with(".json") || name.ends_with(".md") {
            // Source code files
            name.yellow()
        } else {
            name.normal()
        };
        
        println!("{:4} {:9} {:10} {:20} {}", 
            file_type, 
            permissions, 
            size_str.cyan(), 
            modified_time.bright_black(),
            colored_name);
    }

    Ok(())
}

pub fn pwd() -> CrateResult<String> {
    let current_dir = std::env::current_dir()?;

    Ok(current_dir.display().to_string())
}

pub fn cd(path: &str) -> CrateResult<()> {
    std::env::set_current_dir(path)?;

    Ok(())
}

pub fn touch(path: &str) -> CrateResult<()> {
    // Check if file exists
    if Path::new(path).exists() {
        // Update the access and modification times
        let now = FileTime::now();
        filetime::set_file_times(path, now, now)?;
    } else {
        // Create the file if it doesn't exist
        fs::File::create(path)?;
    }

    Ok(())
}

pub fn rm(path: &str) -> CrateResult<()> {
    fs::remove_file(path)?;

    Ok(())
}

pub fn mkdir(path: &str) -> CrateResult<()> {
    fs::create_dir(path)?;
    
    Ok(())
}

pub fn mkdir_p(path: &str) -> CrateResult<()> {
    fs::create_dir_all(path)?;
    
    Ok(())
}

pub fn rmdir(path: &str) -> CrateResult<()> {
    fs::remove_dir(path)?;
    
    Ok(())
}

pub fn rmdir_r(path: &str) -> CrateResult<()> {
    fs::remove_dir_all(path)?;
    
    Ok(())
}

pub fn cp(source: &str, destination: &str) -> CrateResult<()> {
    // Check if the source is a directory
    if Path::new(source).is_dir() {
        return Err(anyhow::anyhow!("Source is a directory. Use cp_r for recursive copy."));
    }
    
    fs::copy(source, destination)?;
    
    Ok(())
}

pub fn cp_r(source: &str, destination: &str) -> CrateResult<()> {
    copy_dir_recursive(source, destination)?;
    
    Ok(())
}

fn copy_dir_recursive(source: &str, destination: &str) -> CrateResult<()> {
    let src_path = Path::new(source);
    let dst_path = Path::new(destination);
    
    if !src_path.exists() {
        return Err(anyhow::anyhow!("Source path doesn't exist"));
    }
    
    if !src_path.is_dir() {
        // Simple file copy
        fs::copy(source, destination)?;
        return Ok(());
    }
    
    // Create destination directory if it doesn't exist
    if !dst_path.exists() {
        fs::create_dir_all(destination)?;
    }
    
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let src_path = entry.path();
        let dst_path = Path::new(destination).join(&file_name);
        
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(
                src_path.to_str().unwrap(),
                dst_path.to_str().unwrap()
            )?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

pub fn mv(source: &str, destination: &str) -> CrateResult<()> {
    fs::rename(source, destination)?;
    
    Ok(())
}

pub fn cat(path: &str) -> CrateResult<String> {
    let pwd = pwd()?;

    let joined_path = std::path::Path::new(&pwd).join(path);
    let contents = fs::read_to_string(joined_path)?;

    Ok(contents)
}

pub fn stat(path: &str) -> CrateResult<String> {
    let metadata = fs::metadata(path)?;
    let mut result = String::new();
    
    result.push_str(&format!("File: {}\n", path));
    result.push_str(&format!("Size: {} bytes\n", metadata.len()));
    result.push_str(&format!("Type: {}\n", 
        if metadata.is_file() { "Regular File" } 
        else if metadata.is_dir() { "Directory" }
        else { "Special File" }));
    
    result.push_str(&format!("Permissions: {:o}\n", metadata.permissions().mode() & 0o777));
    
    if let Ok(created) = metadata.created() {
        if let Ok(time) = created.duration_since(UNIX_EPOCH) {
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(time.as_secs() as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            result.push_str(&format!("Created: {}\n", dt));
        }
    }
    
    if let Ok(modified) = metadata.modified() {
        if let Ok(time) = modified.duration_since(UNIX_EPOCH) {
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(time.as_secs() as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            result.push_str(&format!("Modified: {}\n", dt));
        }
    }
    
    if let Ok(accessed) = metadata.accessed() {
        if let Ok(time) = accessed.duration_since(UNIX_EPOCH) {
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(time.as_secs() as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            result.push_str(&format!("Accessed: {}\n", dt));
        }
    }
    
    Ok(result)
}

pub fn find(dir: &str, pattern: &str) -> CrateResult<Vec<PathBuf>> {
    let mut results = Vec::new();
    find_recursive(dir, pattern, &mut results)?;
    Ok(results)
}

fn find_recursive(dir: &str, pattern: &str, results: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            find_recursive(path.to_str().unwrap_or(""), pattern, results)?;
        }
        
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                if name_str.contains(pattern) {
                    results.push(path.clone());
                }
            }
        }
    }
    
    Ok(())
}

pub fn grep(path: &str, pattern: &str) -> CrateResult<String> {
    let content = fs::read_to_string(path)?;
    let mut result = String::new();
    
    for (i, line) in content.lines().enumerate() {
        if line.contains(pattern) {
            result.push_str(&format!("{}:{}\n", i + 1, line));
        }
    }
    
    Ok(result)
}

pub fn ln(target: &str, link_name: &str) -> CrateResult<()> {
    std::os::unix::fs::symlink(target, link_name)?;
    Ok(())
}
