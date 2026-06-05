//! Undo/redo commands for canvas operations
//!
//! Each command implements the `undo::Edit` trait, providing `edit()` and `undo()` methods.

use undo::Edit;

use crate::canvas::{Canvas, Layer};

/// Enum wrapping all canvas commands for use with a single Record
#[derive(Debug, Clone)]
pub enum CanvasCommand {
    AddLayer(AddLayer),
    RemoveLayer(RemoveLayer),
    ToggleVisibility(ToggleVisibility),
    SetOpacity(SetOpacity),
}

impl Edit for CanvasCommand {
    type Target = Canvas;
    type Output = Result<(), String>;

    fn edit(&mut self, target: &mut Canvas) -> Self::Output {
        match self {
            CanvasCommand::AddLayer(cmd) => cmd.edit(target),
            CanvasCommand::RemoveLayer(cmd) => cmd.edit(target),
            CanvasCommand::ToggleVisibility(cmd) => {
                cmd.edit(target);
                Ok(())
            }
            CanvasCommand::SetOpacity(cmd) => {
                cmd.edit(target);
                Ok(())
            }
        }
    }

    fn undo(&mut self, target: &mut Canvas) -> Self::Output {
        match self {
            CanvasCommand::AddLayer(cmd) => cmd.undo(target),
            CanvasCommand::RemoveLayer(cmd) => cmd.undo(target),
            CanvasCommand::ToggleVisibility(cmd) => {
                cmd.undo(target);
                Ok(())
            }
            CanvasCommand::SetOpacity(cmd) => {
                cmd.undo(target);
                Ok(())
            }
        }
    }
}

/// Command to add a new layer to the canvas
#[derive(Debug, Clone, Default)]
pub struct AddLayer {
    name: Option<String>,
    layer: Option<Layer>,
    insertion_index: Option<usize>,
}

impl AddLayer {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            layer: None,
            insertion_index: None,
        }
    }
}

impl Edit for AddLayer {
    type Target = Canvas;
    type Output = Result<(), String>;

    fn edit(&mut self, target: &mut Canvas) -> Self::Output {
        let idx = target.add_layer(self.name.clone())?;
        self.layer = Some(target.layers[idx].clone());
        self.insertion_index = Some(idx);
        Ok(())
    }

    fn undo(&mut self, target: &mut Canvas) -> Self::Output {
        if let Some(idx) = self.insertion_index {
            if idx < target.layers.len() {
                self.insertion_index = None;
                self.layer = None;
                target.remove_layer(idx)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

/// Command to remove a layer from the canvas
#[derive(Debug, Clone)]
pub struct RemoveLayer {
    index: usize,
    layer: Option<Layer>,
}

impl RemoveLayer {
    pub fn new(index: usize) -> Self {
        Self { index, layer: None }
    }
}

impl Edit for RemoveLayer {
    type Target = Canvas;
    type Output = Result<(), String>;

    fn edit(&mut self, target: &mut Canvas) -> Self::Output {
        self.layer = Some(target.layers[self.index].clone());
        target.remove_layer(self.index)
    }

    fn undo(&mut self, target: &mut Canvas) -> Self::Output {
        if let Some(layer) = self.layer.take() {
            target.layers.insert(self.index, layer);
            target.invalidate();
            Ok(())
        } else {
            Err("No layer to restore".to_string())
        }
    }
}

/// Command to toggle layer visibility
#[derive(Debug, Clone)]
pub struct ToggleVisibility {
    index: usize,
    was_visible: bool,
}

impl ToggleVisibility {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            was_visible: true,
        }
    }
}

impl Edit for ToggleVisibility {
    type Target = Canvas;
    type Output = ();

    fn edit(&mut self, target: &mut Canvas) -> Self::Output {
        if self.index < target.layers.len() {
            self.was_visible = target.layers[self.index].visible;
        }
        let _ = target.toggle_layer_visibility(self.index);
    }

    fn undo(&mut self, target: &mut Canvas) -> Self::Output {
        if self.index < target.layers.len() {
            target.layers[self.index].visible = self.was_visible;
            target.invalidate();
        }
    }
}

/// Command to set layer opacity
#[derive(Debug, Clone)]
pub struct SetOpacity {
    index: usize,
    old_opacity: f32,
    new_opacity: f32,
}

impl SetOpacity {
    pub fn new(index: usize, opacity: f32) -> Self {
        Self {
            index,
            old_opacity: 1.0,
            new_opacity: opacity,
        }
    }
}

impl Edit for SetOpacity {
    type Target = Canvas;
    type Output = ();

    fn edit(&mut self, target: &mut Canvas) -> Self::Output {
        if self.index < target.layers.len() {
            self.old_opacity = target.layers[self.index].opacity;
        }
        let _ = target.set_layer_opacity(self.index, self.new_opacity);
    }

    fn undo(&mut self, target: &mut Canvas) -> Self::Output {
        if self.index < target.layers.len() {
            target.layers[self.index].opacity = self.old_opacity;
            target.invalidate();
        }
    }
}
