// src/ui/renderer.rs
use std::{collections::HashSet, io::{self, Write}, time::Instant};
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    queue,
    style::{self, Attribute, Color, Colors, Print, SetColors, Stylize},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use parking_lot::RwLock;
use std::sync::Arc;
use crate::editor::{Buffer, Editor, Mode};

#[derive(Debug)]
pub struct Renderer {
    screen_cache: Arc<RwLock<ScreenCache>>,
    dimensions: (u16, u16),
    dirty_regions: HashSet<Region>,
    last_render: Instant,
    force_redraw: bool,
    status_line_height: u16,
    command_line_height: u16,
}

#[derive(Debug)]
struct ScreenCache {
    buffer_lines: Vec<CachedLine>,
    status_line: String,
    command_line: String,
    last_update: Instant,
}

#[derive(Debug)]
struct CachedLine {
    content: String,
    styles: Vec<Style>,
    last_modified: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Region {
    Buffer { start: usize, end: usize },
    StatusLine,
    CommandLine,
}

#[derive(Debug, Clone)]
struct Style {
    foreground: Option<Color>,
    background: Option<Color>,
    attributes: Vec<Attribute>,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        // Setup terminal
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            terminal::Clear(ClearType::All)
        )?;
        
        terminal::enable_raw_mode()?;

        let (width, height) = terminal::size()?;
        
        Ok(Self {
            screen_cache: Arc::new(RwLock::new(ScreenCache {
                buffer_lines: Vec::new(),
                status_line: String::new(),
                command_line: String::new(),
                last_update: Instant::now(),
            })),
            dimensions: (width, height),
            dirty_regions: HashSet::new(),
            last_render: Instant::now(),
            force_redraw: true,
            status_line_height: 1,
            command_line_height: 1,
        })
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        // Restore terminal
        terminal::disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        Ok(())
    }

    // Main render loop with double buffering
    pub fn render<W: Write>(&mut self, writer: &mut W, editor: &Editor) -> io::Result<()> {
        let start = Instant::now();
        
        // Check if window size changed
        if let Ok(size) = terminal::size() {
            if size != self.dimensions {
                self.dimensions = size;
                self.force_redraw = true;
            }
        }

        // Get only changed regions
        let dirty_regions = if self.force_redraw {
            self.get_all_regions()
        } else {
            self.collect_dirty_regions(editor)
        };

        if dirty_regions.is_empty() && !self.force_redraw {
            return Ok(());
        }

        // Create off-screen buffer
        let mut buffer = Vec::new();
        
        // Render each region
        for region in dirty_regions {
            self.render_region(&mut buffer, editor, region)?;
        }

        // Hide cursor during updates
        queue!(writer, cursor::Hide)?;

        // Batch write changes
        writer.write_all(&buffer)?;
        writer.flush()?;

        // Show cursor at final position
        let (cursor_row, cursor_col) = self.get_cursor_screen_position(editor);
        queue!(
            writer,
            cursor::MoveTo(cursor_col, cursor_row),
            cursor::Show
        )?;
        
        writer.flush()?;

        // Update cache and reset flags
        self.update_screen_cache(editor);
        self.force_redraw = false;
        self.dirty_regions.clear();
        self.last_render = Instant::now();

        // Performance logging
        let elapsed = start.elapsed();
        if elapsed.as_millis() > 16 { // Target 60 FPS
            eprintln!("Slow render: {:?}", elapsed);
        }
        
        Ok(())
    }

    fn render_region<W: Write>(
        &self,
        writer: &mut W,
        editor: &Editor,
        region: Region,
    ) -> io::Result<()> {
        match region {
            Region::Buffer { start, end } => {
                self.render_buffer_region(writer, editor, start, end)?;
            }
            Region::StatusLine => {
                self.render_status_line(writer, editor)?;
            }
            Region::CommandLine => {
                self.render_command_line(writer, editor)?;
            }
        }
        Ok(())
    }

    fn render_buffer_region<W: Write>(
        &self,
        writer: &mut W,
        editor: &Editor,
        start: usize,
        end: usize,
    ) -> io::Result<()> {
        let buffer = editor.current_buffer();
        let viewport_height = self.get_viewport_height();
        
        for row in start..end.min(viewport_height) {
            // Position cursor
            queue!(writer, cursor::MoveTo(0, row as u16))?;

            // Render line with number
            if let Some(line) = buffer.get_line(row) {
                let line_num = format!("{:4} │ ", row + 1);
                queue!(
                    writer,
                    SetColors(Colors::new(Color::DarkGrey, Color::Reset)),
                    Print(&line_num),
                    SetColors(Colors::new(Color::Reset, Color::Reset)),
                )?;

                // Apply syntax highlighting and render line content
                let rendered = self.highlight_line(line, *editor.mode());
                queue!(writer, Print(rendered))?;

                // Clear to end of line
                queue!(writer, Clear(ClearType::UntilNewLine))?;
            } else {
                // Empty line marker
                queue!(
                    writer,
                    SetColors(Colors::new(Color::DarkGrey, Color::Reset)),
                    Print("   ~ │"),
                    Clear(ClearType::UntilNewLine)
                )?;
            }
        }
        Ok(())
    }

    fn render_status_line<W: Write>(&self, writer: &mut W, editor: &Editor) -> io::Result<()> {
        let row = self.dimensions.1 - 2;
        let mode_text = editor.mode().display_name();
        let file_info = editor.file_info();  // Get file info from editor instead of buffer
        let position_info = editor.cursor_position_info();

        queue!(
            writer,
            cursor::MoveTo(0, row),
            SetColors(Colors::new(Color::Black, Color::Grey)),
            Print(format!(" {} | {} | {} ", mode_text, file_info, position_info)),
            SetColors(Colors::new(Color::Reset, Color::Reset)),
            Clear(ClearType::UntilNewLine)
        )
    }

    // Update command line rendering to use mode().command_prefix()
    fn render_command_line<W: Write>(&self, writer: &mut W, editor: &Editor) -> io::Result<()> {
        let row = self.dimensions.1 - 1;
        let mode = editor.mode();
        
        if let Mode::Command(_) = mode {
            let prefix = mode.command_prefix();
            let command = editor.command_line_content();
            
            queue!(
                writer,
                cursor::MoveTo(0, row),
                Print(format!("{}{}", prefix, command)),
                Clear(ClearType::UntilNewLine)
            )
        } else {
            // Clear command line when not in command mode
            queue!(
                writer,
                cursor::MoveTo(0, row),
                Clear(ClearType::UntilNewLine)
            )
        }
    }

    fn highlight_line(&self, line: &str, mode: Mode) -> String {
        // Add syntax highlighting here
        // For now, just return the plain line
        line.to_string()
    }

    fn get_cursor_screen_position(&self, editor: &Editor) -> (u16, u16) {
        let (row, col) = editor.cursor_position();
        let line_number_width = 6; // "123 │ "
        (
            row as u16,
            (col + line_number_width) as u16
        )
    }

    fn get_viewport_height(&self) -> usize {
        (self.dimensions.1 - self.status_line_height - self.command_line_height) as usize
    }

    fn get_all_regions(&self) -> HashSet<Region> {
        let mut regions = HashSet::new();
        regions.insert(Region::Buffer {
            start: 0,
            end: self.get_viewport_height(),
        });
        regions.insert(Region::StatusLine);
        regions.insert(Region::CommandLine);
        regions
    }

    fn collect_dirty_regions(&self, editor: &Editor) -> HashSet<Region> {
        let mut regions = self.dirty_regions.clone();
        
        // Check if buffer content changed
        let cache = self.screen_cache.read();
        let buffer = editor.current_buffer();
        
        if cache.buffer_lines.len() != buffer.line_count() {
            regions.insert(Region::Buffer {
                start: 0,
                end: self.get_viewport_height(),
            });
        }

        // Check if status line needs update
        if editor.mode().display_name() != cache.status_line {
            regions.insert(Region::StatusLine);
        }

        // Check if command line needs update
        if let Mode::Command(_) = editor.mode() {
            regions.insert(Region::CommandLine);
        }

        regions
    }

    fn update_screen_cache(&self, editor: &Editor) {
        let mut cache = self.screen_cache.write();
        let buffer = editor.current_buffer();
        
        // Update buffer lines
        cache.buffer_lines = buffer
            .get_content()
            .iter()
            .map(|line| CachedLine {
                content: line.clone(),
                styles: Vec::new(), // Add styles when implementing syntax highlighting
                last_modified: Instant::now(),
            })
            .collect();

        // Update status and command lines
        cache.status_line = editor.mode().display_name().to_string();
        cache.command_line = editor.command_line_content().to_string();
        cache.last_update = Instant::now();
    }

    pub fn mark_dirty(&mut self, region: Region) {
        self.dirty_regions.insert(region);
    }

    pub fn force_redraw(&mut self) {
        self.force_redraw = true;
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.dimensions = (width, height);
        self.force_redraw = true;
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}