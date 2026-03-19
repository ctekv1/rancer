# Rancer v0.0.2 Release Notes

## 🎉 New Features & Improvements

### Brush Size Selection
- **New**: Added brush size selector with 5 different sizes (3px, 5px, 10px, 25px, 50px)
- **New**: Individual gray background boxes behind each brush size button for better visual distinction
- **Fixed**: Brush size now correctly reflects on canvas strokes
- **Fixed**: Brush size persists between strokes and during active drawing

### UI Improvements
- **Improved**: Brush size selector redesigned with horizontal layout
- **Improved**: Better visual feedback for brush size selection
- **Improved**: Individual button backgrounds for clearer UI separation

### Bug Fixes
- **Fixed**: Stroke width not changing based on selected brush size
- **Fixed**: Brush size parameter not being passed correctly through closures
- **Fixed**: Individual gray boxes not being drawn behind each button

## 🛠 Technical Changes
- Enhanced shared state management for brush size using `Rc<RefCell<f32>>`
- Improved mouse event handling for brush size selection
- Better integration between UI components and drawing system
- Added comprehensive debugging output for brush size operations

## 📋 Version History
- **v0.0.1**: Initial release with basic canvas and color selection
- **v0.0.2**: Added brush size selection and fixed UI issues

## 🚀 Getting Started
Run the application with:
```bash
cargo run
```

Use the brush size selector (below the color palette) to change stroke widths, then draw on the canvas to see the different brush sizes in action!