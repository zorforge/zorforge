// src/ui/windows/mod.rs
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::sync::Arc;
use parking_lot::RwLock;
use portable_pty::{native_pty_system, CommandBuilder, Child as PtyChild, MasterPty, PtySize};
use crate::editor::Buffer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitDirection {
    Vertical,
    Horizontal,
}

#[derive(Debug)]
pub enum WindowContent {
    Buffer(Arc<RwLock<Buffer>>),
    Terminal(Arc<RwLock<Terminal>>),
}

#[derive(Debug, Clone)]
pub struct WindowDimensions {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
pub struct Window {
    id: WindowId,
    content: WindowContent,
    dimensions: WindowDimensions,
    is_focused: bool,
}

pub struct Terminal {
    pty: TerminalPty,
    scrollback: Vec<String>,
    cursor: (u16, u16),
}

struct TerminalPty {
    master: Option<Box<dyn MasterPty>>,
    child: Option<Box<dyn PtyChild>>,
}

// Manual Debug implementation for Terminal
impl std::fmt::Debug for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Terminal")
            .field("scrollback", &self.scrollback)
            .field("cursor", &self.cursor)
            .field("has_pty", &self.pty.master.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(usize);

#[derive(Debug)]
pub struct Layout {
    root: Option<Box<LayoutNode>>,
}

#[derive(Debug)]
enum LayoutNode {
    Leaf {
        window_id: WindowId,
        dimensions: WindowDimensions,
    },
    Split {
        direction: SplitDirection,
        ratio: f32,
        left: Box<LayoutNode>,
        right: Box<LayoutNode>,
        dimensions: WindowDimensions,
    },
}

impl Window {
    pub fn new(id: WindowId, content: WindowContent, dimensions: WindowDimensions) -> Self {
        Self {
            id,
            content,
            dimensions,
            is_focused: false,
        }
    }

    pub fn new_terminal(id: WindowId, dimensions: WindowDimensions) -> io::Result<Self> {
        let terminal = Terminal::new();
        let terminal = Arc::new(RwLock::new(terminal));
        Ok(Self::new(id, WindowContent::Terminal(terminal), dimensions))
    }

    pub fn resize(&mut self, new_dimensions: WindowDimensions) {
        if let WindowContent::Terminal(term) = &self.content {
            let mut term = term.write();
            let _ = term.resize(new_dimensions.width, new_dimensions.height);
        }
        self.dimensions = new_dimensions;
    }

    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            pty: TerminalPty {
                master: None,
                child: None,
            },
            scrollback: Vec::new(),
            cursor: (0, 0),
        }
    }

    pub fn spawn(&mut self) -> io::Result<()> {
        let pty_system = native_pty_system();
        let size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system.openpty(size)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        let cmd = if cfg!(windows) {
            "cmd.exe"
        } else {
            "/bin/bash"
        };

        let mut cmd_builder = CommandBuilder::new(cmd);
        let child = pair.slave.spawn_command(cmd_builder)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        self.pty.master = Some(pair.master);
        self.pty.child = Some(child);
        Ok(())
    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(master) = &mut self.pty.master {
            let mut writer = master.take_writer()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            writer.write_all(data)?;
            writer.flush()?;
        }
        Ok(())
    }

    pub fn read(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        if let Some(master) = &mut self.pty.master {
            let mut reader = master.try_clone_reader()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            reader.read_to_end(&mut buffer)?;
        }
        Ok(buffer)
    }

    pub fn resize(&mut self, width: u16, height: u16) -> io::Result<()> {
        if let Some(master) = &mut self.pty.master {
            master.resize(PtySize {
                rows: height,
                cols: width,
                pixel_width: 0,
                pixel_height: 0,
            }).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        }
        Ok(())
    }
}

impl Layout {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn split_vertical(&mut self, dimensions: WindowDimensions, ratio: f32) -> (WindowDimensions, WindowDimensions) {
        let left_width = (dimensions.width as f32 * ratio) as u16;
        let right_width = dimensions.width - left_width;

        (
            WindowDimensions {
                x: dimensions.x,
                y: dimensions.y,
                width: left_width,
                height: dimensions.height,
            },
            WindowDimensions {
                x: dimensions.x + left_width,
                y: dimensions.y,
                width: right_width,
                height: dimensions.height,
            },
        )
    }

    pub fn split_horizontal(&mut self, dimensions: WindowDimensions, ratio: f32) -> (WindowDimensions, WindowDimensions) {
        let top_height = (dimensions.height as f32 * ratio) as u16;
        let bottom_height = dimensions.height - top_height;

        (
            WindowDimensions {
                x: dimensions.x,
                y: dimensions.y,
                width: dimensions.width,
                height: top_height,
            },
            WindowDimensions {
                x: dimensions.x,
                y: dimensions.y + top_height,
                width: dimensions.width,
                height: bottom_height,
            },
        )
    }

    pub fn add_bottom_panel(&mut self, total_height: u16, panel_height: u16) -> WindowDimensions {
        WindowDimensions {
            x: 0,
            y: total_height - panel_height,
            width: 0,
            height: panel_height,
        }
    }
}

pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
    layout: Layout,
    next_id: usize,
    active_window: Option<WindowId>,
    terminal_window: Option<WindowId>,
    total_dimensions: WindowDimensions,
}

impl WindowManager {
    pub fn new(width: u16, height: u16) -> Self {
        let dimensions = WindowDimensions {
            x: 0,
            y: 0,
            width,
            height,
        };

        let mut wm = Self {
            windows: HashMap::new(),
            layout: Layout::new(),
            next_id: 0,
            active_window: None,
            terminal_window: None,
            total_dimensions: dimensions.clone(),
        };

        let initial_buffer = Arc::new(RwLock::new(Buffer::new()));
        let window_id = wm.create_window(
            WindowContent::Buffer(initial_buffer),
            dimensions,
        );
        wm.active_window = Some(window_id);

        wm
    }

    fn create_window(&mut self, content: WindowContent, dimensions: WindowDimensions) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;

        let window = Window::new(id, content, dimensions);
        self.windows.insert(id, window);
        id
    }

    pub fn split(&mut self, direction: SplitDirection) -> io::Result<()> {
        if let Some(active_id) = self.active_window {
            if let Some(active_window) = self.windows.get(&active_id) {
                let dimensions = active_window.dimensions.clone();
                let (first_dims, second_dims) = match direction {
                    SplitDirection::Vertical => self.layout.split_vertical(dimensions.clone(), 0.5),
                    SplitDirection::Horizontal => self.layout.split_horizontal(dimensions.clone(), 0.5),
                };

                if let Some(window) = self.windows.get_mut(&active_id) {
                    window.resize(first_dims.clone());
                }

                let new_buffer = Arc::new(RwLock::new(Buffer::new()));
                let new_window_id = self.create_window(
                    WindowContent::Buffer(new_buffer),
                    second_dims.clone(),
                );

                let new_node = LayoutNode::Split {
                    direction,
                    ratio: 0.5,
                    left: Box::new(LayoutNode::Leaf {
                        window_id: active_id,
                        dimensions: first_dims,
                    }),
                    right: Box::new(LayoutNode::Leaf {
                        window_id: new_window_id,
                        dimensions: second_dims,
                    }),
                    dimensions,
                };

                self.layout.root = Some(Box::new(new_node));
                self.focus_window(new_window_id);
            }
        }
        Ok(())
    }

    pub fn focus_window(&mut self, id: WindowId) {
        if let Some(current_id) = self.active_window {
            if let Some(window) = self.windows.get_mut(&current_id) {
                window.unfocus();
            }
        }

        if let Some(window) = self.windows.get_mut(&id) {
            window.focus();
            self.active_window = Some(id);
        }
    }

    pub fn close_window(&mut self, id: WindowId) -> io::Result<()> {
        if self.windows.remove(&id).is_some() {
            self.layout.root = self.layout.root.take().map(|node| {
                self.remove_window_from_layout(*node, id)
            }).flatten().map(Box::new);

            if Some(id) == self.active_window {
                self.active_window = self.windows.keys().next().copied();
                if let Some(new_active) = self.active_window {
                    self.focus_window(new_active);
                }
            }

            if Some(id) == self.terminal_window {
                self.terminal_window = None;
            }
        }
        Ok(())
    }

    fn remove_window_from_layout(&mut self, node: LayoutNode, id: WindowId) -> Option<LayoutNode> {
        match node {
            LayoutNode::Leaf { window_id, .. } if window_id == id => None,
            LayoutNode::Split { direction, ratio, left, right, dimensions } => {
                let new_left = self.remove_window_from_layout(*left, id);
                let new_right = self.remove_window_from_layout(*right, id);

                match (new_left, new_right) {
                    (Some(l), Some(r)) => Some(LayoutNode::Split {
                        direction,
                        ratio,
                        left: Box::new(l),
                        right: Box::new(r),
                        dimensions,
                    }),
                    (Some(l), None) => Some(l),
                    (None, Some(r)) => Some(r),
                    (None, None) => None,
                }
            }
            other => Some(other),
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) -> io::Result<()> {
        let new_dimensions = WindowDimensions {
            x: 0,
            y: 0,
            width,
            height,
        };
        
        self.total_dimensions = new_dimensions.clone();

        if let Some(mut root) = self.layout.root.take() {
            self.resize_layout_node(&mut root, &new_dimensions)?;
            self.layout.root = Some(root);
        }
        Ok(())
    }

    fn resize_layout_node(&mut self, node: &mut LayoutNode, dimensions: &WindowDimensions) -> io::Result<()> {
        match node {
            LayoutNode::Leaf { window_id, dimensions: window_dims } => {
                *window_dims = dimensions.clone();
                if let Some(window) = self.windows.get_mut(window_id) {
                    window.resize(dimensions.clone());
                }
            }
            LayoutNode::Split { direction, ratio, left, right, dimensions: split_dims } => {
                *split_dims = dimensions.clone();
                let (left_dims, right_dims) = match direction {
                    SplitDirection::Vertical => self.layout.split_vertical(dimensions.clone(), *ratio),
                    SplitDirection::Horizontal => self.layout.split_horizontal(dimensions.clone(), *ratio),
                };
                self.resize_layout_node(left, &left_dims)?;
                self.resize_layout_node(right, &right_dims)?;
            }
        }
        Ok(())
    }

    pub fn toggle_terminal(&mut self) -> io::Result<()> {
        if let Some(term_id) = self.terminal_window {
            self.close_window(term_id)?;
        } else {
            let term_height = 10;
            let term_dims = self.layout.add_bottom_panel(
                self.total_dimensions.height,
                term_height,
            );
            
            let window = Window::new_terminal(
                WindowId(self.next_id),
                term_dims,
            )?;
            
            let window_id = window.id;
            self.windows.insert(window_id, window);
            self.next_id += 1;

            self.terminal_window = Some(window_id);
            self.focus_window(window_id);
        }
        Ok(())
    }

    pub fn find_terminal_window(&self) -> Option<WindowId> {
        self.terminal_window
    }
}