// src/ui/windows/mod.rs

enum SplitDirection {
    Vertical,
    Horizontal,
}

struct Window {
    // TODO: implement new window
    pub fn new() -> Self {
        // implement
    }

    pub fn new_terminal() -> Self {
        // implement
    }
}
pub struct WindowManager {
    windows: Vec<Window>,
    active_window: usize,
    layout: Layout,
}

impl WindowManager {
    // Handle window splits
    pub fn split(&mut self, direction: SplitDirection) {
        let active = self.windows[self.active_window].clone();
        let new_window = Window::new();

        match direction {
            SplitDirection::Vertical => {
                self.layout.split_vertical(active, new_window);
            }
            SplitDirection::Horizontal => {
                self.layout.split_horizontal(active, new_window);
            }
        }

        self.windows.push(new_window);
    }

    pub fn find_terminal_window(&mut self) -> Option<usize> {
        // implement
    }

    // Toggle terminal
    pub fn toggle_terminal(&mut self) {
        if let Some(term_idx) = self.find_terminal_window() {
            self.windows.remove(term_idx);
        } else {
            let term = Window::new_terminal();
            self.windows.push(term);
            self.layout.add_bottom_panel(term);
        }
    }
}