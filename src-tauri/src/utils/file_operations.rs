use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::Path;

// Creates a directory if it doesn't already exist.
pub fn create_directory(path: &str) -> io::Result<()> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

// Writes content to a file, creating it if necessary.
pub fn write_to_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// Reads content from a file.
pub fn read_from_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

// Deletes a file if it exists.
pub fn delete_file(path: &str) -> io::Result<()> {
    if Path::new(path).exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

// Lists all files in a directory with a given extension (e.g., `.md`).
pub fn list_files_with_extension(dir: &str, extension: &str) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                        files.push(filename.to_string());
                    }
                }
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_directory() {
        let test_dir = "test_dir";
        create_directory(test_dir).unwrap();
        assert!(Path::new(test_dir).exists());
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_write_read_delete_file() {
        let test_file = "test_file.txt";
        let content = "Hello, Rust!";
        
        write_to_file(test_file, content).unwrap();
        assert_eq!(read_from_file(test_file).unwrap(), content);

        delete_file(test_file).unwrap();
        assert!(Path::new(test_file).exists() == false);
    }

    #[test]
    fn test_list_files_with_extension() {
        let test_dir = "test_files";
        create_directory(test_dir).unwrap();

        let test_md = format!("{}/note1.md", test_dir);
        let test_txt = format!("{}/note2.txt", test_dir);

        write_to_file(&test_md, "Markdown content").unwrap();
        write_to_file(&test_txt, "Text content").unwrap();

        let md_files = list_files_with_extension(test_dir, "md").unwrap();
        assert_eq!(md_files, vec!["note1.md"]);

        fs::remove_dir_all(test_dir).unwrap();
    }
}