use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Parsed contents of an `ignis.toml` plugin manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub game: GameSection,
    pub display: Option<DisplaySection>,
    pub rendering: Option<RenderingSection>,
}

/// Required `[game]` section — identity information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSection {
    pub name: String,
    pub version: String,
    pub author: String,
    pub igi_version: String,
}

/// Optional `[display]` section — resolution preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySection {
    pub resolution: Option<Resolution>,
}

/// Logical resolution the plugin was designed for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

/// Optional `[rendering]` section — rendering tier hint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingSection {
    pub tier: Option<String>,
}

/// A discovered plugin: its manifest paired with the path to its `.wasm` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPlugin {
    pub id: String,
    pub manifest: PluginManifest,
    #[serde(skip)]
    pub wasm_path: PathBuf,
}

/// Scans `plugins_dir` for subdirectories containing `ignis.toml` + a `.wasm` file.
/// Returns all valid plugins; logs warnings for malformed entries and skips them.
pub fn discover_plugins(plugins_dir: &Path) -> Vec<DiscoveredPlugin> {
    let mut plugins = Vec::new();

    let entries = match std::fs::read_dir(plugins_dir) {
        Ok(e) => e,
        Err(e) => {
            log::warn!("Cannot read plugins directory {}: {e}", plugins_dir.display());
            return plugins;
        }
    };

    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }

        let manifest_path = dir.join("ignis.toml");
        if !manifest_path.exists() {
            log::warn!("Plugin dir {} has no ignis.toml — skipping", dir.display());
            continue;
        }

        let toml_str = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Cannot read {}: {e}", manifest_path.display());
                continue;
            }
        };

        let manifest: PluginManifest = match toml::from_str(&toml_str) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Invalid ignis.toml in {}: {e}", dir.display());
                continue;
            }
        };

        // Find the first .wasm file in the directory
        let wasm_path = match find_wasm_file(&dir) {
            Some(p) => p,
            None => {
                log::warn!("No .wasm file found in {} — skipping", dir.display());
                continue;
            }
        };

        let id = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        plugins.push(DiscoveredPlugin {
            id,
            manifest,
            wasm_path,
        });
    }

    log::info!("Discovered {} plugin(s)", plugins.len());
    plugins
}

/// Finds the first `.wasm` file in a directory.
fn find_wasm_file(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|ext| ext == "wasm"))
}

/// Tauri command: scans the `plugins/` directory and returns all discovered plugins.
#[tauri::command]
pub fn list_discovered_plugins(app: tauri::AppHandle) -> Result<Vec<DiscoveredPlugin>, String> {
    let plugins_dir = crate::resolve_plugins_dir(&app)?;
    Ok(discover_plugins(&plugins_dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_manifest_parsing() {
        let toml_str = r#"
[game]
name = "Test Game"
version = "0.1.0"
author = "Test Author"
igi_version = "1"

[display]
resolution = { width = 320, height = 240 }

[rendering]
tier = "standard"
"#;
        let manifest: PluginManifest = toml::from_str(toml_str).expect("parse manifest");
        assert_eq!(manifest.game.name, "Test Game");
        assert_eq!(manifest.game.author, "Test Author");
        assert_eq!(manifest.display.as_ref().unwrap().resolution.as_ref().unwrap().width, 320);
        assert_eq!(manifest.rendering.as_ref().unwrap().tier.as_deref(), Some("standard"));
    }

    #[test]
    fn test_discover_plugins_with_temp_dir() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let plugin_dir = tmp.path().join("test-game");
        fs::create_dir_all(&plugin_dir).expect("create plugin dir");

        fs::write(
            plugin_dir.join("ignis.toml"),
            r#"
[game]
name = "Test Game"
version = "1.0.0"
author = "Tester"
igi_version = "1"
"#,
        )
        .expect("write manifest");

        // Create a dummy .wasm file
        fs::write(plugin_dir.join("game.wasm"), [0u8; 4]).expect("write wasm");

        let plugins = discover_plugins(tmp.path());
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].manifest.game.name, "Test Game");
        assert_eq!(plugins[0].id, "test-game");
        assert!(plugins[0].wasm_path.ends_with("game.wasm"));
    }

    #[test]
    fn test_discover_skips_missing_wasm() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let plugin_dir = tmp.path().join("no-wasm");
        fs::create_dir_all(&plugin_dir).expect("create dir");

        fs::write(
            plugin_dir.join("ignis.toml"),
            "[game]\nname=\"X\"\nversion=\"0\"\nauthor=\"A\"\nigi_version=\"1\"\n",
        )
        .expect("write manifest");

        let plugins = discover_plugins(tmp.path());
        assert_eq!(plugins.len(), 0);
    }
}
