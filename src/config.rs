use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MANIFEST_NAME: &str = "dot.toml";
pub const DEFAULT_BACKUP_DIR: &str = ".bak";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    File,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemConfig {
    pub target: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_deploy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    #[serde(default = "default_true")]
    pub backup_enabled: bool,
    #[serde(default = "default_backup_dir")]
    pub backup_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_backup_dir() -> String {
    DEFAULT_BACKUP_DIR.to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            backup_enabled: true,
            backup_dir: default_backup_dir(),
            package_manager: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Hooks {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_deploy: Vec<String>,
}

fn is_default_hooks(h: &Hooks) -> bool {
    h.post_deploy.is_empty()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum DependencyGroup {
    Simple(Vec<String>),
    Detailed(DependencyDetail),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DependencyDetail {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manager: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
}

impl DependencyGroup {
    pub fn packages(&self) -> &[String] {
        match self {
            Self::Simple(pkgs) => pkgs,
            Self::Detailed(detail) => &detail.packages,
        }
    }

    pub fn manager(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed(detail) => detail.manager.as_deref(),
        }
    }

    pub fn cmd(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed(detail) => detail.cmd.as_deref(),
        }
    }

    pub fn script(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed(detail) => detail.script.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DotConfig {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default, skip_serializing_if = "is_default_hooks")]
    pub hooks: Hooks,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub installers: IndexMap<String, String>,
    #[serde(default)]
    pub items: IndexMap<String, ItemConfig>,
    #[serde(default)]
    pub dependencies: IndexMap<String, DependencyGroup>,
}

impl DotConfig {
    pub fn default_template() -> Self {
        let mut items = IndexMap::new();
        items.insert(
            "zsh/zshrc".to_string(),
            ItemConfig {
                target: "~/.zshrc".to_string(),
                item_type: ItemType::File,
                tags: vec![],
                post_deploy: None,
            },
        );
        items.insert(
            "nvim".to_string(),
            ItemConfig {
                target: "~/.config/nvim".to_string(),
                item_type: ItemType::Folder,
                tags: vec!["dev".to_string()],
                post_deploy: None,
            },
        );

        let mut dependencies = IndexMap::new();
        dependencies.insert(
            "core".to_string(),
            DependencyGroup::Simple(vec!["git".into(), "curl".into(), "zsh".into(), "tmux".into()]),
        );
        dependencies.insert(
            "rust_tools".to_string(),
            DependencyGroup::Simple(vec!["ripgrep".into(), "bat".into(), "eza".into(), "bottom".into()]),
        );

        Self {
            settings: Settings::default(),
            hooks: Hooks::default(),
            installers: IndexMap::new(),
            items,
            dependencies,
        }
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest at '{}'", path.display()))?;
        let config: DotConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML manifest at '{}'", path.display()))?;
        Ok(config)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        let content =
            toml::to_string_pretty(self).context("Failed to serialize dot.toml configuration")?;
        crate::fs::atomic_write(path, &content)?;
        Ok(())
    }

    pub fn find_manifest() -> Result<PathBuf> {
        let mut curr =
            std::env::current_dir().context("Failed to get current working directory")?;
        loop {
            let candidate = curr.join(DEFAULT_MANIFEST_NAME);
            if candidate.exists() {
                return Ok(candidate);
            }
            if !curr.pop() {
                break;
            }
        }
        anyhow::bail!(
            "Could not find '{}' in the current directory or any parent. Did you run 'dotman init'?",
            DEFAULT_MANIFEST_NAME
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_config() {
        let template = DotConfig::default_template();
        let serialized = toml::to_string(&template).unwrap();
        let deserialized: DotConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(template, deserialized);
    }

    #[test]
    fn test_parse_detailed_dependencies() {
        let toml_str = r#"
        [settings]
        package_manager = "cargo"

        [installers]
        aur = "yay -S --needed {packages}"
        binstall = "cargo binstall -y {packages}"

        [dependencies.core]
        packages = ["git", "curl"]

        [dependencies.rust_tools]
        manager = "cargo"
        packages = ["ripgrep", "bat"]

        [dependencies.custom_env]
        script = "scripts/setup.sh"
        "#;

        let config: DotConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.settings.package_manager.as_deref(), Some("cargo"));
        assert_eq!(config.installers.get("aur").unwrap(), "yay -S --needed {packages}");
        assert_eq!(config.dependencies.len(), 3);

        let core = config.dependencies.get("core").unwrap();
        assert_eq!(core.packages(), &["git", "curl"]);

        let rust = config.dependencies.get("rust_tools").unwrap();
        assert_eq!(rust.packages(), &["ripgrep", "bat"]);
        assert_eq!(rust.manager(), Some("cargo"));

        let custom = config.dependencies.get("custom_env").unwrap();
        assert_eq!(custom.script(), Some("scripts/setup.sh"));
    }
}
