//! Preferences module for Rancer
//!
//! Provides user preference management with TOML-based configuration files.
//! Stores preferences in platform-specific config directories:
//! - Windows: %APPDATA%\rancer\config.toml
//! - Linux: ~/.config/rancer/config.toml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_VERSION: &str = "1.0";

/// Main preferences structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub version: String,
    pub window: WindowPreferences,
    pub canvas: CanvasPreferences,
    pub brush: BrushPreferences,
    pub renderer: RendererPreferences,
    pub palette: PalettePreferences,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPreferences {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

/// Canvas configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasPreferences {
    pub width: u32,
    pub height: u32,
    pub background_color: String, // Hex format: "#FFFFFF"
}

/// Brush configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushPreferences {
    pub default_size: f32,
    pub sizes: Vec<f32>,
}

/// Renderer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererPreferences {
    pub msaa_samples: u32,
    pub clear_color: String, // Hex format: "#FFFFFF"
}

/// Palette configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalettePreferences {
    pub selected_index: usize,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION.to_string(),
            window: WindowPreferences {
                width: 1280,
                height: 720,
                title: "Rancer".to_string(),
            },
            canvas: CanvasPreferences {
                width: 1280,
                height: 720,
                background_color: "#FFFFFF".to_string(),
            },
            brush: BrushPreferences {
                default_size: 3.0,
                sizes: vec![3.0, 5.0, 10.0, 25.0, 50.0],
            },
            renderer: RendererPreferences {
                msaa_samples: 1,
                clear_color: "#FFFFFF".to_string(),
            },
            palette: PalettePreferences { selected_index: 0 },
        }
    }
}

/// Get the platform-specific config file path
pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));

    #[cfg(target_os = "windows")]
    {
        path.push("rancer");
    }

    #[cfg(target_os = "linux")]
    {
        path.push("rancer");
    }

    path.push("config.toml");
    path
}

/// Load preferences from config file
/// Creates default config file if it doesn't exist
pub fn load() -> Preferences {
    let config_path = get_config_path();

    if !config_path.exists() {
        crate::logger::info(&format!(
            "Config file not found, creating defaults at: {:?}",
            config_path
        ));
        let prefs = Preferences::default();
        if let Err(e) = save(&prefs) {
            crate::logger::error(&format!("Failed to create config file: {}", e));
        }
        return prefs;
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str::<Preferences>(&content) {
            Ok(prefs) => {
                crate::logger::info(&format!("Loaded preferences from: {:?}", config_path));
                prefs
            }
            Err(e) => {
                crate::logger::warn(&format!(
                    "Failed to parse config file, using defaults: {}",
                    e
                ));
                Preferences::default()
            }
        },
        Err(e) => {
            crate::logger::warn(&format!(
                "Failed to read config file, using defaults: {}",
                e
            ));
            Preferences::default()
        }
    }
}

/// Save preferences to config file
pub fn save(prefs: &Preferences) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let toml_string = toml::to_string(prefs)?;
    fs::write(&config_path, toml_string)?;
    crate::logger::info(&format!("Saved preferences to: {:?}", config_path));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preferences() {
        let prefs = Preferences::default();
        assert_eq!(prefs.version, "1.0");
        assert_eq!(prefs.window.width, 1280);
        assert_eq!(prefs.window.height, 720);
        assert_eq!(prefs.window.title, "Rancer");
        assert_eq!(prefs.canvas.width, 1280);
        assert_eq!(prefs.canvas.height, 720);
        assert_eq!(prefs.canvas.background_color, "#FFFFFF");
        assert_eq!(prefs.brush.default_size, 3.0);
        assert_eq!(prefs.brush.sizes.len(), 5);
        assert_eq!(prefs.renderer.msaa_samples, 1);
        assert_eq!(prefs.renderer.clear_color, "#FFFFFF");
        assert_eq!(prefs.palette.selected_index, 0);
    }

    #[test]
    fn test_preferences_serialization() {
        let prefs = Preferences::default();
        let toml_string = toml::to_string(&prefs).expect("Failed to serialize");
        assert!(toml_string.contains("version = \"1.0\""));
        assert!(toml_string.contains("width = 1280"));
    }

    #[test]
    fn test_config_path() {
        let path = get_config_path();
        assert!(path.to_string_lossy().contains("rancer"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_preferences_deserialization() {
        let original = Preferences::default();
        let toml_string = toml::to_string(&original).expect("Failed to serialize");
        let deserialized: Preferences =
            toml::from_str(&toml_string).expect("Failed to deserialize");

        assert_eq!(deserialized.version, original.version);
        assert_eq!(deserialized.window.width, original.window.width);
        assert_eq!(deserialized.window.height, original.window.height);
        assert_eq!(deserialized.window.title, original.window.title);
        assert_eq!(deserialized.canvas.width, original.canvas.width);
        assert_eq!(deserialized.brush.default_size, original.brush.default_size);
        assert_eq!(deserialized.brush.sizes.len(), original.brush.sizes.len());
        assert_eq!(
            deserialized.palette.selected_index,
            original.palette.selected_index
        );
    }

    #[test]
    fn test_preferences_round_trip() {
        let mut prefs = Preferences::default();
        prefs.window.width = 1920;
        prefs.window.height = 1080;
        prefs.brush.default_size = 10.0;
        prefs.palette.selected_index = 5;

        let toml_string = toml::to_string(&prefs).expect("Failed to serialize");
        let recovered: Preferences = toml::from_str(&toml_string).expect("Failed to deserialize");

        assert_eq!(recovered.window.width, 1920);
        assert_eq!(recovered.window.height, 1080);
        assert_eq!(recovered.brush.default_size, 10.0);
        assert_eq!(recovered.palette.selected_index, 5);
    }

    #[test]
    fn test_preferences_save_and_load() {
        let mut prefs = Preferences::default();
        prefs.window.width = 1024;
        prefs.window.height = 768;

        // Save to a temp file
        let temp_dir = std::env::temp_dir().join("rancer_test_prefs");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let temp_config = temp_dir.join("config.toml");

        let toml_string = toml::to_string(&prefs).unwrap();
        std::fs::write(&temp_config, &toml_string).unwrap();

        // Read it back
        let content = std::fs::read_to_string(&temp_config).unwrap();
        let loaded: Preferences = toml::from_str(&content).unwrap();

        assert_eq!(loaded.window.width, 1024);
        assert_eq!(loaded.window.height, 768);

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_preferences_corrupted_toml() {
        let corrupted = "this is not valid toml {{{{";
        let result = toml::from_str::<Preferences>(corrupted);
        assert!(result.is_err(), "Corrupted TOML should fail to parse");
    }

    #[test]
    fn test_preferences_window_defaults() {
        let prefs = Preferences::default();
        assert!(prefs.window.width > 0);
        assert!(prefs.window.height > 0);
        assert!(!prefs.window.title.is_empty());
    }

    #[test]
    fn test_preferences_brush_defaults() {
        let prefs = Preferences::default();
        assert!(prefs.brush.default_size > 0.0);
        assert!(prefs.brush.default_size <= 100.0);
    }

    #[test]
    fn test_preferences_palette_defaults() {
        let prefs = Preferences::default();
        assert_eq!(prefs.palette.selected_index, 0);
    }

    #[test]
    fn test_preferences_serialization_roundtrip() {
        let prefs = Preferences::default();
        let serialized = toml::to_string(&prefs).unwrap();
        let deserialized: Preferences = toml::from_str(&serialized).unwrap();
        assert_eq!(prefs.window.width, deserialized.window.width);
        assert_eq!(prefs.window.height, deserialized.window.height);
        assert_eq!(prefs.brush.default_size, deserialized.brush.default_size);
    }

    #[test]
    fn test_preferences_partial_update() {
        let mut prefs = Preferences::default();
        let original_width = prefs.window.width;

        prefs.window.width = 1920;
        prefs.brush.default_size = 15.0;
        prefs.palette.selected_index = 5;

        assert_eq!(prefs.window.width, 1920);
        assert_ne!(prefs.window.width, original_width);
        assert_eq!(prefs.brush.default_size, 15.0);
        assert_eq!(prefs.palette.selected_index, 5);
    }
}
