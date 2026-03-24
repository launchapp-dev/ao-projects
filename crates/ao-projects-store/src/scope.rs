use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub fn state_root_dir() -> PathBuf {
    dirs_home().join(".ao-projects")
}

pub fn scoped_state_root(project_root: &Path) -> PathBuf {
    let scope = repository_scope(project_root);
    state_root_dir().join(scope)
}

pub fn repository_scope(project_root: &Path) -> String {
    let canonical = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let sanitized = sanitize_identifier(&name);
    let hash = short_hash(&canonical.to_string_lossy());
    format!("{}-{}", sanitized, hash)
}

fn sanitize_identifier(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .to_lowercase()
}

fn short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])
}

fn dirs_home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

// Re-export for convenience
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
