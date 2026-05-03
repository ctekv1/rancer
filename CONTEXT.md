# Rancer — Domain Context

## Overview

Rancer is a **high-performance digital art application** built in Rust. It supports raster-based painting with layers, multiple brush types, and a modern egui-based UI.

## Core Concepts

- **Canvas**: The main drawing surface, composed of multiple **Layers**
- **Layer**: A single raster layer with its own opacity, visibility, and content (`RasterImage`)
- **RasterImage**: Pixel buffer (RGBA) stored as a flat `Vec<u8>` with width/height
- **BrushTool**: Freehand painting tool using dab stamping (Round/Square brushes)
- **SelectionTool**: Pixel-region selection with move support
- **AppState**: Owns the canvas, active tool, and undo/redo history
- **UiState**: Manages egui panel visibility, tool selection, and theme

## Domain Language

| Term | Meaning |
|------|---------|
| Dab | A single brush stamp at (x, y) with size/opacity |
| Stamp | Synonym for Dab in BrushEngine context |
| Rasterize | Convert vector SVG to pixel buffer (egui texture) |
| Version | Monotonically-increasing u64 counter for canvas dirty-tracking |
| Active tool | The currently selected tool (Brush, Selection, etc.) |
| UI state | egui-specific state (panels, theme, tool selection) |

## Architecture

See `ARCHITECTURE.md` for module overview and data flow.
See `REDESIGN.md` for the SDL2 + OpenGL + egui migration plan.

## Current Status

- **Version**: 0.0.7
- **Phase**: Mid-redesign (SDL2 + OpenGL + egui)
- **Tools implemented**: BrushTool (Round/Square), SelectionTool (stubbed)
- **UI**: egui integration with SVG icons, theme toggle, layer management
- **Tests**: 123+ tests passing (unit + integration)
