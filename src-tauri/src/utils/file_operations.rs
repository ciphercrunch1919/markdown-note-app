use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::Path;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref PATH: Mutex<Option<String>> = Mutex::new(Some("Vaults".to_string()));
}

// Sets the base path for file operations.
#[allow(dead_code)]
pub fn set_base_path(path: Option<String>) {
    *PATH.lock().unwrap() = path;
}

// Creates a directory if it doesn't already exist.
pub fn create_directory(path: &str) -> io::Result<()> {
    let base_path = PATH.lock().unwrap();
    let full_path = match &*base_path {
        Some(base) => format!("{}/{}", base, path),
        None => path.to_string(),
    };
    
    if !Path::new(&full_path).exists() {
        fs::create_dir_all(&full_path)?;
    }
    Ok(())
}

// Deletes a directory and all its contents if it exists.
pub fn delete_directory(path: &str) -> io::Result<()> {
    let base_path = PATH.lock().unwrap();
    let full_path = match &*base_path {
        Some(base) => format!("{}/{}", base, path),
        None => path.to_string(),
    };
    
    if Path::new(&full_path).exists() {
        println!("🧹 Attempting to delete directory: {}", full_path);
        // Recursively delete all files and subdirectories
        for entry in fs::read_dir(&full_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Err(e) = delete_directory(&path.to_string_lossy()) {
                    println!("❌ Failed to delete subdirectory {}: {}", path.display(), e);
                }
            } else {
                if let Err(e) = fs::remove_file(&path) {
                    println!("❌ Failed to delete file {}: {}", path.display(), e);
                }
            }
        }
        // Delete the directory itself
        if let Err(e) = fs::remove_dir(&full_path) {
            println!("❌ Failed to delete directory {}: {}", full_path, e);
            return Err(e);
        }
        println!("✅ Successfully deleted directory: {}", full_path);
    }
    Ok(())
}

// Writes content to a file, creating it if necessary.
pub fn write_to_file(path: &str, content: &str) -> io::Result<()> {
    let base_path = PATH.lock().unwrap();
    let full_path = match &*base_path {
        Some(base) => format!("{}/{}", base, path),
        None => path.to_string(),
    };
    
    let mut file = File::create(&full_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// Reads content from a file.
pub fn read_from_file(path: &str) -> io::Result<String> {
    let base_path = PATH.lock().unwrap();
    let full_path = match &*base_path {
        Some(base) => format!("{}/{}", base, path),
        None => path.to_string(),
    };
    
    let mut file = File::open(&full_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

// Deletes a file if it exists.
pub fn delete_file(path: &str) -> io::Result<()> {
    let base_path = PATH.lock().unwrap();
    let full_path = match &*base_path {
        Some(base) => format!("{}/{}", base, path),
        None => path.to_string(),
    };
    
    if Path::new(&full_path).exists() {
        fs::remove_file(&full_path)?;
    }
    Ok(())
}

// Rename a file if it exists.
pub fn rename_file(old_path: &str, new_path: &str) -> io::Result<()> {
    let base_path = PATH.lock().unwrap();
    let old_full_path = match &*base_path {
        Some(base) => format!("{}/{}", base, old_path),
        None => old_path.to_string(),
    };
    let new_full_path = match &*base_path {
        Some(base) => format!("{}/{}", base, new_path),
        None => new_path.to_string(),
    };
    
    if Path::new(&old_full_path).exists() {
        fs::rename(&old_full_path, &new_full_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_directory() {
        // Disable the base path for tests
        set_base_path(None);

        let test_dir = "test_dir";
        create_directory(test_dir).unwrap();
        assert!(Path::new(test_dir).exists());
        delete_directory(test_dir).unwrap();
    }

    #[test]
    fn test_write_read_delete_file() {
        // Disable the base path for tests
        set_base_path(None);

        let test_file = "test_file.txt";
        let content = "Hello, Rust!";
        
        write_to_file(test_file, content).unwrap();
        assert_eq!(read_from_file(test_file).unwrap(), content);

        delete_file(test_file).unwrap();
        assert!(!Path::new(test_file).exists());
    }

    #[test]
    fn test_delete_directory() {
        // Disable the base path for tests
        set_base_path(None);

        let test_dir = "test_delete_dir";
        create_directory(test_dir).unwrap();
        assert!(Path::new(test_dir).exists());

        delete_directory(test_dir).unwrap();
        assert!(!Path::new(test_dir).exists());
    }
}