use serde::{Serialize, Deserialize};

use crate::utils::{file_operations, string_utils};

#[derive(Serialize, Deserialize)]
pub struct Vault {
    pub name: String,
    pub path: String,
}

impl Vault {
    pub fn create_vault(name: &str) -> std::io::Result<Self> {
        let sanitized_name = string_utils::sanitize_filename(name);
        let vault_path = format!("Vaults/{}", sanitized_name);

        // Use file_operations::create_directory instead of std::fs::create_dir_all
        file_operations::create_directory(&vault_path)?;

        Ok(Vault {
            name: sanitized_name,
            path: vault_path,
        })
    }

    pub fn delete_vault(&self) -> std::io::Result<()> {
        // Use file_operations::delete_directory instead of std::fs::remove_dir_all
        file_operations::delete_directory(&self.path)?;
        Ok(())
    }

    pub fn list_vaults(base_path: &str) -> std::io::Result<Vec<String>> {
        // Use file_operations::read_dir (if implemented) or keep using std::fs::read_dir
        let paths = std::fs::read_dir(base_path)?;
        Ok(paths
            .filter_map(|entry| entry.ok().map(|e| e.file_name().into_string().ok()))
            .flatten()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_create_vault() {
        let vault_name = "TestVault";
        let vault = Vault::create_vault(vault_name).expect("Failed to create test vault");
        assert_eq!(vault.name, vault_name, "Vault name mismatch");
        assert!(Path::new(&vault.path).exists(), "Vault directory does not exist");

        // Cleanup
        vault.delete_vault().expect("Failed to delete vault");
        assert!(!Path::new(&vault.path).exists(), "Vault directory was not deleted");
    }

    #[test]
    fn test_delete_vault() {
        let vault_name = "TestVaultToDelete";
        let vault = Vault::create_vault(vault_name).expect("Failed to create test vault");
        assert!(Path::new(&vault.path).exists(), "Vault should exist before deletion");

        // Cleanup
        vault.delete_vault().expect("Vault deletion failed");
        assert!(!Path::new(&vault.path).exists(), "Vault directory was not deleted");
    }
}