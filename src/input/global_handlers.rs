// src/input/global_handlers.rs
use std::io;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use crate::editor::Editor;
use crate::editor::mode::{Mode, ModeTrigger};

/// Global key handler for operations that should work across different modes
pub struct GlobalKeyHandler;

impl GlobalKeyHandler {
    /// Attempt to handle global key bindings
    /// Returns true if the key was handled, false otherwise
    pub fn handle(editor: &mut Editor, key: KeyEvent) -> io::Result<bool> {
        match (key.code, key.modifiers) {
            // Clipboard operations
            (KeyCode::Char('c'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                Self::handle_global_copy(editor)
            },
            (KeyCode::Char('x'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                Self::handle_global_cut(editor)
            },
            (KeyCode::Char('v'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                Self::handle_global_paste(editor)
            },

            // Undo/Redo operations
            (KeyCode::Char('z'), KeyModifiers::CONTROL) => {
                Self::handle_undo(editor)
            },
            (KeyCode::Char('z'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                Self::handle_redo(editor)
            },

            // Other global operations can be added here
            _ => Ok(false)
        }
    }

    /// Handle global copy operation
    fn handle_global_copy(editor: &mut Editor) -> io::Result<bool> {
        // Try to get selected text first, fallback to current line
        let text = editor.buffer.get_selected_text()
            .or_else(|| editor.buffer.get_current_line().cloned());
        
        if let Some(content) = text {
            editor.clipboard.yank(content);
        }
        
        Ok(true)
    }

    /// Handle global cut operation
    fn handle_global_cut(editor: &mut Editor) -> io::Result<bool> {
        // Try to get and delete selected text first, fallback to current line
        if editor.buffer.get_selected_text().is_some() {
            if let Some(text) = editor.buffer.get_selected_text() {
                editor.clipboard.yank(text);
                editor.buffer.delete_selection();
            }
            editor.buffer.clear_visual();
        } else {
            // Cut current line
            if let Some(line) = editor.buffer.get_current_line().cloned() {
                editor.clipboard.yank(line);
                editor.buffer.delete_line();
            }
        }
        
        Ok(true)
    }

    /// Handle global paste operation
    fn handle_global_paste(editor: &mut Editor) -> io::Result<bool> {
        // Check if there's a visual selection
        if editor.buffer.get_visual_selection().is_some() {
            editor.buffer.paste_over_selection();
            editor.buffer.clear_visual();
        } else {
            // Normal paste at cursor
            editor.buffer.paste();
        }
        
        Ok(true)
    }

    /// Handle global undo
    fn handle_undo(editor: &mut Editor) -> io::Result<bool> {
        if editor.mode.allows_undo() {
            editor.buffer.undo();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Handle global redo
    fn handle_redo(editor: &mut Editor) -> io::Result<bool> {
        if editor.mode.allows_undo() {
            editor.buffer.redo();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}