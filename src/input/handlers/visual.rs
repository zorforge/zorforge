// src/input/handlers/visual.rs
use std::io;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use crate::editor::Editor;
use crate::editor::mode::{Mode, ModeTrigger};

pub fn handle_visual_mode(editor: &mut Editor, key: KeyEvent) -> io::Result<()> {
    match key.code {
        // Mode transitions
        KeyCode::Esc => {
            editor.buffer.clear_visual();
            editor.set_mode(editor.mode.transition(ModeTrigger::Escape));
        }

        // Visual mode operations
        KeyCode::Char('y') => {
            // Yank selection and return to normal mode
            if let Some(text) = editor.buffer.get_selected_text() {
                editor.clipboard.yank(text);
            }
            editor.buffer.clear_visual();
            editor.set_mode(Mode::Normal);
        }
        KeyCode::Char('d') | KeyCode::Char('x') => {
            // Delete/cut selection and return to normal mode
            if let Some(text) = editor.buffer.get_selected_text() {
                editor.clipboard.yank(text); // Save to clipboard before deleting
                editor.buffer.delete_selection();
            }
            editor.buffer.clear_visual();
            editor.set_mode(Mode::Normal);
        }
        KeyCode::Char('c') => {
            // Change selection (delete and enter insert mode)
            if let Some(text) = editor.buffer.get_selected_text() {
                editor.clipboard.yank(text);
                editor.buffer.delete_selection();
            }
            editor.buffer.clear_visual();
            editor.set_mode(Mode::Insert(InsertVariant::Insert));
        }
        KeyCode::Char('>') => {
            // Indent selection
            editor.buffer.indent_selection(editor.config.tab_size);
        }
        KeyCode::Char('<') => {
            // De-indent selection
            editor.buffer.dedent_selection(editor.config.tab_size);
        }

        // Modern clipboard operations
        KeyCode::Char('c') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
            if let Some(text) = editor.buffer.get_selected_text() {
                editor.clipboard.yank(text);
            }
            editor.buffer.clear_visual();
            editor.set_mode(Mode::Normal);
        }
        KeyCode::Char('x') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
            if let Some(text) = editor.buffer.get_selected_text() {
                editor.clipboard.yank(text);
                editor.buffer.delete_selection();
            }
            editor.buffer.clear_visual();
            editor.set_mode(Mode::Normal);
        }
        KeyCode::Char('v') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
            editor.buffer.paste_over_selection();
            editor.buffer.clear_visual();
            editor.set_mode(Mode::Normal);
        }

        // Movement keys (Vim style)
        KeyCode::Char('h') => editor.buffer.move_cursor("left"),
        KeyCode::Char('j') => editor.buffer.move_cursor("down"),
        KeyCode::Char('k') => editor.buffer.move_cursor("up"),
        KeyCode::Char('l') => editor.buffer.move_cursor("right"),
        KeyCode::Char('w') => editor.buffer.move_word_forward(),
        KeyCode::Char('b') => editor.buffer.move_word_backward(),
        KeyCode::Char('0') | KeyCode::Char('^') => editor.buffer.move_cursor("line_start"),
        KeyCode::Char('$') => editor.buffer.move_cursor("line_end"),
        KeyCode::Char('g') if key.modifiers == KeyModifiers::NONE => editor.buffer.move_cursor("top"),
        KeyCode::Char('G') => editor.buffer.move_cursor("bottom"),
        
        // Movement keys (Modern)
        KeyCode::Left => editor.buffer.move_cursor("left"),
        KeyCode::Right => editor.buffer.move_cursor("right"),
        KeyCode::Up => editor.buffer.move_cursor("up"),
        KeyCode::Down => editor.buffer.move_cursor("down"),
        KeyCode::Home => editor.buffer.move_cursor("line_start"),
        KeyCode::End => editor.buffer.move_cursor("line_end"),
        KeyCode::PageUp => editor.buffer.move_page_up(),
        KeyCode::PageDown => editor.buffer.move_page_down(),

        // Search within selection
        KeyCode::Char('/') => {
            // Store the current selection bounds before entering search mode
            editor.buffer.store_visual_bounds();
            editor.set_mode(Mode::Command(CommandType::Search));
        }

        // Switch visual mode type (char, line, block)
        KeyCode::Char('v') if key.modifiers == KeyModifiers::NONE => {
            editor.buffer.toggle_visual_mode(VisualMode::Char);
        }
        KeyCode::Char('V') => {
            editor.buffer.toggle_visual_mode(VisualMode::Line);
        }
        KeyCode::Char('v') if key.modifiers == KeyModifiers::CONTROL => {
            editor.buffer.toggle_visual_mode(VisualMode::Block);
        }

        // Text object selection
        KeyCode::Char('i') => {
            // Wait for next character to determine text object
            editor.set_visual_object_mode(SelectionType::Inner);
        }
        KeyCode::Char('a') => {
            // Wait for next character to determine text object
            editor.set_visual_object_mode(SelectionType::Around);
        }

        _ => {}
    }
    Ok(())
}

// Handle text object selection after 'i' or 'a'
fn handle_text_object(editor: &mut Editor, c: char, selection_type: SelectionType) {
    match c {
        'w' => editor.buffer.select_word(selection_type),
        'p' => editor.buffer.select_paragraph(selection_type),
        '(' | ')' | 'b' => editor.buffer.select_parentheses(selection_type),
        '[' | ']' => editor.buffer.select_brackets(selection_type),
        '{' | '}' | 'B' => editor.buffer.select_braces(selection_type),
        '<' | '>' => editor.buffer.select_angle_brackets(selection_type),
        '\'' => editor.buffer.select_single_quotes(selection_type),
        '"' => editor.buffer.select_double_quotes(selection_type),
        '`' => editor.buffer.select_backticks(selection_type),
        _ => {}
    }
}