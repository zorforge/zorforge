// src/editor/viewport.rs
use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub start: usize,
    pub height: usize,
    pub width: usize,
}

impl Viewport {
    pub fn visible_lines(&self) -> Range<usize> {
        self.start..self.start + self.height
    }

    pub fn contains(&self, line: usize) -> bool {
        line >= self.start && line < self.start + self.height
    }

    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = height;
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }
}