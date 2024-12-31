// src/ui/render.rs
use std::collections::HashSet;
use crossterm::style::{Color, Attribute};
use crate::editor::mode::Mode;

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
    pub attributes: HashSet<Attribute>,
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub show_line_numbers: bool,
    pub tab_size: usize,
    pub highlight_current_line: bool,
    pub show_whitespace: bool,
    pub window_width: usize,
    pub window_height: usize,
}

#[derive(Debug)]
pub struct RenderRegion {
    pub start_row: usize,
    pub end_row: usize,
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Debug)]
pub enum RenderElement {
    Text {
        content: String,
        style: Style,
    },
    LineNumber {
        number: usize,
        style: Style,
    },
    Cursor {
        position: (usize, usize),
        style: Style,
    },
    StatusLine {
        content: String,
        style: Style,
    },
    CommandLine {
        content: String,
        style: Style,
    },
    Selection {
        region: RenderRegion,
        style: Style,
    },
    SearchMatch {
        region: RenderRegion,
        is_current: bool,
        style: Style,
    },
}

pub trait Render {
    /// Render the content with the given mode
    fn render(&self, mode: &Mode) -> Vec<String>;

    /// Render a specific region of content
    fn render_region(&self, region: RenderRegion, mode: &Mode) -> Vec<String>;

    /// Get the dimensions required for rendering
    fn get_dimensions(&self) -> (usize, usize);

    /// Check if content needs redrawing
    fn needs_redraw(&self) -> bool;

    /// Get elements that need to be rendered
    fn get_render_elements(&self, mode: &Mode) -> Vec<RenderElement>;
}

impl Style {
    pub fn new() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
            attributes: HashSet::new(),
        }
    }

    pub fn with_fg_color(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn with_attribute(mut self, attr: Attribute) -> Self {
        self.attributes.insert(attr);
        self
    }

    pub fn merge(&self, other: &Style) -> Style {
        let mut merged = self.clone();
        if other.fg_color.is_some() {
            merged.fg_color = other.fg_color;
        }
        if other.bg_color.is_some() {
            merged.bg_color = other.bg_color;
        }
        merged.attributes.extend(other.attributes.iter());
        merged
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            tab_size: 4,
            highlight_current_line: true,
            show_whitespace: false,
            window_width: 80,
            window_height: 24,
        }
    }
}

impl RenderRegion {
    pub fn new(start_row: usize, end_row: usize, start_col: usize, end_col: usize) -> Self {
        Self {
            start_row,
            end_row,
            start_col,
            end_col,
        }
    }

    pub fn contains_point(&self, row: usize, col: usize) -> bool {
        row >= self.start_row && row < self.end_row &&
        col >= self.start_col && col < self.end_col
    }

    pub fn overlaps(&self, other: &RenderRegion) -> bool {
        !(self.end_row <= other.start_row || other.end_row <= self.start_row ||
          self.end_col <= other.start_col || other.end_col <= self.start_col)
    }

    pub fn height(&self) -> usize {
        self.end_row - self.start_row
    }

    pub fn width(&self) -> usize {
        self.end_col - self.start_col
    }
}

// Helper functions for creating common render elements
pub fn create_text_element(content: String, fg: Option<Color>, bg: Option<Color>) -> RenderElement {
    RenderElement::Text {
        content,
        style: Style {
            fg_color: fg,
            bg_color: bg,
            attributes: HashSet::new(),
        }
    }
}

pub fn create_line_number(number: usize) -> RenderElement {
    RenderElement::LineNumber {
        number,
        style: Style {
            fg_color: Some(Color::DarkGrey),
            bg_color: None,
            attributes: HashSet::new(),
        }
    }
}

pub fn create_cursor(position: (usize, usize), mode: &Mode) -> RenderElement {
    let style = match mode {
        Mode::Normal => Style::new()
            .with_bg_color(Color::Grey)
            .with_fg_color(Color::Black),
        Mode::Insert(_) => Style::new()
            .with_bg_color(Color::Green)
            .with_fg_color(Color::Black),
        Mode::Visual(_) => Style::new()
            .with_bg_color(Color::Blue)
            .with_fg_color(Color::White),
        Mode::Command(_) => Style::new()
            .with_bg_color(Color::Yellow)
            .with_fg_color(Color::Black),
    };

    RenderElement::Cursor {
        position,
        style,
    }
}

pub fn create_status_line(content: String, mode: &Mode) -> RenderElement {
    let style = match mode {
        Mode::Normal => Style::new()
            .with_bg_color(Color::DarkGrey)
            .with_fg_color(Color::White),
        Mode::Insert(_) => Style::new()
            .with_bg_color(Color::DarkGreen)
            .with_fg_color(Color::White),
        Mode::Visual(_) => Style::new()
            .with_bg_color(Color::DarkBlue)
            .with_fg_color(Color::White),
        Mode::Command(_) => Style::new()
            .with_bg_color(Color::DarkYellow)
            .with_fg_color(Color::White),
    };

    RenderElement::StatusLine {
        content,
        style,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_merge() {
        let style1 = Style::new()
            .with_fg_color(Color::Red)
            .with_attribute(Attribute::Bold);
        
        let style2 = Style::new()
            .with_bg_color(Color::Blue)
            .with_attribute(Attribute::Italic);

        let merged = style1.merge(&style2);
        assert_eq!(merged.fg_color, Some(Color::Red));
        assert_eq!(merged.bg_color, Some(Color::Blue));
        assert!(merged.attributes.contains(&Attribute::Bold));
        assert!(merged.attributes.contains(&Attribute::Italic));
    }

    #[test]
    fn test_render_region_overlap() {
        let region1 = RenderRegion::new(0, 10, 0, 10);
        let region2 = RenderRegion::new(5, 15, 5, 15);
        let region3 = RenderRegion::new(20, 30, 20, 30);

        assert!(region1.overlaps(&region2));
        assert!(!region1.overlaps(&region3));
        assert!(!region2.overlaps(&region3));
    }

    #[test]
    fn test_render_region_contains_point() {
        let region = RenderRegion::new(5, 10, 5, 10);
        
        assert!(region.contains_point(7, 7));
        assert!(!region.contains_point(4, 7));
        assert!(!region.contains_point(7, 11));
    }
}