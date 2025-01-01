// src/main.rs
use std::{
    io::{self, stdout},
    path::PathBuf,
    time::Duration,
};
use crossterm::{
    event::{self, Event, KeyEvent},
    terminal::{enable_raw_mode, disable_raw_mode},
    ExecutableCommand,
};
use clap::Parser;

mod editor;
mod ui;
mod input;
mod config;
mod utils;
mod splash;
mod cli;

use editor::{Editor, Mode};
use ui::Renderer;
use input::handle_input;
use config::EditorConfig;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to open
    #[arg(name = "FILE")]
    file: Option<PathBuf>,

    /// Config file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Start in read-only mode
    #[arg(short, long)]
    readonly: bool,
}

fn main() -> io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    if let Ok(log_path) = std::env::var("ZORFORGE_LOG") {
        simple_logging::log_to_file(
            log_path,
            log::LevelFilter::Info,
        ).expect("Failed to initialize logging");
    }

    // Load configuration
    let config = match &args.config {
        Some(path) => EditorConfig::load_from_file(path),
        None => EditorConfig::load_default(),
    }.unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}", e);
        EditorConfig::default()
    });

    // Initialize editor
    let mut editor = Editor::new(config);
    
    // Load initial file if specified
    if let Some(path) = args.file {
        if let Err(e) = editor.open_file(&path) {
            eprintln!("Error opening file: {}", e);
            return Ok(());
        }
        if args.readonly {
            editor.set_readonly(true);
        }
    }

    // Initialize renderer
    let mut renderer = Renderer::new()?;

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(crossterm::terminal::EnterAlternateScreen)?;

    // Main event loop
    run_event_loop(&mut editor, &mut renderer)?;

    // Cleanup
    cleanup()?;

    Ok(())
}

fn run_event_loop(editor: &mut Editor, renderer: &mut Renderer) -> io::Result<()> {
    let mut last_render = std::time::Instant::now();
    let frame_duration = Duration::from_millis(16); // ~60 FPS

    loop {
        // Handle input events
        if event::poll(Duration::from_millis(1))? {
            match event::read()? {
                Event::Key(key) => {
                    if !handle_key_event(editor, key)? {
                        break;
                    }
                }
                Event::Resize(width, height) => {
                    renderer.resize(width, height);
                }
                Event::Mouse(event) => {
                    handle_mouse_event(editor, event);
                }
                _ => {}
            }
        }

        // Throttle rendering to target frame rate
        let now = std::time::Instant::now();
        if now.duration_since(last_render) >= frame_duration {
            renderer.render(&mut stdout(), editor)?;
            last_render = now;
        }
    }

    Ok(())
}

fn handle_key_event(editor: &mut Editor, key: KeyEvent) -> io::Result<bool> {
    match editor.mode() {
        Mode::Normal => {
            // Check for quit command
            if key.matches_ctrl_key('q') {
                if editor.has_unsaved_changes() {
                    editor.show_message("Warning: Unsaved changes. Use :q! to force quit.");
                    return Ok(true);
                }
                return Ok(false);
            }
        }
        Mode::Command(_) => {
            // Handle force quit in command mode
            if editor.command_line_content() == "q!" {
                return Ok(false);
            }
        }
        _ => {}
    }

    // Handle all other input
    handle_input(editor, key)?;
    Ok(true)
}

fn handle_mouse_event(editor: &mut Editor, event: event::MouseEvent) {
    use crossterm::event::MouseEventKind::*;

    match event.kind {
        Down(button) => {
            editor.handle_mouse_click(
                event.column as usize,
                event.row as usize,
                button
            );
        }
        Drag(button) => {
            editor.handle_mouse_drag(
                event.column as usize,
                event.row as usize,
                button
            );
        }
        ScrollDown => {
            editor.scroll_down();
        }
        ScrollUp => {
            editor.scroll_up();
        }
        _ => {}
    }
}

fn cleanup() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

trait KeyEventExt {
    fn matches_ctrl_key(&self, c: char) -> bool;
}

impl KeyEventExt for KeyEvent {
    fn matches_ctrl_key(&self, c: char) -> bool {
        use crossterm::event::{KeyModifiers, KeyCode};
        
        matches!(
            (self.modifiers, self.code),
            (KeyModifiers::CONTROL, KeyCode::Char(k)) if k == c
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_ctrl_matching() {
        use crossterm::event::{KeyCode, KeyModifiers};

        let key = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            kind: event::KeyEventKind::Press,
            state: event::KeyEventState::NONE,
        };

        assert!(key.matches_ctrl_key('q'));
        assert!(!key.matches_ctrl_key('w'));
    }
}