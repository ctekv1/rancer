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
            palette: PalettePreferences {
                selected_index: 0,
            },
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
        crate::logger::info(&format!("Config file not found, creating defaults at: {:?}", config_path));
        let prefs = Preferences::default();
        if let Err(e) = save(&prefs) {
            crate::logger::error(&format!("Failed to create config file: {}", e));
        }
        return prefs;
    }
    
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            match toml::from_str::<Preferences>(&content) {
                Ok(prefs) => {
                    crate::logger::info(&format!("Loaded preferences from: {:?}", config_path));
                    prefs
                }
                Err(e) => {
                    crate::logger::warn(&format!("Failed to parse config file, using defaults: {}", e));
                    Preferences::default()
                }
            }
        }
        Err(e) => {
            crate::logger::warn(&format!("Failed to read config file, using defaults: {}", e));
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
}
