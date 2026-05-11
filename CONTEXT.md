# Rancer — Domain Context

## Overview

Rancer is a **high-performance digital art application** built in Rust. It supports raster-based painting with layers, multiple brush types, and a modern egui-based UI.

## Core Concepts

- **Canvas**: The main drawing surface, composed of multiple **Layers**
- **Layer**: A single raster layer with its own opacity, visibility, and content (`RasterImage`)
- **RasterImage**: Pixel buffer (RGBA) stored as a flat `Vec<u8>` with width/height
- **BrushTool**: Freehand painting tool with two modes:
  - **Paint mode**: Stamps brush dabs with color/size/opacity from `paint_settings`
  - **Eraser mode**: Erases to `canvas.background_color` using `eraser_settings`
  - Toggle via `is_eraser: bool` — same tool, different settings
- **AppState**: Owns the canvas, active tool, and undo/redo history
- **UiState**: Manages egui panel visibility, tool selection, eraser mode, color picker state, and theme

## Domain Language

| Term | Meaning |
|------|---------|
| Dab | A single brush stamp at (x, y) with size/opacity |
| Stamp | Synonym for Dab in BrushEngine context |
| Rasterize | Convert vector SVG to pixel buffer (egui texture) |
| Version | Monotonically-increasing u64 counter for canvas dirty-tracking |
| Active tool | The currently selected tool (Brush, Selection, etc.) |
| UI state | egui-specific state (panels, theme, tool selection, eraser mode) |
| Eraser mode | BrushTool state (`is_eraser=true`) — uses `eraser_settings` to erase pixels |

## Architecture

See `ARCHITECTURE.md` for module overview and data flow.
See `REDESIGN.md` for the SDL2 + OpenGL + egui migration plan.

## Current Status

- **Version**: 0.0.7
- **Phase**: Mid-redesign (SDL2 + egui + glow)
- **Tools implemented**: BrushTool (Paint + Eraser modes)
- **UI**: egui integration with SVG icons, theme toggle, layer management, tool strip, color picker popup
- **Tests**: 117 tests passing (unit + integration + TDD eraser + TDD color picker)
- **Eraser mode**: `BrushTool` with `is_eraser` toggle, separate `paint_settings`/`eraser_settings`
- **Color picker**: Popup above bottom bar using `egui::color_picker_color32()`, reads/writes `BrushTool::paint_settings.color`
