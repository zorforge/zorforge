// src/input/handlers/normal.rs
use std::io;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use crate::editor::Editor;
use crate::editor::mode::{Mode, ModeTrigger, InsertVariant, CommandType};
use crate::editor::buffer::{Buffer, VisualMode};

pub fn handle_normal_mode(editor: &mut Editor, key: KeyEvent) -> io::Result<()> {
    match key.code {
        // Mode transitions
        KeyCode::Char('i') => {
            editor.set_mode(editor.mode.transition(ModeTrigger::InsertNormal));
        }
        KeyCode::Char('a') => {
            editor.buffer.prepare_append();
            editor.set_mode(editor.mode.transition(ModeTrigger::InsertAppend));
        }
        KeyCode::Char('A') => {
            editor.buffer.prepare_append_end_of_line();
            editor.set_mode(editor.mode.transition(ModeTrigger::InsertAppendEnd));
        }
        KeyCode::Char('I') => {
            editor.buffer.prepare_insert_start_of_line();
            editor.set_mode(editor.mode.transition(ModeTrigger::InsertLineStart));
        }
        KeyCode::Char('o') => {
            editor.buffer.insert_line_below();
            editor.set_mode(editor.mode.transition(ModeTrigger::InsertLineBelow));
        }
        KeyCode::Char('O') => {
            editor.buffer.insert_line_above();
            editor.set_mode(editor.mode.transition(ModeTrigger::InsertLineAbove));
        }
        KeyCode::Char('R') => {
            editor.set_mode(editor.mode.transition(ModeTrigger::InsertReplace));
        }
        KeyCode::Char('v') => {
            editor.buffer.start_visual();
            editor.set_mode(editor.mode.transition(ModeTrigger::VisualChar));
        }
        KeyCode::Char('V') => {
            editor.buffer.start_visual();
            editor.set_mode(editor.mode.transition(ModeTrigger::VisualLine));
        }
        KeyCode::Char(':') => {
            editor.set_mode(editor.mode.transition(ModeTrigger::CommandMode));
        }
        KeyCode::Char('/') => {
            editor.set_mode(editor.mode.transition(ModeTrigger::SearchForward));
        }
        KeyCode::Char('?') => {
            editor.set_mode(editor.mode.transition(ModeTrigger::SearchBackward));
        }

        // Undo/Redo
        KeyCode::Char('u') if editor.mode.allows_undo() => {
            editor.buffer.undo();
        }
        KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL && editor.mode.allows_undo() => {
            editor.buffer.redo();
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

        // Movement keys (Modern)
        KeyCode::Left => editor.buffer.move_cursor("left"),
        KeyCode::Right => editor.buffer.move_cursor("right"),
        KeyCode::Up => editor.buffer.move_cursor("up"),
        KeyCode::Down => editor.buffer.move_cursor("down"),
        KeyCode::Home => editor.buffer.move_cursor("line_start"),
        KeyCode::End => editor.buffer.move_cursor("line_end"),
        KeyCode::PageUp => editor.buffer.move_page_up(),
        KeyCode::PageDown => editor.buffer.move_page_down(),

        // Clipboard operations
        KeyCode::Char('y') => editor.buffer.yank(),
        KeyCode::Char('p') => editor.buffer.paste(),
        KeyCode::Char('c') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
            editor.buffer.yank()
        }
        KeyCode::Char('v') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
            editor.buffer.paste()
        }

        // Cut/Delete operations
        KeyCode::Char('x') if editor.mode.allows_cut() => {
            editor.buffer.cut_char();
        }
        KeyCode::Delete if editor.mode.allows_deletion() => {
            editor.buffer.delete_char_forward();
        }
        KeyCode::Char('d') if editor.mode.allows_deletion() => {
            editor.buffer.delete_line();
        }

        _ => {}
    }
    Ok(())
}