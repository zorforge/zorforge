// src/editor/buffer_manager.rs
use std::collections::{HashMap, VecDeque};
use crossterm::style::Stylize;
use parking_lot::RwLock;
use rayon::prelude::*;

#[derive(Debug)]
pub struct BufferManager {
    buffers: HashMap<BufferId, Arc<RwLock<Buffer>>>,
    buffer_order: VecDeque<BufferId>,
    active_buffer: Option<BufferId>,
    line_cache: LruCache<(BufferId, usize), CachedLine>,
}

#[derive(Debug)]
struct CachedLine {
    content: String,
    styles: Vec<Style>,
    last_modified: Instant,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            buffer_order: VecDeque::new(),
            active_buffer: None,
            line_cache: LruCache::new(1000), // Cache 1000 lines
        }
    }

    // Efficient buffer switching
    pub fn switch_buffer(&mut self, id: BufferId) -> io::Result<()> {
        if let Some(current) = self.active_buffer {
            // Cache current buffer's viewport
            if let Some(buffer) = self.buffers.get(&current) {
                self.cache_viewport(buffer);
            }
        }

        self.active_buffer = Some(id);
        self.buffer_order.push_front(id);

        // Maintain reasonable buffer limit
        if self.buffer_order.len() > 100 {
            if let Some(old_id) = self.buffer_order.pop_back() {
                self.cleanup_buffer(old_id);
            }
        }

        Ok(())
    }

    // Parallel line processing for large files
    pub fn process_viewport(&self, start: usize, end: usize) -> Vec<RenderedLine> {
        if let Some(buffer) = self.get_active_buffer() {
            let lines = buffer.read().get_lines(start..end);

            lines.par_iter()
                .map(|line| self.render_line(line))
                .collect()
        } else {
            Vec::new()
        }
    }

    // Incremental update system
    pub fn update_viewport(&mut self, changes: &[BufferChange]) {
        let viewport = self.get_viewport();
        let affected_lines: HashSet<usize> = changes.iter()
            .flat_map(|change| change.affected_lines())
            .collect();

        // Only render affected lines
        for line_num in affected_lines {
            if viewport.contains(&line_num) {
                self.invalidate_cache(line_num);
                self.rerender_line(line_num);
            }
        }
    }

    // Efficient line caching
    fn cache_viewport(&mut self, buffer: &Buffer) {
        let viewport = buffer.get_viewport();
        for line_num in viewport.visible_lines() {
            if let Some(line) = buffer.get_line(line_num) {
                self.line_cache.put(
                    (buffer.id(), line_num),
                    CachedLine {
                        content: line.to_string(),
                        styles: buffer.get_line_styles(line_num),
                        last_modified: Instant::now(),
                    }
                );
            }
        }
    }

    // Async file loading
    pub async fn load_file(&mut self, path: &Path) -> io::Result<BufferId> {
        let buffer_id = self.create_buffer();

        // Load file content asynchronously
        let contents = tokio::fs::read_to_string(path).await?;

        if let Some(buffer) = self.buffers.get_mut(&buffer_id) {
            let mut buffer = buffer.write();
            buffer.set_contents(&contents);
            buffer.set_path(path);
        }

        Ok(buffer_id)
    }
}