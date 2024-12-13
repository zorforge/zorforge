// src/input/handlers/normal.rs
use crossterm::event::{KeyCode, KeyModifiers};
use crate::editor::Editor;
use crate::editor::mode::{Mode, ModeTrigger};

pub fn handle_normal_mode(editor: &mut Editor, key: KeyEvent) -> io::Result<()> {
    match key.code {
        // Mode transitions
        KeyCode::Char('i') => {
            editor.set_mode(Mode::Insert(InsertVariant::Insert));
        }
        KeyCode::Char('a') => {
            editor.buffer.prepare_append();
            editor.set_mode(Mode::Insert(InsertVariant::Append));
        }
        KeyCode::Char('A') => {
            editor.buffer.prepare_append_end_of_line();
            editor.set_mode(Mode::Insert(InsertVariant::AppendEnd));
        }
        KeyCode::Char('I') => {
            editor.buffer.prepare_insert_start_of_line();
            editor.set_mode(Mode::Insert(InsertVariant::LineStart));
        }
        KeyCode::Char('o') => {
            editor.buffer.insert_line_below();
            editor.set_mode(Mode::Insert(InsertVariant::LineBelow));
        }
        KeyCode::Char('O') => {
            editor.buffer.insert_line_above();
            editor.set_mode(Mode::Insert(InsertVariant::LineAbove));
        }
        KeyCode::Char('R') => {
            editor.set_mode(Mode::Insert(InsertVariant::Replace));
        }
        KeyCode::Char('v') => {
            editor.buffer.start_visual();
            editor.set_mode(Mode::Visual);
        }
        KeyCode::Char(':') => {
            editor.set_mode(Mode::Command(CommandType::Regular));
        }
        KeyCode::Char('/') => {
            editor.set_mode(Mode::Command(CommandType::Search));
        }
        KeyCode::Char('u') => {
            if editor.mode.allows_undo() {
                editor.buffer.undo();
            }
        }
        KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
            if editor.mode.allows_undo() {
                editor.buffer.redo();
            }
        }

        // Movement keys (Vim style)
        KeyCode::Char('h') => editor.buffer.move_cursor("left"),
        KeyCode::Char('j') => editor.buffer.move_cursor("down"),
        KeyCode::Char('k') => editor.buffer.move_cursor("up"),
        KeyCode::Char('l') => editor.buffer.move_cursor("right"),
        KeyCode::Char('0') | KeyCode::Char('^') => editor.buffer.move_cursor("line_start"),
        KeyCode::Char('$') => editor.buffer.move_cursor("line_end"),
        KeyCode::Char('g') if key.modifiers == KeyModifiers::NONE => editor.buffer.move_cursor("top"),
        KeyCode::Char('G') => editor.buffer.move_cursor("bottom"),

        // Movement keys (arrows, home, end)
        KeyCode::Left => editor.buffer.move_cursor("left"),
        KeyCode::Right => editor.buffer.move_cursor("right"),
        KeyCode::Up => editor.buffer.move_cursor("up"),
        KeyCode::Down => editor.buffer.move_cursor("down"),
        KeyCode::Home => editor.buffer.move_cursor("line_start"),
        KeyCode::End => editor.buffer.move_cursor("line_end"),

        // Clipboard operations
        KeyCode::Char('y') => editor.buffer.yank_line(),
        KeyCode::Char('p') => editor.buffer.paste(),
        KeyCode::Char('c') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
            editor.buffer.yank_line()
        }
        KeyCode::Char('v') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
            editor.buffer.paste()
        }

        KeyCode::Char('x') => {
            editor.buffer.cut_char()
        }

        // Deletion operations
        KeyCode::Delete => {
            editor.buffer.delete_char()
        }
        KeyCode::Char('d') => editor.buffer.delete_line(),

        _ => {}
    }
    Ok(())
}