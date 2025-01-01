// src/input/handlers/insert.rs
use std::io;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::editor::Editor;
use crate::editor::mode::{Mode, ModeTrigger, InsertVariant};

pub struct InsertHandler;

impl InsertHandler {
    /// Handler keypress event in insert mode
    pub fn handle(editor: &mut Editor, key: KeyEvent) -> io::Result<()> {
        match key.code {
            // Mode Transitions
            KeyCode::Esc => {
                // Move cursor back one space when exiting insert mode
                // (vim behavior: cursor should end up on last insert character)
                if let Some(line) = editor.buffer.get_current_line() {
                    if !line.is_empty() && editor.buffer.get_cursor_position().1 > 0 {
                        editor.buffer.move_cursor("left");
                    }
                }
                editor.set_mode(editor.mode.transition(ModeTrigger::Escape));
            }

            // Character input handling
            KeyCode::Char(c) => {
                if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) {
                    match c {
                        // Modern clipboard operations
                        'C' | 'c' => {
                            if let Some(line) = editor.buffer.get_current_line() {
                                editor.clipboard.yank(line.to_string());
                            }
                        }
                        'V' | 'v' => {
                            if let Some(line) = editor.clipboard.peek() {
                                editor.buffer.paste_at_cursor(content);
                            }
                        }
                        _ => (),
                    }
                } else if key.modifiers == KeyModifiers::CONTROL {
                    match c {
                        // Standard Vim ctrl shortcuts in insert mode
                        'w' => { // Delete word before cursor
                            editor.buffer.delete_word_backward();
                        }
                        'u' => { // Delete to start of line
                            editor.buffer.delete_to_line_start();
                        }
                        'h' => { // Delete character before cursor (same as backspace)
                            editor.buffer.delete_char();
                        }
                        'j' | 'm' => { // New line (same as Enter)
                            editor.buffer.insert_newline_auto_indent();
                        }
                        't' => { // Indent one shiftwidth
                            editor.buffer.indent_line(editor.config.tab_size);
                        }
                        'd' => { // De-indent one shiftwidth
                            editor.buffer.dedent_line(editor.config.tab_size);    
                        }
                        _ => (), 
                    }
                } else {
                    // Normal character insertion
                    match editor.mode {
                        Mode::Insert(InsertVariant::Replace) => {
                            editor.buffer.insert_char_replace(c);
                        }
                        _ => {
                            editor.buffer.insert_char(c);
                        }
                    }
                }
            }

            // Special Keys
            KeyCode::Enter => {
                editor.buffer.insert_newline_auto_indent();
            }
            KeyCode::Tab => {
                if key.modifiers == KeyModifiers::SHIFT {
                    editor.buffer.dedent_line(editor.config.tab_size);
                } else {
                    editor.buffer.indent_line(editor.config.tab_size);
                }
            }
            KeyCode::Backspace => {
                editor.buffer.delete_char();
            }
            KeyCode::Delete => {
                editor.buffer.delete_char_forward();
            }

            // Cursor movement
            KeyCode::Left => {
                if key.modifiers == KeyModifiers::CONTROL {
                    editor.buffer.move_word_backward();
                } else {
                    editor.buffer.move_cursor("left");
                }
            }
            KeyCode::Right => {
                if key.modifiers == KeyModifiers::CONTROL {
                    editor.buffer.move_word_forward();
                } else {
                    editor.buffer.move_cursor("right");
                }
            }
            KeyCode::Up => {
                editor.buffer.move_cursor("up");
            }
            KeyCode::Down => {
                editor.buffer.move_cursor("down");
            }
            KeyCode::Home => {
                if key.modifiers == KeyModifiers::CONTROL {
                    editor.buffer.move_cursor("file_start");
                } else {
                    editor.buffer.move_cursor("line_start");
                }
            }
            KeyCode::End => {
                if key.modifiers == KeyModifiers::CONTROL {
                    editor.buffer.move_cursor("file_end");
                } else {
                    editor.buffer.move_cursor("line_end");
                }
            }
            KeyCode::PageUp => {
                editor.buffer.move_page_up();
            }
            KeyCode::PageDown => {
                editor.buffer.move_page_down();
            }
            
            _ => (),
        }
        Ok(())
    }
}

impl Editor {
    /// Extend Editor with helper methods needed by insert mode
    fn indent_line(&mut self, size: usize) {
        let spaces = " ".repeat(size);
        self.buffer.insert_at_cursor(&spaces);
    }

    fn dedent_line(&mut self, size: usize) {
        let current_pos = self.buffer.get_cursor_position();
        if let Some(line) = self.buffer.get_current_line() {
            let whitespace_count = line.chars()
                .take_while(|c| c.is_whitespace())
                .count();
            let remove_count = whitespace_count.min(size);
            if remove_count > 0 {
                for _ in 0..remove_count {
                    self.buffer.delete_char();
                }
                // Restore cursor position if it wasn't at the start
                if current_pos.1 > remove_count {
                    self.buffer.set_cursor_position(current_pos.0, current_pos.1 - remove_count);
                }
            }
        }
    }

    fn delete_word_backward(&mut self) {
        while let Some(c) = self.buffer.get_char_before_cursor() {
            if !c.is_alphanumeric() && c != '_' {
                self.buffer.delete_char();
                break;
            }
            self.buffer.delete_char();
        }
    }

    fn delete_to_line_start(&mut self) {
        while self.buffer.get_cursor_position().1 > 0 {
            self.buffer.delete_char();
        }
    }
}