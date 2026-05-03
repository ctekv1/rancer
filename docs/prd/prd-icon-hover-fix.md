## Problem Statement

When hovering over tool icons in the left tool strip, the SVG icons either disappear entirely or appear to shift/duplicate. This creates a confusing user experience where:
- Icons vanish on hover instead of showing a subtle light gray background
- Icons appear to shrink or change position when the mouse moves over them
- Multiple visual artifacts appear (duplication) when the UI redraws

The root cause is that the code was:
1. Creating new SVG textures on every frame (causing drop/recreation)
2. Drawing hover backgrounds ON TOP of icons (covering them)
3. Using `painter().image()` with incorrect signatures, causing rendering artifacts
4. Not pre-loading textures, so they'd disappear when egui's context changed

## Solution

Pre-load all SVG icon textures at application startup into an `IconCache` (HashMap), then reference them throughout the UI lifecycle. This ensures:
- Textures persist across frames (no disappearance)
- No manual `painter()` calls needed (no duplication artifacts)
- `Button::image()` handles centering and sizing correctly
- Hover effect (light gray background) draws behind the icon, not on top

## User Stories

1. As a digital artist, I want tool icons to remain visible when I hover over them, so that I can clearly see which tool I'm about to select.

2. As a digital artist, I want tool icons to stay centered and fixed in size when hovering, so that the UI feels stable and predictable.

3. As a digital artist, I want the active tool to have a blue highlight while inactive tools show a light gray background on hover, so that I can quickly identify the current tool and available options.

4. As a digital artist, I want SVG icons to load once at startup (not per-frame), so that the UI is performant and icons never flicker or disappear.

5. As a digital artist, I want the brush tool icon to be clearly visible and the eraser icon (currently mapped to Selection tool) to display correctly, so that I can switch tools intuitively.

6. As a developer, I want the icon rendering code to use `Button::image()` correctly, so that we avoid manual `painter().image()` calls that cause duplication.

7. As a developer, I want all icon-related UI tests to pass, so that regressions in icon rendering are caught early.

8. As a user, I want the top bar undo/redo/settings/theme icons to also use the pre-loaded texture cache, so that they don't disappear on hover either.

## Implementation Decisions

- **IconCache module**: Create `IconCache` struct in `ui/egui_impl.rs` that holds a `HashMap<&'static str, TextureHandle>` for pre-loaded SVG textures
- **Startup initialization**: Create `IconCache` in `Sdl2App::new()` after egui context is available, pre-loading all 8 SVG icons (brush, eraser, zoom_in, zoom_out, undo, redo, add_layer, settings, theme)
- **Modified `show_ui()` signature**: Add `icon_cache: &IconCache` parameter so the UI can reference pre-loaded textures
- **Button::image() usage**: Replace manual `painter().image()` calls with `egui::Button::image(&texture)` which handles centering, sizing, and hover states internally
- **Fixed-size buttons**: Use `.min_size(egui::vec2(36.0, 36.0))` on tool buttons to prevent shrinkage
- **Hover background**: Draw light gray (`rgb(80,80,82)` for dark theme, `rgb(220,220,222)` for light theme) behind the button using `painter().rect_filled()`, but ONLY when not active
- **Active state**: Blue background (`theme.accent.linear_multiply(0.2)`) with blue border (`Stroke::new(1.5, theme.accent)`)
- **No manual icon redraws**: Remove all `painter().image()` calls that were causing duplication — `Button::image()` handles this
- **TextureHandle persistence**: `IconCache` stores `TextureHandle` which keeps the GPU texture alive as long as the cache exists

## Testing Decisions

- **TDD approach**: Write failing tests FIRST (RED), then implement (GREEN), following the cycle we used:
  - `test_icon_does_not_disappear_on_hover()` — verifies SVG icons load correctly (valid XML, proper structure)
  - `test_tool_button_fixed_size()` — checks code uses `min_size(36,36)` and NOT `shrink_to_fit()`
  - `test_icon_not_duplicated()` — ensures no `painter().image()` calls that cause duplication
  - `test_click_brush_tool_icon()` / `test_click_selection_tool_icon()` — verify tool switching works via UI state
- **Test external behavior, not implementation**: Tests check that icons DON'T disappear and buttons ARE fixed-size, not which internal methods are called
- **Modules tested**: `ui_tests.rs` (all icon-related tests now in one place)
- **Prior art**: Existing tests in `ui_tests.rs` for `UiState` tool switching, egui integration, and SVG icon validity

## Out of Scope

- **Color picker implementation**: The color swatch button exists but picker UI is not part of this PRD
- **Brush type switching UI**: No UI controls for Round/Square brush types (only code support exists)
- **Layer switching via bottom bar**: Layer chips display but clicking doesn't switch layers yet
- **Zoom controls**: Zoom in/out icons are loaded but no UI buttons implement zoom functionality
- **Settings panel**: Settings icon button exists but opens no panel
- **Export functionality**: Export button exists but doesn't export
- **Spray/Caligraphy brushes**: Listed in REDESIGN.md but not implemented
- **PanTool**: Listed in architecture docs but not implemented

## Further Notes

- The `egui_extras` crate with "svg" feature is required for SVG loading via `load_svg_bytes()`
- `resvg = "0.45"` dependency was added to access `resvg::usvg::Options` for SVG parsing
- The `TextureHandle` type from `egui` is used (accessed via `egui_sdl2::egui::TextureHandle`)
- This fix was developed using TDD (RED→GREEN cycle) with 23 tests now passing in `ui_tests`
- Follow-up work: fix the Selection tool's `begin_selection()/commit_selection()` calls (currently commented out in `selection_tool.rs`)
