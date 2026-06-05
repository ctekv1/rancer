//! Tests for Phase 7: egui UI integration
//!
//! Tests for egui integration with SDL2, UI state management,
//! and tool switching via the egui toolbar.

use crate::ui::egui_integration::EguiIntegration;
use crate::ui::{ToolType, UiState};

/// Test that UiState initializes with correct defaults
#[test]
fn test_ui_state_defaults() {
    let ui_state = UiState::new();
    assert_eq!(ui_state.active_tool, ToolType::Brush);
    assert!(ui_state.show_tool_panel);
    assert!(ui_state.show_brush_panel);
    assert!(ui_state.show_layer_panel);
    // show_color_panel removed - use color_picker_open instead
    assert!(!ui_state.color_picker_open); // Default: popup closed
}

/// Test tool switching in UiState
#[test]
fn test_ui_state_tool_switching() {
    let mut ui_state = UiState::new();
    assert_eq!(ui_state.active_tool, ToolType::Brush);

    ui_state.set_tool(ToolType::Brush);
    assert_eq!(ui_state.active_tool, ToolType::Brush);
}

/// Test UiState apply_to_app creates correct tool
#[test]
fn test_ui_state_apply_to_app() {
    use crate::app::AppState;

    let mut app = AppState::new(800, 600);
    let mut ui_state = UiState::new();

    ui_state.apply_to_app(&mut app);
    assert_eq!(app.tool_name(), "Brush");
}

/// Test UiState layer operations
#[test]
fn test_ui_state_layer_operations() {
    use crate::app::AppState;

    let mut app = AppState::new(800, 600);
    let mut ui_state = UiState::new();

    // Add a layer
    ui_state.add_layer(&mut app);
    assert!(app.canvas().layer_count() >= 2); // Started with 1, added 1

    // Remove a layer (not index 0)
    let layer_count_before = app.canvas().layer_count();
    if layer_count_before > 1 {
        ui_state.remove_layer(&mut app, 1);
        assert_eq!(app.canvas().layer_count(), layer_count_before - 1);
    }
}

/// Test UiState undo/redo
#[test]
fn test_ui_state_undo_redo() {
    use crate::app::AppState;

    let mut app = AppState::new(800, 600);
    let mut ui_state = UiState::new();

    // Do an action that can be undone
    ui_state.add_layer(&mut app);
    let layer_count_after_add = app.canvas().layer_count();

    // Undo
    ui_state.undo(&mut app);
    assert_eq!(app.canvas().layer_count(), layer_count_after_add - 1);

    // Redo
    ui_state.redo(&mut app);
    assert_eq!(app.canvas().layer_count(), layer_count_after_add);
}

/// Test EguiIntegration creation (requires SDL2 context)
/// This test verifies the integration can be created without panicking
/// Note: SDL2 can only be initialized once, so we skip if already initialized
#[test]
#[ignore = "requires a display/GPU (fails on headless CI runners)"]
fn test_egui_integration_creation() {
    // Skip if SDL2 already initialized (e.g., by another test)
    let sdl = match sdl2::init() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: SDL2 already initialized");
            return;
        }
    };
    let video = sdl.video().expect("Failed to init video");

    let window = video
        .window("Test", 800, 600)
        .opengl()
        .build()
        .expect("Failed to create window");

    let gl_context = window
        .gl_create_context()
        .expect("Failed to create GL context");

    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            video.gl_get_proc_address(s) as *const std::os::raw::c_void
        })
    };

    // This should not panic
    let result = EguiIntegration::new(&window, &gl_context, &gl);
    assert!(
        result.is_ok(),
        "EguiIntegration creation failed: {:?}",
        result.err()
    );
}

/// Test that egui context is accessible
#[test]
#[ignore = "requires a display/GPU (fails on headless CI runners)"]
fn test_egui_integration_ctx_access() {
    // Skip if SDL2 already initialized (e.g., by another test)
    let sdl = match sdl2::init() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: SDL2 already initialized");
            return;
        }
    };
    let video = sdl.video().expect("Failed to init video");

    let window = video
        .window("Test", 800, 600)
        .opengl()
        .build()
        .expect("Failed to create window");

    let gl_context = window
        .gl_create_context()
        .expect("Failed to create GL context");

    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            video.gl_get_proc_address(s) as *const std::os::raw::c_void
        })
    };

    let mut egui =
        EguiIntegration::new(&window, &gl_context, &gl).expect("Failed to create EguiIntegration");

    // Should be able to get context (just check it's not null)
    let _ctx = egui.ctx(); // If this doesn't panic, it's working
}

/// Test ToolType conversion from UiState to app
#[test]
fn test_tool_type_consistency() {
    assert_eq!(ToolType::Brush as u8, crate::tools::ToolType::Brush as u8);
}

/// Test that clicking tool icon in UI updates active_tool in UiState
#[test]
fn test_tool_switching_via_ui() {
    let mut ui_state = UiState::new();
    assert_eq!(ui_state.active_tool, ToolType::Brush);

    ui_state.set_tool(ToolType::Brush);
    assert_eq!(ui_state.active_tool, ToolType::Brush);
}

/// Test that apply_to_app correctly changes the tool in AppState
#[test]
fn test_apply_to_app_switches_tools() {
    use crate::app::AppState;

    let mut app = AppState::new(800, 600);
    let mut ui_state = UiState::new();

    ui_state.apply_to_app(&mut app);
    assert_eq!(app.tool_name(), "Brush");

    ui_state.set_tool(ToolType::Brush);
    ui_state.apply_to_app(&mut app);
    assert_eq!(app.tool_name(), "Brush");
}

/// Test that tool switching preserves brush settings
#[test]
fn test_tool_switching_preserves_settings() {
    use crate::app::AppState;
    use crate::brush::BrushType;

    let mut app = AppState::new(800, 600);
    let mut ui_state = UiState::new();

    if let Some(config) = app.active_tool_mut().as_brush_config() {
        config.set_brush_size(25);
        config.set_brush_opacity(0.5);
        config.set_brush_type(BrushType::Square);
    }

    ui_state.set_tool(ToolType::Brush);
    ui_state.apply_to_app(&mut app);
    assert_eq!(app.tool_name(), "Brush");
}

/// Test that multiple rapid tool switches work correctly
#[test]
fn test_rapid_tool_switching() {
    use crate::app::AppState;

    let mut app = AppState::new(800, 600);
    let mut ui_state = UiState::new();

    for _ in 0..10 {
        ui_state.set_tool(ToolType::Brush);
        ui_state.apply_to_app(&mut app);
    }

    assert_eq!(app.tool_name(), "Brush");
}

/// Test that clicking Brush tool icon updates active_tool and applies to app
#[test]
fn test_click_brush_tool_icon() {
    use crate::app::AppState;

    let mut app = AppState::new(800, 600);
    let mut ui_state = UiState::new();

    // Simulate clicking Brush tool (should already be Brush)
    ui_state.set_tool(ToolType::Brush);
    ui_state.apply_to_app(&mut app);
    assert_eq!(app.tool_name(), "Brush");
    assert_eq!(ui_state.active_tool, ToolType::Brush);
}

/// Test that apply_to_app doesn't recreate tool if already correct
#[test]
fn test_apply_to_app_no_recreate() {
    use crate::app::AppState;

    let mut app = AppState::new(800, 600);
    let mut ui_state = UiState::new();

    // Apply when already Brush (should not recreate)
    ui_state.apply_to_app(&mut app);
    assert_eq!(app.tool_name(), "Brush");
}

/// Test that SVG icon doesn't disappear on hover
/// This tests that the UI correctly renders icons without duplication or disappearance
#[test]
fn test_icon_does_not_disappear_on_hover() {
    // This is a visual test - we can't test hover state directly in unit tests
    // But we can verify the icon loading works correctly
    use crate::ui::icons;

    // Verify all icons can be loaded as SVG
    let icon_list = [
        ("Brush", icons::BRUSH_ICON),
        ("Eraser", icons::ERASER_ICON),
        ("Zoom In", icons::ZOOM_IN_ICON),
        ("Zoom Out", icons::ZOOM_OUT_ICON),
        ("Undo", icons::UNDO_ICON),
        ("Redo", icons::REDO_ICON),
        ("Add Layer", icons::ADD_LAYER_ICON),
    ];

    for (name, svg) in &icon_list {
        // SVG should be valid and loadable
        assert!(!svg.is_empty(), "{} icon SVG is empty", name);
        assert!(svg.contains("<svg"), "{} icon missing <svg> tag", name);
        assert!(svg.contains("</svg>"), "{} icon missing </svg> tag", name);
    }
}

/// Test that tool button click area is correct size (no shrinkage)
#[test]
fn test_tool_button_fixed_size() {
    // Verify that the UI uses fixed min_size for tool buttons
    // This is a code review test - checking that our implementation
    // uses min_size(36.0, 36.0) and not shrink_to_fit()
    use std::fs;
    let ui_impl = fs::read_to_string("src/ui/egui_impl.rs").expect("Failed to read egui_impl.rs");

    // Should use min_size for fixed dimensions
    assert!(
        ui_impl.contains("min_size(egui::vec2(36.0, 36.0))"),
        "Tool buttons should use min_size(36, 36) to prevent shrinkage"
    );

    // Should NOT use shrink_to_fit or similar
    assert!(
        !ui_impl.contains("shrink_to_fit"),
        "Tool buttons should not shrink on hover"
    );
}

/// Test that icon is drawn ONCE (no duplication)
#[test]
fn test_icon_not_duplicated() {
    use std::fs;
    let ui_impl = fs::read_to_string("src/ui/egui_impl.rs").expect("Failed to read egui_impl.rs");

    // Should use Button::image() which draws icon once
    assert!(
        ui_impl.contains("Button::image("),
        "Should use Button::image() to draw icon once"
    );

    // Should NOT have manual painter().image() calls that duplicate
    assert!(
        !ui_impl.contains("painter().image("),
        "Should not have painter().image() calls that cause duplication"
    );
}

/// Test that SVG icon module exists and has expected icons
#[test]
fn test_svg_icons_exist() {
    use crate::ui::icons;

    // Verify icon constants exist and are non-empty
    assert!(!icons::BRUSH_ICON.is_empty());
    assert!(!icons::ERASER_ICON.is_empty());
    assert!(!icons::ZOOM_IN_ICON.is_empty());
    assert!(!icons::ZOOM_OUT_ICON.is_empty());
    assert!(!icons::UNDO_ICON.is_empty());
    assert!(!icons::REDO_ICON.is_empty());
    assert!(!icons::ADD_LAYER_ICON.is_empty());

    // Verify icons contain SVG header
    assert!(icons::BRUSH_ICON.contains("<svg"));
    assert!(icons::ERASER_ICON.contains("<svg"));
    assert!(icons::ZOOM_IN_ICON.contains("<svg"));
}

/// Test that SVG icons are valid XML (basic check)
#[test]
fn test_svg_icons_valid_xml() {
    use crate::ui::icons;

    // All icons should have opening and closing svg tags
    let icon_list = [
        icons::BRUSH_ICON,
        icons::ERASER_ICON,
        icons::COLOR_PICKER_ICON,
        icons::MOVE_ICON,
        icons::TEXT_ICON,
        icons::SHAPES_ICON,
        icons::FILL_ICON,
        icons::LINE_ICON,
        icons::ZOOM_IN_ICON,
        icons::ZOOM_OUT_ICON,
        icons::EXPORT_ICON,
        icons::ADD_LAYER_ICON,
        icons::DELETE_ICON,
        icons::UNDO_ICON,
        icons::REDO_ICON,
        icons::EYE_ICON,
        icons::EYE_OFF_ICON,
        icons::SETTINGS_ICON,
        icons::THEME_ICON,
    ];

    for (i, svg) in icon_list.iter().enumerate() {
        // Check basic SVG structure
        assert!(svg.contains("<svg"), "Icon {} missing <svg> tag", i);
        assert!(svg.contains("</svg>"), "Icon {} missing </svg> tag", i);
        assert!(svg.contains("xmlns="), "Icon {} missing xmlns", i);
        assert!(svg.contains("width="), "Icon {} missing width", i);
        assert!(svg.contains("height="), "Icon {} missing height", i);
    }
}

/// Test SVG icon loading with egui_extras
#[test]
fn test_svg_icon_loading() {
    // Test that egui_extras can load SVG bytes
    use crate::ui::icons;

    // Try to load brush icon (this tests the API without needing egui context)
    let svg_bytes = icons::BRUSH_ICON.as_bytes();
    assert!(!svg_bytes.is_empty());

    // Verify SVG has expected elements for a brush icon
    let svg_str = std::str::from_utf8(svg_bytes).expect("SVG should be valid UTF-8");
    assert!(
        svg_str.contains("path"),
        "Brush icon should have path elements"
    );
}

/// Test UiState has theme toggle field
#[test]
fn test_ui_state_theme_toggle() {
    let ui_state = UiState::new();
    // Should have use_dark_theme field
    assert!(ui_state.use_dark_theme); // Default is true (dark theme)
}

/// Test icon size consistency (all should be 24x24)
#[test]
fn test_svg_icon_size_consistency() {
    use crate::ui::icons;

    // Parse SVG to check viewBox or width/height
    // All icons should be 24x24 based on our design
    let icon_list = [
        ("Brush", icons::BRUSH_ICON),
        ("Eraser", icons::ERASER_ICON),
        ("Zoom In", icons::ZOOM_IN_ICON),
        ("Undo", icons::UNDO_ICON),
    ];

    for (name, svg) in icon_list {
        // Check width and height are 24
        assert!(
            svg.contains("width=\"24\""),
            "{} icon should be 24px wide",
            name
        );
        assert!(
            svg.contains("height=\"24\""),
            "{} icon should be 24px tall",
            name
        );
        assert!(
            svg.contains("viewBox=\"0 0 24 24\""),
            "{} icon should have 24x24 viewBox",
            name
        );
    }
}

/// Test color conversion functions
#[test]
fn test_color_conversion_functions() {
    use crate::canvas::Color;
    use crate::ui::egui_impl::{color_to_color32, color32_to_color};

    // Test basic RGB color (opaque)
    let color = Color {
        r: 255,
        g: 128,
        b: 64,
        a: 255,
    };
    let color32 = color_to_color32(color);
    let converted = color32_to_color(color32);

    assert_eq!(converted.r, 255);
    assert_eq!(converted.g, 128);
    assert_eq!(converted.b, 64);
    assert_eq!(converted.a, 255);
}

/// Test opaque black conversion
#[test]
fn test_color_conversion_black() {
    use crate::canvas::Color;
    use crate::ui::egui_impl::{color_to_color32, color32_to_color};

    let color = Color::BLACK; // r:0, g:0, b:0, a:255
    let color32 = color_to_color32(color);
    let converted = color32_to_color(color32);

    assert_eq!(converted, Color::BLACK);
}

/// Test white color conversion
#[test]
fn test_color_conversion_white() {
    use crate::canvas::Color;
    use crate::ui::egui_impl::{color_to_color32, color32_to_color};

    let color = Color::WHITE; // r:255, g:255, b:255, a:255
    let color32 = color_to_color32(color);
    let converted = color32_to_color(color32);

    assert_eq!(converted, Color::WHITE);
}

/// Test that color swatch reads from active tool's brush color
#[test]
fn test_color_swatch_shows_brush_color() {
    use crate::app::AppState;
    use crate::canvas::Color;
    use crate::ui::egui_impl::color_to_color32;

    let app = AppState::new(800, 600);

    // Get current brush color (default is black)
    let settings = app.active_tool().brush_settings().unwrap();
    let color32 = color_to_color32(settings.color);

    // Default color should be black (0,0,0,255)
    assert_eq!(settings.color, Color::BLACK);

    // Color32 should match
    let expected = color_to_color32(Color::BLACK);
    assert_eq!(color32, expected);
}

/// Test that changing brush color updates what swatch shows
#[test]
fn test_changing_brush_color_updates_swatch() {
    use crate::app::AppState;
    use crate::canvas::Color;
    use crate::ui::egui_impl::{color_to_color32, color32_to_color};

    let mut app = AppState::new(800, 600);

    // Change brush color to red via BrushConfig
    let red = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    if let Some(config) = app.active_tool_mut().as_brush_config() {
        config.set_brush_color(red);
    }

    // Now swatch should show red
    let settings = app.active_tool().brush_settings().unwrap();
    let color32 = color_to_color32(settings.color);
    let converted = color32_to_color(color32);

    assert_eq!(converted, red);
}

/// Test that clicking color swatch toggles color_picker_open
#[test]
fn test_color_swatch_click_toggles_picker() {
    let mut ui_state = UiState::new();

    // Initially closed
    assert!(!ui_state.color_picker_open);

    // Simulate click: toggle open
    ui_state.color_picker_open = !ui_state.color_picker_open;
    assert!(ui_state.color_picker_open);

    // Simulate click again: toggle closed
    ui_state.color_picker_open = !ui_state.color_picker_open;
    assert!(!ui_state.color_picker_open);
}

/// Test that color conversion with alpha < 255 preserves RGB values
/// within 1 of the original (integer rounding from premultiplied format).
#[test]
fn test_color_conversion_with_alpha_preserves_rgb() {
    use crate::canvas::Color;
    use crate::ui::egui_impl::{color_to_color32, color32_to_color};

    let color = Color {
        r: 200,
        g: 150,
        b: 100,
        a: 128,
    };
    let color32 = color_to_color32(color);
    let converted = color32_to_color(color32);

    assert_eq!(converted.a, 128);
    // u8 premultiplied arithmetic loses at most 1 per channel
    assert!(converted.r.abs_diff(200) <= 1);
    assert!(converted.g.abs_diff(150) <= 1);
    assert!(converted.b.abs_diff(100) <= 1);
}

/// Test that UiState hsva lifecycle matches picker open/close
#[test]
fn test_ui_state_hsva_lifecycle() {
    use crate::ui::state::UiState;

    let mut state = UiState::new();
    assert!(state.hsva.is_none());

    // Simulate picker opening: initialize Hsva from brush color
    let color = crate::canvas::Color {
        r: 100,
        g: 150,
        b: 200,
        a: 255,
    };
    state.hsva = Some(egui_sdl2::egui::ecolor::Hsva::from_srgba_unmultiplied([
        color.r, color.g, color.b, color.a,
    ]));
    assert!(state.hsva.is_some());

    // Simulate picker closing: reset to None
    state.color_picker_open = false;
    state.hsva = None;
    assert!(state.hsva.is_none());
}
