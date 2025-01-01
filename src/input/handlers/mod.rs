// src/input/handlers/mod.rs
mod command;
mod insert;
mod normal;
mod visual;

use std::io;
use crossterm::event::KeyEvent;
use crate::editor::Editor;

pub fn handle_input(editor: &mut Editor, key: KeyEvent) -> io::Result<()> {
    match editor.mode() {
        Mode::Normal => normal::handle_normal_mode(editor, key),
        Mode::Insert(_) => insert::handle_insert_mode(editor, key),
        Mode::Visual(_) => visual::handle_visual_mode(editor, key),
        Mode::Command(_) => command::handle_command_mode(editor, key),
    }
}