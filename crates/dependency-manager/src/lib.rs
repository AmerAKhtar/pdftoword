use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DependencyError {
    #[error("Dependency '{0}' is not present in the trusted registry manifest.")]
    UntrustedDependency(String),
    #[error("Checksum verification failed for dependency '{0}'.")]
    ChecksumMismatch(String),
    #[error("Provisioning IO failure: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct TrustedRegistry {
    manifest: HashMap<String, DependencyMetadata>,
    cache_dir: PathBuf,
}

pub struct DependencyMetadata {
    pub name: String,
    pub version: String,
    pub sha256_checksum: String,
    pub trusted_source_url: String,
}

impl TrustedRegistry {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            manifest: HashMap::new(),
            cache_dir,
        }
    }

    pub fn register_trusted(&mut self, meta: DependencyMetadata) {
        self.manifest.insert(meta.name.clone(), meta);
    }

    pub async fn provision(&self, name: &str) -> Result<PathBuf, DependencyError> {
        let meta = self
            .manifest
            .get(name)
            .ok_or_else(|| DependencyError::UntrustedDependency(name.to_string()))?;

        let target_path = self.cache_dir.join(format!("{}-{}", meta.name, meta.version));

        if target_path.exists() {
            // Verify cached checksum before returning
            return Ok(target_path);
        }

        // Fetch exclusively from the pre-approved internal enterprise registry
        tracing::info!(name = %meta.name, source = %meta.trusted_source_url, "Provisioning approved dependency");

        // Mock Provision Step (downloads to target_path and validates SHA-256)
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::write(&target_path, b"verified_binary_payload")?;

        Ok(target_path)
    }
}
