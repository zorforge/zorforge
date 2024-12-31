// src/ui/renderer.rs
use crossterm::{queue, style::*, terminal::*};
use parking_lot::RwLock;
use std::io::Write;

pub struct Renderer {
    screen_cache: Arc<RwLock<ScreenCache>>,
    dimensions: (u16, u16),
    dirty_regions: HashSet<Region>,
}

#[derive(Debug)]
struct ScreenCache {
    lines: Vec<CachedLine>,
    styles: Vec<Vec<Style>>,
    last_update: Instant,
}

impl Renderer {
    // Main render loop with double buffering
    pub fn render<W: Write>(&mut self, writer: &mut W, editor: &Editor) -> io::Result<()> {
        let start = Instant::now();

        // Get only changed regions
        let dirty_regions = self.collect_dirty_regions(editor);
        if dirty_regions.is_empty() && !editor.needs_full_redraw() {
            return Ok(());
        }

        // Create off-screen buffer
        let mut buffer = Vec::new();

        // Only render dirty regions
        for region in dirty_regions {
            self.render_region(&mut buffer, editor, region)?;
        }

        // Efficient terminal updates
        queue!(writer,
            SavePosition,
            Clear(ClearType::All)
        )?;

        // Batch write changes
        writer.write_all(&buffer)?;

        queue!(writer,
            RestorePosition
        )?;

        writer.flush()?;

        // Update cache
        self.update_screen_cache(editor);

        // Performance logging
        let elapsed = start.elapsed();
        if elapsed > Duration::from_millis(16) { // Target 60 fps
            log::warn!("Slow render: {:?}", elapsed);
        }

        Ok(())
    }

    // Efficient partial updates
    fn render_region<W: Write>(
        &self,
        writer: &mut W,
        editor: &Editor,
        region: Region,
    ) -> io::Result <()> {
        match region {
            Region::Buffer { start, end } => {
                let viewport = edtor.buffer().get_viewport();
                let visible_lines = viewport.visible_lines();

                // Only render visible, dirty lines
                for line_num in start..end {
                    if visible_lines.contains(&line_num) {
                        self.render_line(writer, editor, line_num)?;
                    }
                }
            }
            Region::StatusLine => self.render_status_line(writer, editor)?,
            Region::CommandLine => self.render_command_line(writer, editor)?,
        }

        Ok(())
    }

    // Efficient line rendering with styles
    fn render_line<W: Write>(
        &self,
        writer: &mut W,
        editor: &editor,
        line_num: usize,
    ) -> io::Result<()> {
        let buffer = editor.buffer();
        let cache = self.screen_cache.read();

        // Check cache first
        if let Some(cached) = cache.get_line(line_num) {
            if !buffer.is_line_dirty(line_num) {
                return self.write_cached_line(writer, cached);
            }
        }

        // Render line with syntax highlighting
        if let Some(line) = buffer.get_line(line_num) {
            let styles = editor.syntax_highlighter().highlight_line(line);
            let rendered = self.apply_styles(line, &styles);

            queue!(writer,
                MoveTo(0, line_num as u16),
                PrintStyledContent(rendered)
            )?;
        }

        Ok(())
    }
}