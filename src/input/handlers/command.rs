// src/input/handlers/command.rs
use std::io;
use std::path::PathBuf;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::editor::Editor;
use crate::editor::mode::{Mode, ModeTrigger, CommandType};

pub fn handle_command_mode(editor: &mut Editor, key: KeyEvent) -> io::Result<()> {
    match key.code {
        // Exit command mode
        KeyCode::Esc => {
            editor.set_mode(editor.mode.transition(ModeTrigger::Escape));
        }

        // Execute command
        KeyCode::Enter => {
            let cmd = editor.command_line_content();
            execute_command(editor, &cmd)?;
            editor.set_mode(editor.mode.transition(ModeTrigger::Enter));
        }

        // Basic editing
        KeyCode::Char(c) => {
            // Add character to command buffer
            if let Mode::Command(cmd_type) = editor.mode() {
                editor.append_to_command(c);
            }
        }

        KeyCode::Backspace => {
            editor.delete_from_command();
        }

        _ => (),
    }
    Ok(())
}

fn execute_command(editor: &mut Editor, cmd: &str) -> io::Result<()> {
    // Basic command implementation
    match cmd {
        "q" | "quit" => {
            if editor.has_unsaved_changes() {
                editor.show_message("No write since last change (add ! to override)");
            } else {
                // TODO: Implement proper exit
                std::process::exit(0);
            }
        }

        "q!" | "quit!" => {
            std::process::exit(0);
        }

        "w" | "write" => {
            editor.save_buffer()?;
        }

        "wq" => {
            editor.save_buffer()?;
            std::process::exit(0);
        }

        // Add more commands here as needed

        _ => {
            // Handle save-as command
            if cmd.starts_with("w ") || cmd.starts_with("write ") {
                let file_path = cmd.split_whitespace().nth(1)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No file specified"))?;
                editor.save_buffer_as(PathBuf::from(file_path))?;
                return Ok(());
            }

            // Handle edit command
            if cmd.starts_with("e ") || cmd.starts_with("edit ") {
                if editor.has_unsaved_changes() {
                    editor.show_message("No write since last change (add ! to override)");
                    return Ok(());
                }
                let file_path = cmd.split_whitespace().nth(1)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No file specified"))?;
                editor.open_file(&PathBuf::from(file_path))?;
                return Ok(());
            }

            editor.show_message(&format!("Unknown command: {}", cmd));
        }
    }
    Ok(())
}