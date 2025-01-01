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
}

impl Editor {
    pub fn new(config: EditorConfig) -> Self {
        Self {
            buffer: Buffer::new(),
            clipboard: Clipboard::new(),
            mode: Mode::Normal,
            config,
            is_readonly: false,
        }
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn has_unsaved_changes(&self) -> bool {
        // TODO: Implement actual change tracking
        false
    }

    pub fn show_message(&mut self, msg: &str) {
        // TODO: Implement message display
        println!("{}", msg);
    }

    pub fn command_line_content(&self) -> String {
        // TODO: Implement command line content
        String::new()
    }

    pub fn handle_mouse_click(&mut self, col: usize, row: usize, _button: MouseButton) {
        self.buffer.set_cursor_position(row, col);
    }

    pub fn handle_mouse_drag(&mut self, col: usize, row: usize, _button: MouseButton) {
        // TODO: Implement drag selection
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

    pub fn file_info(&self) -> String {
        // TODO: Implement file info
        String::from("[No Name]")
    }
}