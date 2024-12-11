// use crossterm::event::{self, Event, KeyCode};
// use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
// use std::io;
// use tui::backend::CrosstermBackend;
// use tui::layout::{Constraint, Direction, Layout};
// use tui::widgets::{Block, Borders};
// use tui::Terminal;
//
// fn main() -> Result<(), io::Error> {
//     // Enable raw mode
//     enable_raw_mode()?;
//     let mut stdout = io::stdout();
//     let backend = CrosstermBackend::new(&mut stdout);
//     let mut terminal = Terminal::new(backend)?;
//
//     // Main application loop
//     loop {
//         terminal.draw(|frame| {
//             // let size = frame.size();
//             //
//             // // Layout: A block with borders
//             // let block = Block::default().title("Zorforge").borders(Borders::ALL);
//             //
//             // frame.render_widget(block, size);
//             let chunks = Layout::default()
//                 .direction(Direction::Vertical)
//                 .constraints(
//                     [
//                         Constraint::Percentage(75), // Editor
//                         Constraint::Percentage(5),  // Status bar
//                         Constraint::Percentage(20), // Command input
//                     ]
//                     .as_ref(),
//                 )
//                 .split(frame.size());
//
//             // Editor block
//             let editor = Block::default().title("Forge").borders(Borders::ALL);
//             frame.render_widget(editor, chunks[0]);
//
//             // Status bar block
//             let status_bar = Block::default().title("Status").borders(Borders::ALL);
//             frame.render_widget(status_bar, chunks[1]);
//
//             // Command input block
//             let command_input = Block::default().title("Terminal").borders(Borders::ALL);
//             frame.render_widget(command_input, chunks[2]);
//         })?;
//
//         // Handle input
//         if let Event::Key(key) = event::read()? {
//             if key.code == KeyCode::Char('q') {
//                 break; // Exit on 'q'
//             }
//         }
//     }
//
//     disable_raw_mode()
// }
//
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::collections::VecDeque;
use std::io;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Terminal;

#[derive(Debug, PartialEq)]
enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[derive(Debug)]
struct Editor {
    clipboard: VecDeque<String>,
    cursor_position: (usize, usize),      // (row, column)
    text_buffer: Vec<String>,             // Lines of text in the editor
    visual_start: Option<(usize, usize)>, // Start of visual selection
    tab_size: usize,                      // Default tab size
}

impl Editor {
    fn new() -> Self {
        Self {
            clipboard: VecDeque::new(),
            cursor_position: (0, 0),
            text_buffer: vec![String::new()],
            visual_start: None,
            tab_size: 4,
        }
    }

    fn render_with_cursor(&self, mode: &Mode) -> Vec<String> {
        self.text_buffer
            .iter()
            .enumerate()
            .map(|(row, line)| {
                let mut displayed_line = line.clone();
                if row == self.cursor_position.0 {
                    let cursor_char = match mode {
                        Mode::Insert => '_', // Thin cursor
                        _ => '█',            // Thick cursor
                    };

                    // Ensure safe boundary handling for inserting cursor
                    let char_idx = displayed_line
                        .char_indices()
                        .nth(self.cursor_position.1)
                        .map(|(idx, _)| idx)
                        .unwrap_or(displayed_line.len());

                    if char_idx < displayed_line.len() {
                        displayed_line
                            .replace_range(char_idx..char_idx + 1, &cursor_char.to_string());
                    } else {
                        displayed_line.push(cursor_char);
                    }
                }

                // Add line numbers to the beginning of each line
                format!("{:4} | {}", row + 1, displayed_line)
            })
            .collect()
    }

    fn move_cursor(&mut self, direction: &str) {
        match direction {
            "left" => {
                if self.cursor_position.1 > 0 {
                    self.cursor_position.1 -= 1;
                }
            }
            "right" => {
                if self.cursor_position.1 < self.text_buffer[self.cursor_position.0].len() {
                    self.cursor_position.1 += 1;
                }
            }
            "up" => {
                if self.cursor_position.0 > 0 {
                    self.cursor_position.0 -= 1;
                    self.cursor_position.1 = self.text_buffer[self.cursor_position.0]
                        .len()
                        .min(self.cursor_position.1);
                }
            }
            "down" => {
                if self.cursor_position.0 + 1 < self.text_buffer.len() {
                    self.cursor_position.0 += 1;
                    self.cursor_position.1 = self.text_buffer[self.cursor_position.0]
                        .len()
                        .min(self.cursor_position.1);
                }
            }
            "top" => {
                self.cursor_position.0 = 0;
                self.cursor_position.1 = 0;
            }
            "bottom" => {
                self.cursor_position.0 = self.text_buffer.len().saturating_sub(1);
                self.cursor_position.1 = 0;
            }
            _ => {}
        }
    }

    fn insert_text(&mut self, c: char) {
        if c == '\t' {
            // Replace tab character with spaces equivalent to tab_size
            let spaces = " ".repeat(self.tab_size);
            self.text_buffer[self.cursor_position.0].insert_str(self.cursor_position.1, &spaces);
            self.cursor_position.1 += self.tab_size;
        } else {
            self.text_buffer[self.cursor_position.0].insert(self.cursor_position.1, c);
            self.cursor_position.1 += 1;
        }
    }

    fn delete_text(&mut self) {
        if self.cursor_position.1 > 0 {
            self.text_buffer[self.cursor_position.0].remove(self.cursor_position.1 - 1);
            self.cursor_position.1 -= 1;
        } else if self.cursor_position.0 > 0 {
            let current_line = self.text_buffer.remove(self.cursor_position.0);
            self.cursor_position.0 -= 1;
            self.cursor_position.1 = self.text_buffer[self.cursor_position.0].len();
            self.text_buffer[self.cursor_position.0].push_str(&current_line);
        }
    }

    fn insert_newline(&mut self) {
        let current_line =
            self.text_buffer[self.cursor_position.0].split_off(self.cursor_position.1);
        self.text_buffer
            .insert(self.cursor_position.0 + 1, current_line);
        self.cursor_position.0 += 1;
        self.cursor_position.1 = 0;
    }

    fn yank_line(&mut self) {
        if let Some(line) = self.text_buffer.get(self.cursor_position.0) {
            self.clipboard.push_front(line.clone());
        }
    }

    fn delete_line(&mut self) {
        if self.text_buffer.len() > 1 {
            self.text_buffer.remove(self.cursor_position.0);
            if self.cursor_position.0 >= self.text_buffer.len() {
                self.cursor_position.0 = self.text_buffer.len() - 1;
            }
            self.cursor_position.1 = 0;
        } else {
            self.text_buffer[0].clear();
            self.cursor_position = (0, 0);
        }
    }

    fn paste(&mut self) {
        if let Some(content) = self.clipboard.front() {
            self.text_buffer[self.cursor_position.0].insert_str(self.cursor_position.1, content);
            self.cursor_position.1 += content.len();
        }
    }

    fn start_visual_mode(&mut self) {
        self.visual_start = Some(self.cursor_position);
    }

    fn clear_visual_mode(&mut self) {
        self.visual_start = None;
    }

    fn highlight_visual(&self) -> Vec<String> {
        let mut highlighted = self.text_buffer.clone();
        if let Some((start_row, start_col)) = self.visual_start {
            let end_row = self.cursor_position.0.max(start_row);
            let start_row = self.cursor_position.0.min(start_row);

            for (row, line) in highlighted.iter_mut().enumerate() {
                if row < start_row || row > end_row {
                    continue;
                }

                if row == start_row && row == end_row {
                    // Highlight part of the line between start_col and cursor_position.1
                    let (start, end) = if start_col <= self.cursor_position.1 {
                        (start_col, self.cursor_position.1)
                    } else {
                        (self.cursor_position.1, start_col)
                    };

                    let highlighted_segment = format!("\x1b[7m{}{}\x1b[0m", &line[start..end], "");
                    line.replace_range(start..end, &highlighted_segment);
                } else {
                    // Highlight the entire line if it's fully selected
                    let highlighted_line = format!("\x1b[7m{}{}\x1b[0m", line, "");
                    *line = highlighted_line;
                }
            }
        }
        highlighted
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input_buffer = String::new();
    let mut mode = Mode::Normal;
    let mut editor = Editor::new();

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(97), // Main editor area
                        Constraint::Percentage(3),  // Command line input
                    ]
                    .as_ref(),
                )
                .split(frame.size());

            // Rendor editor content with line numbers, cursor, and highlighting
            let editor_content: String = match mode {
                Mode::Visual => editor.highlight_visual().join("\n"), // Visual mode rendering
                _ => editor.render_with_cursor(&mode).join("\n"),     // Default rendering
            };

            // Render editor pane
            let editor_block = Paragraph::new(editor_content).block(
                Block::default()
                    .title(format!("Forge - {:?}", mode))
                    .borders(Borders::ALL),
            );
            frame.render_widget(editor_block, chunks[0]);

            // Render command line input
            let command_line = Paragraph::new(format!(":{}", input_buffer))
                .block(Block::default().title("Command").borders(Borders::ALL));
            frame.render_widget(command_line, chunks[1]);
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            match mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('i') => mode = Mode::Insert, // Enter Insert mode
                    KeyCode::Char('a') => {
                        editor.cursor_position.1 += 1; // Move one position to the right
                        mode = Mode::Insert;
                    }
                    KeyCode::Char('y') => editor.yank_line(), // Yank line
                    KeyCode::Char('p') => editor.paste(),     // Paste
                    KeyCode::Char('x') | KeyCode::Char('d') => editor.delete_line(), // Delete line
                    KeyCode::Char(':') => {
                        mode = Mode::Command; // Enter Command mode
                        input_buffer.clear();
                    }
                    KeyCode::Esc => {} // Stay in Normal mode
                    KeyCode::Char('v') if key.modifiers == KeyModifiers::CONTROL => {
                        mode = Mode::Visual; // Ctrl+V for Visual mode
                        editor.start_visual_mode();
                    }
                    KeyCode::Char('h') | KeyCode::Left => editor.move_cursor("left"),
                    KeyCode::Char('j') | KeyCode::Down => editor.move_cursor("down"),
                    KeyCode::Char('k') | KeyCode::Up => editor.move_cursor("up"),
                    KeyCode::Char('l') | KeyCode::Right => editor.move_cursor("right"),
                    KeyCode::Char('g') if key.modifiers == KeyModifiers::NONE => {
                        editor.move_cursor("top")
                    } // gg for top
                    KeyCode::Char('G') => editor.move_cursor("bottom"), // G for bottom
                    _ => {}
                },
                Mode::Insert => match key.code {
                    KeyCode::Esc => mode = Mode::Normal,        // Exit Insert mode
                    KeyCode::Char(c) => editor.insert_text(c),  // Insert Character
                    KeyCode::Tab => editor.insert_text('\t'),   // Handle Tab Key
                    KeyCode::Backspace => editor.delete_text(), // Handle backspace
                    KeyCode::Enter => editor.insert_newline(),  // Insert a new line
                    _ => {}
                },
                Mode::Visual => match key.code {
                    KeyCode::Esc => {
                        mode = Mode::Normal; // Exit Visual mode
                        editor.clear_visual_mode();
                    }
                    KeyCode::Char('$') => {
                        editor.cursor_position.1 =
                            editor.text_buffer[editor.cursor_position.0].len();
                    }
                    KeyCode::Char('j') | KeyCode::Down => editor.move_cursor("down"),
                    KeyCode::Char('y') => {
                        if let Some((start_row, _)) = editor.visual_start {
                            let end_row = editor.cursor_position.0.max(start_row);
                            for row in start_row..=end_row {
                                if let Some(line) = editor.text_buffer.get(row) {
                                    editor.clipboard.push_front(line.clone());
                                }
                            }
                        }
                        editor.clear_visual_mode();
                        mode = Mode::Normal;
                    }
                    KeyCode::Char('d') => {
                        if let Some((start_row, _)) = editor.visual_start {
                            let end_row = editor.cursor_position.0.max(start_row);
                            for _ in start_row..=end_row {
                                editor.delete_line();
                            }
                        }
                        editor.clear_visual_mode();
                        mode = Mode::Normal;
                    }
                    KeyCode::Char('p') => editor.paste(),
                    _ => {}
                },
                Mode::Command => match key.code {
                    KeyCode::Char(c) => input_buffer.push(c), // Add character to buffer
                    KeyCode::Backspace => {
                        input_buffer.pop(); // Remove last character
                    }
                    KeyCode::Enter => {
                        if input_buffer.starts_with("/") {
                            let query = &input_buffer[1..];
                            println!("Searching for: {}", query); // Placeholder for search
                        } else if let Ok(line) = input_buffer.parse::<usize>() {
                            if line > 0 && line <= editor.text_buffer.len() {
                                editor.cursor_position.0 = line - 1;
                                editor.cursor_position.1 = 0;
                            }
                        } else {
                            match input_buffer.as_str() {
                                "q" => {
                                    disable_raw_mode()?;
                                    terminal.clear()?;
                                    terminal.show_cursor()?;
                                    println!("\nExiting...");
                                    return Ok(());
                                }
                                "w" => {
                                    println!("\nFile saved! (Placeholder)");
                                }
                                cmd if cmd.starts_with("e ") => {
                                    let file = cmd.split_whitespace().nth(1).unwrap_or("");
                                    println!("\nEditing file: {}", file);
                                }
                                _ => {
                                    println!("\nUnknown command: {}", input_buffer);
                                }
                            }
                        }
                        input_buffer.clear(); // Clear input buffer after processing
                        mode = Mode::Normal; // Return to Normal mode
                    }
                    KeyCode::Esc => {
                        input_buffer.clear(); // Clear input buffer
                        mode = Mode::Normal; // Exit Command mode
                    }
                    _ => {}
                },
            }
        }
    }
}
