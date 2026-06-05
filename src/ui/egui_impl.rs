//! egui UI implementation based on Rancer UI.html design
//!
//! Implements the main interface: top bar, tool strip, bottom bar,
//! color picker, layer management, themes, and canvas area.

use crate::app::AppState;
use crate::tools::ToolType as AppToolType;
use crate::ui::UiState;
use egui_sdl2::egui::{self, Color32, Context, RichText, Stroke, TextureHandle};
use std::collections::HashMap;

/// Icon texture cache - pre-load all icons at startup
pub struct IconCache {
    textures: HashMap<&'static str, TextureHandle>,
}

impl IconCache {
    pub fn new(ctx: &Context) -> Self {
        let mut cache = Self {
            textures: HashMap::new(),
        };

        // Pre-load all SVG icons
        cache.load_icon(ctx, "brush", crate::ui::icons::BRUSH_ICON);
        cache.load_icon(ctx, "eraser", crate::ui::icons::ERASER_ICON);
        cache.load_icon(ctx, "zoom_in", crate::ui::icons::ZOOM_IN_ICON);
        cache.load_icon(ctx, "zoom_out", crate::ui::icons::ZOOM_OUT_ICON);
        cache.load_icon(ctx, "undo", crate::ui::icons::UNDO_ICON);
        cache.load_icon(ctx, "redo", crate::ui::icons::REDO_ICON);
        cache.load_icon(ctx, "add_layer", crate::ui::icons::ADD_LAYER_ICON);
        cache.load_icon(ctx, "settings", crate::ui::icons::SETTINGS_ICON);
        cache.load_icon(ctx, "theme", crate::ui::icons::THEME_ICON);
        cache.load_icon(ctx, "eye", crate::ui::icons::EYE_ICON);
        cache.load_icon(ctx, "eye_off", crate::ui::icons::EYE_OFF_ICON);

        cache
    }

    fn load_icon(&mut self, ctx: &Context, name: &'static str, svg_bytes: &'static str) {
        let options = resvg::usvg::Options::default();
        let color_image = egui_extras::image::load_svg_bytes(svg_bytes.as_bytes(), &options)
            .unwrap_or_else(|_| egui::ColorImage::new([20, 20], vec![egui::Color32::TRANSPARENT]));

        let handle = ctx.load_texture(name, color_image, egui::TextureOptions::default());

        self.textures.insert(name, handle);
    }

    pub fn get(&self, name: &str) -> Option<&TextureHandle> {
        self.textures.get(name)
    }
}

/// Theme colors for the UI
pub struct Theme {
    pub panel: Color32,
    pub panel_alt: Color32,
    pub accent: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub border: Color32,
    pub success: Color32,
    pub surface: Color32,
}

impl Theme {
    pub fn studio_dark() -> Self {
        Self {
            panel: Color32::from_rgb(36, 36, 38),
            panel_alt: Color32::from_rgb(48, 48, 50),
            accent: Color32::from_rgb(0, 153, 255),
            text: Color32::from_rgb(240, 240, 240),
            text_muted: Color32::from_rgb(160, 160, 160),
            border: Color32::from_rgb(64, 64, 66),
            success: Color32::from_rgb(34, 197, 94),
            surface: Color32::from_rgb(24, 24, 26),
        }
    }

    pub fn studio_light() -> Self {
        Self {
            panel: Color32::from_rgb(245, 245, 247),
            panel_alt: Color32::from_rgb(235, 235, 237),
            accent: Color32::from_rgb(0, 122, 204),
            text: Color32::from_rgb(20, 20, 20),
            text_muted: Color32::from_rgb(120, 120, 120),
            border: Color32::from_rgb(200, 200, 202),
            success: Color32::from_rgb(34, 197, 94),
            surface: Color32::from_rgb(255, 255, 255),
        }
    }
}

/// Convert our Color (RGBA u8) to egui::Color32
pub fn color_to_color32(color: crate::canvas::Color) -> Color32 {
    // Color32 stores RGBA as u32, with a() returning the alpha byte
    // from_rgba_unmultiplied expects values in 0-255 range
    Color32::from_rgba_premultiplied(
        (color.r as f32 * (color.a as f32 / 255.0)) as u8,
        (color.g as f32 * (color.a as f32 / 255.0)) as u8,
        (color.b as f32 * (color.a as f32 / 255.0)) as u8,
        color.a,
    )
}

/// Convert egui::Color32 to our Color (RGBA u8)
pub fn color32_to_color(c: Color32) -> crate::canvas::Color {
    // Color32 stores premultiplied alpha; un-premultiply to recover the original RGB
    let a = c.a();
    if a == 0 {
        return crate::canvas::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
    }
    crate::canvas::Color {
        r: (c.r() as u16 * 255 / a as u16) as u8,
        g: (c.g() as u16 * 255 / a as u16) as u8,
        b: (c.b() as u16 * 255 / a as u16) as u8,
        a,
    }
}

/// Show the main UI using egui
#[allow(clippy::field_reassign_with_default)]
pub fn show_ui(ctx: &Context, app: &mut AppState, ui_state: &mut UiState, icon_cache: &IconCache) {
    let theme = if ui_state.use_dark_theme {
        Theme::studio_dark()
    } else {
        Theme::studio_light()
    };

    // Apply theme to egui visuals
    let mut visuals = egui::Visuals::default();
    visuals.window_fill = theme.panel;
    visuals.panel_fill = theme.panel;
    visuals.faint_bg_color = theme.panel_alt;
    visuals.window_stroke = Stroke::new(1.0, theme.border);
    visuals.widgets.noninteractive.bg_fill = theme.panel_alt;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.border);
    visuals.widgets.inactive.bg_fill = theme.panel_alt;
    visuals.widgets.hovered.bg_fill = theme.accent.linear_multiply(0.2);
    visuals.widgets.active.bg_fill = theme.accent.linear_multiply(0.3);
    visuals.override_text_color = Some(theme.text);
    visuals.weak_text_color = Some(theme.text_muted);
    ctx.set_visuals(visuals);

    // Top bar
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // Title
            ui.label(
                RichText::new("Rancer")
                    .color(theme.text)
                    .size(16.0)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);

                // Theme toggle with SVG icon
                if let Some(texture) = icon_cache.get("theme") {
                    let button = egui::Button::image(texture)
                        .min_size(egui::vec2(32.0, 32.0))
                        .corner_radius(4.0)
                        .fill(Color32::TRANSPARENT);

                    if ui.add(button).clicked() {
                        ui_state.use_dark_theme = !ui_state.use_dark_theme;
                    }
                }

                // Settings with SVG icon
                if let Some(texture) = icon_cache.get("settings") {
                    let button = egui::Button::image(texture)
                        .min_size(egui::vec2(32.0, 32.0))
                        .corner_radius(4.0)
                        .fill(Color32::TRANSPARENT);

                    if ui.add(button).clicked() {
                        // TODO: open settings
                    }
                }

                ui.separator();

                // Redo with SVG icon
                if let Some(texture) = icon_cache.get("redo") {
                    let button = egui::Button::image(texture)
                        .min_size(egui::vec2(32.0, 32.0))
                        .corner_radius(4.0)
                        .fill(Color32::TRANSPARENT);

                    if ui.add(button).clicked() {
                        ui_state.redo(app);
                    }
                }

                // Undo with SVG icon
                if let Some(texture) = icon_cache.get("undo") {
                    let button = egui::Button::image(texture)
                        .min_size(egui::vec2(32.0, 32.0))
                        .corner_radius(4.0)
                        .fill(Color32::TRANSPARENT);

                    if ui.add(button).clicked() {
                        ui_state.undo(app);
                    }
                }

                ui.add_space(8.0);
            });
        });
    });

    // Left tool strip
    egui::SidePanel::left("tool_strip")
        .resizable(false)
        .default_width(48.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(8.0);

                // Brush button
                let is_brush_active =
                    ui_state.active_tool == AppToolType::Brush && !ui_state.eraser_mode;

                if let Some(texture) = icon_cache.get("brush") {
                    let button = egui::Button::image(texture)
                        .min_size(egui::vec2(36.0, 36.0))
                        .corner_radius(8.0)
                        .fill(if is_brush_active {
                            theme.accent.linear_multiply(0.2)
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(if is_brush_active {
                            Stroke::new(1.5, theme.accent)
                        } else {
                            Stroke::NONE
                        });

                    if ui.add(button).clicked() {
                        ui_state.set_tool(AppToolType::Brush);
                        ui_state.eraser_mode = false;
                        ui_state.apply_to_app(app);
                    }
                }

                // Eraser button (uses BrushTool with eraser_mode=true)
                let is_eraser_active =
                    ui_state.active_tool == AppToolType::Brush && ui_state.eraser_mode;

                if let Some(texture) = icon_cache.get("eraser") {
                    let button = egui::Button::image(texture)
                        .min_size(egui::vec2(36.0, 36.0))
                        .corner_radius(8.0)
                        .fill(if is_eraser_active {
                            theme.accent.linear_multiply(0.2)
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(if is_eraser_active {
                            Stroke::new(1.5, theme.accent)
                        } else {
                            Stroke::NONE
                        });

                    if ui.add(button).clicked() {
                        ui_state.set_tool(AppToolType::Brush);
                        ui_state.eraser_mode = true;
                        ui_state.apply_to_app(app);
                    }
                }
            });
        });

    // Bottom bar
    egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // Color swatch - shows current brush color
            let current_color = app
                .active_tool()
                .brush_settings()
                .map(|s| color_to_color32(s.color))
                .unwrap_or(egui::Color32::BLACK);

            if ui
                .add(
                    egui::Button::new("")
                        .min_size(egui::vec2(38.0, 38.0))
                        .corner_radius(9.0)
                        .fill(current_color)
                        .stroke(Stroke::new(1.5, theme.border)),
                )
                .clicked()
            {
                ui_state.color_picker_open = !ui_state.color_picker_open;
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Layer chips
            let layer_count = app.canvas().layer_count();
            for i in 0..layer_count {
                let is_active = app.canvas().active_layer() == i;
                let is_visible = app.canvas().layers()[i].visible;

                let bg = if is_active {
                    theme.accent.linear_multiply(0.15)
                } else {
                    theme.panel_alt
                };
                let stroke = if is_active {
                    Stroke::new(1.0, theme.accent.linear_multiply(0.3))
                } else {
                    Stroke::NONE
                };

                egui::Frame::NONE
                    .fill(bg)
                    .stroke(stroke)
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        ui.set_min_width(110.0);
                        ui.horizontal(|ui| {
                            let eye_icon = if is_visible { "eye" } else { "eye_off" };
                            if let Some(texture) = icon_cache.get(eye_icon)
                                && ui
                                    .add(
                                        egui::Button::image(texture)
                                            .min_size(egui::vec2(16.0, 16.0))
                                            .frame(false),
                                    )
                                    .clicked()
                            {
                                ui_state.toggle_layer_visibility(app, i);
                            }
                            if ui
                                .add(
                                    egui::Label::new(format!("Layer {}", i + 1))
                                        .sense(egui::Sense::click()),
                                )
                                .clicked()
                            {
                                let _ = app.canvas_mut().set_active_layer(i);
                            }
                        });
                    });
            }

            // Add layer button with SVG icon
            if let Some(texture) = icon_cache.get("add_layer") {
                let button = egui::Button::image(texture)
                    .min_size(egui::vec2(38.0, 38.0))
                    .corner_radius(8.0)
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.5, theme.border));

                if ui.add(button).clicked() {
                    ui_state.add_layer(app);
                }
            }

            // Zoom controls — right-aligned
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Zoom percentage
                let zoom_pct = (app.viewport().scale * 100.0) as u32;
                ui.label(
                    RichText::new(format!("{}%", zoom_pct))
                        .color(theme.text)
                        .size(13.0),
                );

                // Fit button
                if ui
                    .add(
                        egui::Button::new("Fit")
                            .min_size(egui::vec2(36.0, 24.0))
                            .corner_radius(4.0),
                    )
                    .clicked()
                {
                    app.viewport_mut().zoom_to_fit();
                }

                // 1:1 button
                if ui
                    .add(
                        egui::Button::new("1:1")
                            .min_size(egui::vec2(36.0, 24.0))
                            .corner_radius(4.0),
                    )
                    .clicked()
                {
                    app.viewport_mut().zoom_to_100();
                }

                // Zoom out
                if let Some(texture) = icon_cache.get("zoom_out")
                    && ui
                        .add(
                            egui::Button::image(texture)
                                .min_size(egui::vec2(20.0, 20.0))
                                .frame(false),
                        )
                        .clicked()
                {
                    app.viewport_mut().zoom_out();
                }

                // Zoom in
                if let Some(texture) = icon_cache.get("zoom_in")
                    && ui
                        .add(
                            egui::Button::image(texture)
                                .min_size(egui::vec2(20.0, 20.0))
                                .frame(false),
                        )
                        .clicked()
                {
                    app.viewport_mut().zoom_in();
                }

                ui.add_space(4.0);
            });
        });
    });

    // Color picker popup (above bottom bar)
    if ui_state.color_picker_open {
        // Initialize persistent Hsva from brush color when picker first opens.
        // The Hsva is kept across frames during editing to avoid lossy
        // premultiplied-alpha round-trips through Color32.
        let hsva = ui_state.hsva.get_or_insert_with(|| {
            let color = app
                .active_tool()
                .brush_settings()
                .map(|s| s.color)
                .unwrap_or(crate::canvas::Color::BLACK);
            egui::ecolor::Hsva::from_srgba_unmultiplied([color.r, color.g, color.b, color.a])
        });

        egui::Window::new("color_picker")
            .open(&mut ui_state.color_picker_open)
            .title_bar(false)
            .resizable(false)
            .default_pos(egui::pos2(50.0, ctx.content_rect().bottom() - 80.0 - 350.0))
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(theme.panel)
                    .stroke(Stroke::new(1.0, theme.border))
                    .corner_radius(8.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_max_width(280.0);

                        let changed = egui::widgets::color_picker::color_picker_hsva_2d(
                            ui,
                            hsva,
                            egui::widgets::color_picker::Alpha::OnlyBlend,
                        );

                        if changed {
                            let [r, g, b, a] = hsva.to_srgba_unmultiplied();
                            ui_state.pending_color = Some(crate::canvas::Color { r, g, b, a });
                        }
                    });
            });

        // Apply color change after popup renders
        if let Some(color) = ui_state.pending_color.take()
            && let Some(config) = app.active_tool_mut().as_brush_config()
        {
            config.set_brush_color(color);
        }
    } else {
        // Reset Hsva when picker closes to avoid stale state on reopen
        ui_state.hsva = None;
    }
}
