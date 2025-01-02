// src/editor/mod.rs
pub mod buffer;
pub mod clipboard;
pub mod mode;
mod viewport;

// Re-export the types we need publicly
pub use buffer::{Buffer, SelectionType};
pub use clipboard::Clipboard;
pub use mode::{Mode, CommandType, InsertVariant, VisualVariant};

use crossterm::event::MouseButton;
use crate::config::EditorConfig;
use std::path::PathBuf;
use std::io;

pub struct Editor {
    pub buffer: Buffer,
    pub clipboard: Clipboard,
    pub mode: Mode,
    pub config: EditorConfig,
    is_readonly: bool,
    command_buffer: Option<String>,
    file_path: Option<PathBuf>,
    message: Option<String>,
}

impl Editor {
    pub fn new(config: EditorConfig) -> Self {
        Self {
            buffer: Buffer::new(),
            clipboard: Clipboard::new(),
            mode: Mode::Normal,
            config,
            is_readonly: false,
            command_buffer: None,
            file_path: None,
            message: None,
        }
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    // Update save_buffer to mark changes as saved
    pub fn save_buffer(&mut self) -> io::Result<()> {
        if let Some(path) = &self.file_path {
            let content = self.buffer.get_content()
                .join("\n");
            std::fs::write(path, content)?;
            self.buffer.mark_saved();  // Mark current state as saved
            self.show_message(&format!("Wrote {}", path.display()));
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "No file name specified (use :w <path>)",
            ))
        }
    }

    // Update has_unsaved_changes to use buffer's tracking
    pub fn has_unsaved_changes(&self) -> bool {
        self.buffer.has_unsaved_changes()
    }

    // Add method to save with a specific path
    pub fn save_buffer_as(&mut self, path: PathBuf) -> io::Result<()> {
        self.file_path = Some(path);
        self.save_buffer()
    }
    pub fn show_message(&mut self, msg: &str) {
        self.message = Some(msg.to_string());
    }

    pub fn get_message(&self) -> Option<&String> {
        self.message.as_ref()
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn file_info(&self) -> String {
        match &self.file_path {
            Some(path) => path.display().to_string(),
            None => String::from("[No Name]")
        }
    }

    pub fn command_line_content(&self) -> String {
        match &self.command_buffer {
            Some(buffer) => buffer.clone(),
            None => String::new(),
        }
    }

    pub fn handle_mouse_click(&mut self, col: usize, row: usize, _button: MouseButton) {
        self.buffer.set_cursor_position(row, col);
    }

    pub fn handle_mouse_drag(&mut self, col: usize, row: usize, _button: MouseButton) {
        // If we're not already in visual mode, enter it and mark selection start
        if !self.mode.is_visual() {
            self.buffer.start_visual();
            self.mode = Mode::Visual(VisualVariant::Char);
        }
        
        // Update cursor position which will update the selection end
        self.buffer.set_cursor_position(row, col);
    }

    pub fn scroll_up(&mut self) {
        self.buffer.move_page_up();
    }

    pub fn scroll_down(&mut self) {
        self.buffer.move_page_down();
    }

    pub fn set_visual_object_mode(&mut self, selection_type: SelectionType) {
        // Instead of directly accessing the field, we'll use a method
        self.buffer.set_selection_type(selection_type);
    }

    // Buffer access methods
    pub fn current_buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn current_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        self.buffer.get_cursor_position()
    }

    // File operations
    pub fn open_file(&mut self, path: &PathBuf) -> io::Result<()> {
        let contents = std::fs::read_to_string(path)?;
        self.buffer = Buffer::new();
        for line in contents.lines() {
            self.buffer.insert_at(self.buffer.line_count(), line.to_string());
        }
        Ok(())
    }

    pub fn set_readonly(&mut self, readonly: bool) {
        self.is_readonly = readonly;
    }

    pub fn is_readonly(&self) -> bool {
        self.is_readonly
    }

    pub fn cursor_position_info(&self) -> String {
        let (row, col) = self.cursor_position();
        format!("{}:{}", row + 1, col + 1)
    }

    pub fn append_to_command(&mut self, c: char) {
        if let Some(buffer) = &mut self.command_buffer {
            buffer.push(c);
        } else {
            self.command_buffer = Some(String::from(c));
        }
    }

    pub fn delete_from_command(&mut self) {
        if let Some(buffer) = &mut self.command_buffer {
            buffer.pop();
            if buffer.is_empty() {
                self.command_buffer = None;
            }
        }
    }

    pub fn clear_command(&mut self) {
        self.command_buffer = None;
    }
}