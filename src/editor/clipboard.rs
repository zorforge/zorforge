// src/editor/clipboard.rs
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Clipboard {
    history: VecDeque<String>,
    max_history: usize,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            max_history: 10, // Default to storing last 10 copies
        }
    }

    pub fn new_with_capacity(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    // Add content to clipboard
    pub fn yank(&mut self, content: String) {
        if content.is_empty() {
            return;
        }

        self.history.push_front(content);

        // Maintain max history size
        while self.history.len() > self.max_history {
            self.history.pop_back();
        }
    }

    // Get most recent clipboard content without removing it
    pub fn peek(&self) -> Option<&String> {
        self.history.front()
    }

    // Get content at specific history index
    pub fn peek_at(&self, index: usize) -> Option<&String> {
        self.history.get(index)
    }

    // Get and remove the most recent content
    pub fn pop(&mut self) -> Option<String> {
        self.history.pop_front()
    }

    // Clean clipboard history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    // Get number of items in clipboard
    pub fn len(&self) -> usize {
        self.history.len()
    }

    // Check if clipboard is empty
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    // Get clipboard history
    pub fn get_history(&self) -> &VecDeque<String> {
        &self.history
    }

    // Rotate clipboard history (useful for cycling through yanked items)
    pub fn rotate_forward(&mut self) {
        if !self.is_empty() {
            if let Some(item) = self.history.pop_front() {
                self.history.push_back(item);
            }
        }
    }

    pub fn rotate_backward(&mut self) {
        if !self.is_empty() {
            if let Some(item) = self.history.pop_back() {
                self.history.push_front(item);
            }
        }
    }

    // Yank multiple lines at once
    pub fn yank_lines(&mut self, lines: Vec<String>) {
        if lines.is_empty() {
            return;
        }

        let content = lines.join("\n");
        self.yank(content);
    }

    // Get most recent content split into lines
    pub fn peek_lines(&self) -> Option<Vec<&str>> {
        self.peek().map(|content| content.lines().collect())
    }

    // Set maximum history size
    pub fn set_max_history(&mut self, max_history: usize) {
        self.max_history = max_history;
        while self.history.len() > self.max_history {
            self.history.pop_back();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clipboard() {
        let clipboard = Clipboard::new();
        assert!(clipboard.is_empty());
        assert_eq!(clipboard.len(), 0);
    }

    #[test]
    fn test_yank_and_peek() {
        let mut clipboard = Clipboard::new();
        clipboard.yank("test content".to_string());
        assert_eq!(clipboard.peek(), Some(&"test content".to_string()));
        assert_eq!(clipboard.len(), 1);
    }

    #[test]
    fn test_max_history() {
        let mut clipboard = Clipboard::new_with_capacity(2);
        clipboard.yank("first".to_string());
        clipboard.yank("second".to_string());
        clipboard.yank("third".to_string());

        assert_eq!(clipboard.len(), 2);
        assert_eq!(clipboard.peek(), Some(&"third".to_string()));
        assert_eq!(clipboard.peek_at(1), Some(&"second".to_string()));
        assert_eq!(clipboard.peek_at(2), None);
    }

    #[test]
    fn test_rotation() {
        let mut clipboard = Clipboard::new();
        clipboard.yank("first".to_string());
        clipboard.yank("second".to_string());
        clipboard.yank("third".to_string());

        clipboard.rotate_forward();
        assert_eq!(clipboard.peek(), Some(&"second".to_string()));
        
        clipboard.rotate_backward();
        assert_eq!(clipboard.peek(), Some(&"third".to_string()));
    }

    #[test]
    fn test_yank_lines() {
        let mut clipboard = Clipboard::new();
        let lines = vec![
            "line1".to_string(),
            "line2".to_string(),
            "line3".to_string(),
        ];

        clipboard.yank_lines(lines);
        assert_eq!(clipboard.peek(), Some(&"line1\nline2\nline3".to_string()));

        if let Some(peeked_lines) = clipboard.peek_lines() {
            assert_eq!(peeked_lines, vec!["line1", "line2", "line3"]);
        }
    }

    #[test]
    fn test_clear() {
        let mut clipboard = Clipboard::new();
        clipboard.yank("test".to_string());
        assert!(!clipboard.is_empty());

        clipboard.clear();
        assert!(clipboard.is_empty());
        assert_eq!(clipboard.len(), 0);
    }

    #[test]
    fn test_set_max_history() {
        let mut clipboard = Clipboard::new();
        clipboard.yank("first".to_string());
        clipboard.yank("second".to_string());
        clipboard.yank("third".to_string());

        clipboard.set_max_history(2);
        assert_eq!(clipboard.len(), 2);
        assert_eq!(clipboard.peek(), Some(&"third".to_string()));
        assert_eq!(clipboard.peek_at(1), Some(&"second".to_string()));
    }
}