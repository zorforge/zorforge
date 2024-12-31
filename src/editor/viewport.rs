// src/editor/viewport.rs
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    start: usize,
    height: usize,
    width: usize,
}

impl Viewport {
    pub fn visible_lines(&self) -> Range<usize> {
        self.start..self.start + self.height
    }

    pub fn contains(&self, line: usize) -> bool {
        line >= self.start && line < self.start + self.height
    }
}