//! UI state management and egui integration

pub mod state;
pub use state::{UiState, ToolType};

pub mod egui_integration;
pub use egui_integration::EguiIntegration;

pub mod egui_impl;
pub use egui_impl::show_ui;
pub use egui_impl::Theme;
pub use egui_impl::IconCache;

pub mod icons;
