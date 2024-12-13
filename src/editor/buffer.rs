// src/editor/buffer.rs
#[derive(Clone, Debug)]
pub enum BufferChange {
    Insert {
        position: (usize, usize),
        content: String,
    },
    Delete {
        position: (usize, usize),
        content: String,
    },
    NewLine {
        position: (usize, usize),
        content: String,
    },
    DeleteLine {
        position: usize,
        content: String,
    },
}

#[derive(Debug)]
pub struct Buffer {
    content: Vec<String>,             // Lines of text in the buffer
    cursor_position: (usize, usize),  // (row, column)
    visual_start: Option<(usize, usize)>, // Start of visual selection
    tab_size: usize,                  // Tab size in spaces
    search_matches: Vec<(usize, usize, usize)>, // (row, start_col, end_col)
    current_match: Option<usize>,     // Index into search_matches
    undo_stack: Vec<(BufferChange, (usize, usize))>, // (change, cursor_position)
    redo_stack: Vec<(BufferChange, (usize, usize))>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            content: vec![String::new()],
            cursor_position: (0, 0),
            visual_start: None,
            tab_size: 4,
            search_matches: Vec::new(),
            current_match: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let current_line = &mut self.content[self.cursor_position.0];
        let change = BufferChange::Insert {
            position: self.cursor_position,
            content: c.to_string(),
        };
        current_line.insert(self.cursor_position.1, c);
        self.cursor_position.1 += 1;
        self.record_change(change);
    }

    // Get character before cursor for ctrl+w word deletion
    pub fn get_char_before_cursor(&self) -> Option<char> {
        if self.cursor_position.1 > 0 {
            self.content[self.cursor_position.0]
                .chars()
                .nth(self.cursor_position.1 - 1)
        } else {
            None
        }
    }

    // Word movement operations
    pub fn move_word_forward(&mut self) {
        let line = &self.content[self.cursor_position.0];
        if let Some(next_word) = line[self.cursor_position.1..]
            .char_indices()
            .skip_while(|(_, c)| !c.is_whitespace())
            .skip_while(|(_, c)| c.is_whitespace())
            .next()
        {
            self.cursor_position.1 += next_word.0;
        } else {
            self.cursor_position.1 = line.len();
        }
    }

    pub fn move_word_backward(&mut self) {
        let line = &self.content[self.cursor_position.0];
        if self.cursor_position.1 > 0 {
            let reversed: String = line[..self.cursor_position.1].chars().rev().collect();
            if let Some(prev_word) = reversed
                .char_indices()
                .skip_while(|(_, c)| !c.is_whitespace())
                .skip_while(|(_, c)| c.is_whitespace())
                .next()
            {
                self.cursor_position.1 = self.cursor_position.1.saturating_sub(prev_word.0 + 1);
            } else {
                self.cursor_position.1 = 0;
            }
        }
    }

    // Record a change that can be undone
    fn record_change(&mut self, change: BufferChange) {
        self.undo_stack.push((change, self.cursor_position));
        self.redo_stack.clear(); // Clear redo stack when new change is made
    }

    // Undo last change
    pub fn undo(&mut self) -> bool {
        if let Some((change, cursor_pos)) = self.undo_stack.pop() {
            let reverse_change = match change {
                BufferChange::Insert { position, content } => {
                    // For insert, remove the inserted content
                    let (row, col) = position;
                    let line = &mut self.content[row];
                    let end_col = col + content.len();
                    line.replace_range(col..end_col, "");
                    BufferChange::Delete { position, content }
                }
                BufferChange::Delete { position, content } => {
                    // For delete, reinsert the deleted content
                    let (row, col) = position;
                    let line = &mut self.content[row];
                    line.insert_str(col, &content);
                    BufferChange::Insert { position, content }
                }
                BufferChange::NewLine { position, content } => {
                    // For newline, join the lines back
                    let (row, _) = position;
                    let next_line = self.content.remove(row + 1);
                    self.content[row].push_str(&next_line);
                    BufferChange::DeleteLine { position: row, content }
                }
                BufferChange::DeleteLine { position, content } => {
                    // For line deletion, reinsert the line
                    self.content.insert(position, content.clone());
                    BufferChange::NewLine { 
                        position: (position, 0),
                        content 
                    }
                }
            };
            
            // Save the reverse change to redo stack
            self.redo_stack.push((reverse_change, self.cursor_position));
            // Restore cursor position
            self.cursor_position = cursor_pos;
            true
        } else {
            false
        }
    }

    // Redo last undone change
    pub fn redo(&mut self) -> bool {
        if let Some((change, cursor_pos)) = self.redo_stack.pop() {
            let reverse_change = match change {
                BufferChange::Insert { position, content } => {
                    let (row, col) = position;
                    let line = &mut self.content[row];
                    let end_col = col + content.len();
                    line.replace_range(col..end_col, "");
                    BufferChange::Delete { position, content }
                }
                BufferChange::Delete { position, content } => {
                    let (row, col) = position;
                    let line = &mut self.content[row];
                    line.insert_str(col, &content);
                    BufferChange::Insert { position, content }
                }
                BufferChange::NewLine { position, content } => {
                    let (row, _) = position;
                    let next_line = self.content.remove(row + 1);
                    self.content[row].push_str(&next_line);
                    BufferChange::DeleteLine { position: row, content }
                }
                BufferChange::DeleteLine { position, content } => {
                    self.content.insert(position, content.clone());
                    BufferChange::NewLine { 
                        position: (position, 0),
                        content 
                    }
                }
            };
            
            // Save the reverse change to undo stack
            self.undo_stack.push((reverse_change, self.cursor_position));
            // Restore cursor position
            self.cursor_position = cursor_pos;
            true
        } else {
            false
        }
    }

    // page movement operations
    pub fn move_page_up(&mut self) {
        // Move up by screen height (configurable)
        for _ in 0..20 { // Default set to 20 lines, could be made configurable
            if self.cursor_position.0 > 0 {
                self.move_cursor("up");
            }
        }
    }

    pub fn move_page_down(&mut self) {
        // Move down by screen height (configurable)
        for _ in 0..20 { // Default set to 20 lines, could be made configurable
            if self.cursor_position.0 < self.content.len() - 1 {
                self.move_cursor("down");
            }
        }
    }

    // Indentation operations
    pub fn indent_line(&mut self, size: usize) {
        let spaces = " ".repeat(size);
        self.content[self.cursor_position.0].insert_str(0, &spaces);
        self.cursor_position.1 += size;
    }

    pub fn dedent_line(&mut self, size: usize) {
        let line = &mut self.content[self.cursor_position.0];
        let whitespace_count = line.chars()
            .take_while(|c| c.is_whitespace())
            .count();
        let remove_count = whitespace_count.min(size);
        if remove_count > 0 {
            line.replace_range(0..remove_count, "");
            self.cursor_position.1 = self.cursor_position.1.saturating_sub(remove_count);
        }
    }

    pub fn delete_word_backward(&mut self) {
        let start_pos = self.cursor_position.1;
        self.move_word_backward();
        let end_pos = self.cursor_position.1;
        if start_pos > end_pos {
            self.content[self.cursor_position.0]
                .replace_range(end_pos..start_pos, "");
        }
    }

    pub fn delete_to_line_start(&mut self) {
        let line = &mut self.content[self.cursor_position.0];
        line.replace_range(0..self.cursor_position.1, "");
        self.cursor_position.1 = 0;
    }

    // Text insertion at cursor
    pub fn insert_text(&mut self, text: &str) {
        let current_line = &mut self.content[self.cursor_position.0];
        current_line.insert_str(self.cursor_position.1, text);
        self.cursor_position.1 += text.len();
    }

    pub fn paste_at_cursor(&mut self, text: &str) {
        for line in text.lines() {
            self.insert_text(line);
            if text.contains('\n') {
                self.insert_newline_auto_indent();
            }
        }
    }

    // === Insert Mode Entry Preparations ===
    
    // Handle 'a' - appen after cursor
    pub fn prepare_append(&mut self) {
        if !self.cotnent[self.cursor_position.0].is_empty() {
            self.move_cursor("right");
        }
    }

    // Handle 'A' - append at end of line
    pub fn prepare_append_end_of_line(&mut self) {
        self.cursor_position.1 = self.content[self.cursor_position.0].len();
    }

    // Handle 'I' - insert at start of line (after whitespace)
    pub fn prepare_insert_start_of_line(&mut self) {
        let line = &self.content[self.cursor_position.0];
        if let Some(first_non_space) = line.chars().position(|c| !c.is_whitespace()) {
            self.cursor_position.1 = first_non_space;
        } else {
            self.cursor_position.1 = 0;
        }
    }

    // Handle 'o' - open line below
    pub fn insert_line_below(&mut self) {
        let current_indent = self.get_line_indentation(self.cursor_position.0);
        self.cursor_position.0 += 1;
        self.content.insert(self.cursor_position.0, current_indent);
        self.cursor_position.1 = current_indent.len();
    }

    // Handle 'O'- open line above
    pub fn insert_line_above(&mut self) {
        let current_indent = self.get_line_indentation(self.cursor_position.0);
        self.content.insert(self.cursor_position.0, current_indent);
        self.cursor_position.1 = current_indent.len();
    }

    // Cursor operations
    pub fn get_cursor_position(&self) -> (usize, usize) {
        self.cursor_position
    }

    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        if row < self.content.len() {
            self.cursor_position.0 = row;
            self.cursor_position.1 = col.min(self.content[row].len());
        }
    }

    pub fn move_cursor(&mut self, direction: &str) {
        match direction {
            "left" => {
                if self.cursor_position.1 > 0 {
                    self.cursor_position.1 -= 1;
                }
            }
            "right" => {
                if self.cursor_position.1 < self.content[self.cursor_position.0].len() {
                    self.cursor_position.1 += 1;
                }
            }
            "up" => {
                if self.cursor_position.0 > 0 {
                    self.cursor_position.0 -= 1;
                    self.cursor_position.1 = self.content[self.cursor_position.0]
                        .len()
                        .min(self.cursor_position.1);
                }
            }
            "down" => {
                if self.cursor_position.0 + 1 < self.content.len() {
                    self.cursor_position.0 += 1;
                    self.cursor_position.1 = self.content[self.cursor_position.0]
                        .len()
                        .min(self.cursor_position.1);
                }
            }
            "top" => {
                self.cursor_position = (0, 0);
            }
            "bottom" => {
                self.cursor_position.0 = self.content.len().saturating_sub(1);
                self.cursor_position.1 = 0;
            }
            "line_start" => {
                self.cursor_position.1 = 0;
            }
            "line_end" => {
                self.cursor_position.1 = self.content[self.cursor_position.0].len();
            }
            _ => {}
        }
    }

    // === Enhanced Text Operations ===

    // Insert character with replace mode support
    pub fn inser_char_replace(&mut self, c: char) {
        let current_line = &mut self.content[self.cursor_position.0];
        if self.cursor_position.1 < current_line.len() {
            // Replace existing character
            current_line.replace_range(self.cursor_position.1..=self.cursor_position.1, &c.to_string());
        } else {
            // Append if at end of line
            current_line.push(c);
        }
        self.cursor_position.1 += 1;
    }

    // Newline handling with auto-indent
    pub fn insert_newline_auto_indent(&mut self) {
        let current_line = self.cursor_position.0;
        let current_indent = self.get_line_indentation(current_line);
        let remainder = self.content[current_line][self.cursor_position.1..].to_string();

        let change = BufferChange::NewLine {
            position: (current_line, self.cursor_position.1),
            content: remainder.clone(),
        };

        // Update current line to end at cursor
        self.content[current_line] = self.content[current_line][..self.cursor_position.1].to_string();

        // Insert new line with indentation
        self.cursor_position.0 += 1;
        self.content.insert(
            self.cursor_position.0,
            format!("{}{}", current_indent, remainder)
        );
        self.cursor_position.1 = current_indent.len();
        
        self.record_change(change);
    }

    // Helper for getting line indentation
    fn get_line_indentation(&self, line_number: usize) -> String {
        if let Some(line) = self.content.get(line_number) {
            let whitespace_count = line.chars().take_while(|c| c.is_whitespace()).count();
            line[..whitespace_count].to_string()
        } else {
            String::new()
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position.1 > 0 {
            let line = &mut self.content[self.cursor_position.0];
            let deleted = line.remove(self.cursor_position.1 - 1);
            let change = BufferChange::Delete {
                position: (self.cursor_position.0, self.cursor_position.1 - 1),
                content: deleted.to_string(),
            };
            self.cursor_position.1 -= 1;
            self.record_change(change);
        } else if self.cursor_position.0 > 0 {
            let current_line = self.content.remove(self.cursor_position.0);
            let change = BufferChange::DeleteLine {
                position: self.cursor_position.0,
                content: current_line.clone(),
            };
            self.cursor_position.0 -= 1;
            self.cursor_position.1 = self.content[self.cursor_position.0].len();
            self.content[self.cursor_position.0].push_str(&current_line);
            self.record_change(change);
        }
    }

    pub fn cut_char(&mut self) {
        if let Some(line) = self.content.get_mut(self.cursor_position.0) {
            if self.cursor_position.1 < line.len() {
                // Cut character at cursor
                let cut_char = line.remove(self.cursor_position.1);
                // Store in clipboard
                if let Some(clipboard) = &mut self.clipboard {
                    clipboard.yank(cut_char.to_string());
                }
                // Don't move cursor back since we're cutting at cursor position
            } else if self.cursor_position.0 < self.content.len() - 1 {
                // At end of line, joing with next line if it exists
                let next_line = self.content.remove(self.cursor_position.0 + 1);
                self.content[self.cursor_position.0].push_str(&next_line);
            }
        }
    }

    // Method to handle forward delete (Delete Key)
    pub fn delete_char_fn(&mut self) {
        if let Some(line) = self.content.get_mut(self.cursor_position.0) {
            if self.cursor_position.1 < line.len() {
                // Delete character at cursor
                line.remove(self.cursor_position.1);
                // Cursor position stays the same
            } else if self.cursor_position.0 < self.content.len() - 1 {
                // At end of line, joing with next line if it exists
                let next_line = self.content.remove(self.cursor_position.0 + 1);
                self.content[self.cursor_position.0].push_str(&next_line);
            }
        }
    }

    pub fn insert_line(&mut self) {
        let current_line = self.content[self.cursor_position.0]
            .split_off(self.cursor_position.1);
        self.content
            .insert(self.cursor_position.0 + 1, current_line);
        self.cursor_position.0 += 1;
        self.cursor_position.1 = 0;
    }

    pub fn delete_line(&mut self) {
        if self.content.len() > 1 {
            self.content.remove(self.cursor_position.0);
            if self.cursor_position.0 >= self.content.len() {
                self.cursor_position.0 = self.content.len() - 1;
            }
            self.cursor_position.1 = 0;
        } else {
            self.content[0].clear();
            self.cursor_position = (0, 0);
        }
    }

    // Visual selection methods
    pub fn start_visual(&mut self) {
        self.visual_start = Some(self.cursor_position);
    }

    pub fn clear_visual(&mut self) {
        self.visual_start = None;
    }

    pub fn get_visual_selection(&self) -> Option<((usize, usize), (usize, usize))> {
        self.visual_start.map(|start| {
            let end = self.cursor_position;
            (start, end)
        })
    }

    pub fn get_selected_text(&self) -> Option<String> {
        self.get_visual_selection().map(|(start, end)| {
            let start_row = start.0.min(end.0);
            let end_row = start.0.max(end.0);
            let mut selected = String::new();

            for row in start_row..=end_row {
                if row == start_row && row == end_row {
                    let start_col = start.1.min(end.1);
                    let end_col = start.1.max(end.1);
                    selected.push_str(&self.content[row][start_col..end_col]);
                } else if row == start_row {
                    let start_col = if start.0 == start_row { start.1 } else { end.1 };
                    selected.push_str(&self.content[row][start_col..]);
                } else if row == end_row {
                    let end_col = if end.0 == end_row { end.1 } else { start.1 };
                    selected.push_str(&self.content[row][..end_col]);
                } else {
                    selected.push_str(&self.content[row]);
                }
                if row != end_row {
                    selected.push('\n');
                }
            }
            selected
        })
    }

    // Buffer content access
    pub fn get_line(&self, index: usize) -> Option<&String> {
        self.content.get(index)
    }

    pub fn get_current_line(&self) -> Option<&String> {
        self.content.get(self.cursor_position.0)
    }

    pub fn get_content(&self) -> &Vec<String> {
        &self.content
    }

    pub fn line_count(&self) -> usize {
        self.content.len()
    }

    pub fn insert_at(&mut self, row: usize, content: String) {
        if row <= self.content.len() {
            self.content.insert(row, content);
        }
    }

    pub fn replace_line(&mut self, row: usize, content: String) {
        if row < self.content.len() {
            self.content[row] = content;
        }
    }

    // Search-related methods
    pub fn search(&mut self, query: &str, case_sensitive: bool) -> usize {
        self.search_matches.clear();
        self.current_match = None;

        if query.is_empty() {
            return 0;
        }

        for (row, line) in self.content.iter().enumerate() {
            let line_to_search = if case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };
            let query_to_search = if case_sensitive {
                query.to_string()
            } else {
                query.to_lowercase()
            };

            let mut start_idx = 0;
            while let Some(found_idx) = line_to_search[start_idx..].find(&query_to_search) {
                let abs_idx = start_idx + found_idx;
                self.search_matches.push((row, abs_idx, abs_idx + query.len()));
                start_idx = abs_idx + 1;
            }
        }

        // if we found matches, select the first one
        if !self.search_matches.is_empty() {
            self.current_match = Some(0);
            self.jump_to_current_match();
        }

        self.search_matches.len()
    }

    pub fn next_match(&mut self) -> bool {
        if let Some(current) = self.current_match {
            if current + 1 < self.search_matches.len() {
                self.current_match = Some(current + 1);
                self.jump_to_current_match();
                return true;
            }
        }
        false
    }

    pub fn previous_match(&mut self) -> bool {
        if let Some(current) = self.current_match {
            if current > 0 {
                self.current_match = Some(current - 1);
                self.jump_to_current_match();
                return true;
            }
        }
        false
    }

    fn jump_to_current_match(&mut self) {
        if let Some(current) = self.current_match {
            if let Some(&(row, col, _)) = self.search_matches.get(current) {
                self.cursor_position = (row, col);
            }
        }
    }

    pub fn clear_search(&mut self) {
        self.search_matches.clear();
        self.current_match = None;
    }

    // Rendering
    pub fn render_lines(&self) -> Vec<String> {
        let mut rendered = self.content.clone();

        // Add search highlighting
        for (row, line) in rendered.iter_mut().enumerate() {
            let mut offset = 0;
            let matches_in_line: Vec<_> = self.search_matches.iter()
                .filter(|&&(match_row, _, _)| match_row == row)
                .collect();

            for &(_, start_col, end_col) in matches_in_line {
                let start_idx = start_col + offset;
                let end_idx = end_col + offset;
                let highligh = if Some(start_col) == self.current_match.map(|i| self.matches[i].1) {
                    "\x1b[43m" // Yellow background for current match
                } else {
                    "\x1b[42m" // Green background for other matches
                };
                let highlighted = format!("{}{}\x1b[0m",
                    highlight,
                    &line[start_col..end_col]
                );
                line.replace_range(start_idx..end_idx, &highlighted);
                offset += highlighted.len() - (end_col - start_col);
            }
        }

        rendered
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:4} | {}", i + 1, line))
            .collect()
    }

    pub fn render_lines_with_visual(&self) -> Vec<String> {
        let mut rendered = self.content.clone();
        
        // First, apply search highlighting
        for (row, line) in rendered.iter_mut().enumerate() {
            let mut offset = 0;
            let matches_in_line: Vec<_> = self.search_matches.iter()
                .filter(|&&(match_row, _, _)| match_row == row)
                .collect();

            for &(_, start_col, end_col) in matches_in_line {
                let start_idx = start_col + offset;
                let end_idx = end_col + offset;
                let highlight = if Some(start_col) == self.current_match.map(|i| self.search_matches[i].1) {
                    "\x1b[43m" // Yellow background for current match
                } else {
                    "\x1b[42m" // Green background for other matches
                };
                let highlighted = format!("{}{}\x1b[0m", 
                    highlight,
                    &line[start_col..end_col]
                );
                line.replace_range(start_idx..end_idx, &highlighted);
                offset += highlighted.len() - (end_col - start_col);
            }
        }
        
        // Then apply visual selection highlighting
        if let Some((start_row, start_col)) = self.visual_start {
            let end_row = self.cursor_position.0.max(start_row);
            let start_row = self.cursor_position.0.min(start_row);

            for row in start_row..=end_row {
                if row >= rendered.len() {
                    break;
                }

                let line = &mut rendered[row];
                
                if row == start_row && row == end_row {
                    // Single line selection
                    let (start, end) = if start_col <= self.cursor_position.1 {
                        (start_col, self.cursor_position.1)
                    } else {
                        (self.cursor_position.1, start_col)
                    };
                    
                    // Ensure we don't go past the line length
                    let end = end.min(line.len());
                    if start < line.len() {
                        let selected_text = &line[start..end];
                        // Use inverse video for visual selection
                        let highlighted = format!("\x1b[7m{}\x1b[0m", selected_text);
                        line.replace_range(start..end, &highlighted);
                    }
                } else {
                    // Full line selection
                    let highlighted = format!("\x1b[7m{}\x1b[0m", line);
                    *line = highlighted;
                }
            }
        }

        // Add line numbers and return
        rendered
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:4} | {}", i + 1, line))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = Buffer::new();
        assert_eq!(buffer.cursor_position, (0, 0));
        assert_eq!(buffer.content, vec![String::new()]);
        assert_eq!(buffer.tab_size, 4);
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = Buffer::new();
        buffer.insert_char('a');
        assert_eq!(buffer.content[0], "a");
        assert_eq!(buffer.cursor_position, (0, 1));
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = Buffer::new();
        buffer.insert_char('a');
        buffer.delete_char();
        assert_eq!(buffer.content[0], "");
        assert_eq!(buffer.cursor_position, (0, 0));
    }

    #[test]
    fn test_insert_line() {
        let mut buffer = Buffer::new();
        buffer.insert_char('a');
        buffer.insert_line();
        assert_eq!(buffer.content.len(), 2);
        assert_eq!(buffer.content[0], "");
        assert_eq!(buffer.content[1], "a");
    }

    #[test]
    fn test_cursor_movement() {
        let mut buffer = Buffer::new();
        buffer.insert_char('a');
        buffer.insert_line();
        buffer.insert_char('b');
        
        buffer.move_cursor("up");
        assert_eq!(buffer.cursor_position, (0, 1));
        
        buffer.move_cursor("down");
        assert_eq!(buffer.cursor_position, (1, 1));
        
        buffer.move_cursor("left");
        assert_eq!(buffer.cursor_position, (1, 0));
        
        buffer.move_cursor("right");
        assert_eq!(buffer.cursor_position, (1, 1));
    }

    #[test]
    fn test_visual_selection() {
        let mut buffer = Buffer::new();
        buffer.insert_char('a');
        buffer.insert_char('b');
        buffer.insert_char('c');
        
        buffer.cursor_position = (0, 0);
        buffer.start_visual();
        buffer.cursor_position = (0, 3);
        
        let selected_text = buffer.get_selected_text().unwrap();
        assert_eq!(selected_text, "abc");
        
        buffer.clear_visual();
        assert_eq!(buffer.get_visual_selection(), None);
    }

    #[test]
    fn test_multiline_visual_selection() {
        let mut buffer = Buffer::new();
        buffer.insert_char('a');
        buffer.insert_line();
        buffer.insert_char('b');
        buffer.insert_line();
        buffer.insert_char('c');

        buffer.cursor_position = (0, 0);
        buffer.start_visual();
        buffer.cursor_position = (2, 1);

        let selected_text = buffer.get_selected_text().unwrap();
        assert_eq!(selected_text, "a\nb\nc");
    }

    #[test]
    fn test_prepare_append() {
        let mut buffer = Buffer::new();
        buffer.insert_char('a');
        buffer.cursor_position.1 = 0;
        buffer.prepare_append();
        assert_eq!(buffer.cursor_position.1, 1);
    }

    #[test]
    fn test_prepare_append_end_of_line() {
        let mut buffer = Buffer::new();
        buffer.insert_char('a');
        buffer.insert_char('b');
        buffer.cursor_position.1 = 0;
        buffer.prepare_append_end_of_line();
        assert_eq!(buffer.cursor_position.1, 2);
    }

    #[test]
    fn test_prepare_insert_start_of_line() {
        let mut buffer = Buffer::new();
        buffer.content[0] = "    text".to_string();
        buffer.cursor_position.1 = 6;
        buffer.prepare_insert_start_of_line();
        assert_eq!(buffer.cursor_position.1, 4); // Should move to first non-space char
    }

    #[test]
    fn test_insert_line_below() {
        let mut buffer = Buffer::new();
        buffer.content[0] = "    first line".to_string();
        buffer.insert_line_below();
        assert_eq!(buffer.content.len(), 2);
        assert_eq!(buffer.content[1], "    ");
        assert_eq!(buffer.cursor_position, (1, 4));
    }

    #[test]
    fn test_insert_line_above() {
        let mut buffer = Buffer::new();
        buffer.content[0] = "    first line".to_string();
        buffer.insert_line_above();
        assert_eq!(buffer.content.len(), 2);
        assert_eq!(buffer.content[0], "    ");
        assert_eq!(buffer.content[1], "    first line");
        assert_eq!(buffer.cursor_position, (0, 4));
    }

    #[test]
    fn test_insert_char_replace() {
        let mut buffer = Buffer::new();
        buffer.insert_char('a');
        buffer.insert_char('b');
        buffer.cursor_position.1 = 0;
        buffer.insert_char_replace('x');
        assert_eq!(buffer.content[0], "xb");
        assert_eq!(buffer.cursor_position.1, 1);
    }

    #[test]
    fn test_insert_newline_auto_indent() {
        let mut buffer = Buffer::new();
        buffer.content[0] = "    first line".to_string();
        buffer.cursor_position = (0, 8);
        buffer.insert_newline_auto_indent();
        assert_eq!(buffer.content[0], "    firs");
        assert_eq!(buffer.content[1], "    t line");
        assert_eq!(buffer.cursor_position, (1, 4));
    }

    #[test]
    fn test_search_basic() {
        let mut buffer = Buffer::new();
        buffer.content = vec![
            "first line".to_string(),
            "second line".to_string(),
            "third line".to_string(),
        ];

        let matches = buffer.search("line", true);
        assert_eq!(matches, 3);
        assert_eq!(buffer.search_matches.len(), 3);
        assert_eq!(buffer.current_match, Some(0));
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut buffer = Buffer::new();
        buffer.content = vec!["UPPER LINE".to_string(), "lower line".to_string()];

        let matches = buffer.search("line", false);
        assert_eq!(matches, 2);
    }

    #[test]
    fn test_search_navigation() {
        let mut buffer = Buffer::new();
        buffer.content = vec![
            "line one".to_string(),
            "line two".to_string(),
        ];

        buffer.search("line", true);
        assert_eq!(buffer.cursor_position, (0, 0)); // First match
        
        buffer.next_match();
        assert_eq!(buffer.cursor_position, (1, 0)); // Second match

        buffer.previous_match();
        assert_eq!(buffer.cursor_position, (0, 0)); // Back to first match
    }

    #[test]
    fn test_render_with_search_and_visual() {
        let mut buffer = Buffer::new();
        buffer.content = vec![
            "first line".to_string(),
            "second line".to_string(),
            "third line".to_string(),
        ];

        // Add some search matches
        buffer.search("line", true);
        
        // Add visual selection
        buffer.cursor_position = (0, 0);
        buffer.start_visual();
        buffer.cursor_position = (1, 5);

        let rendered = buffer.render_lines_with_visual();
        assert!(rendered[0].contains("\x1b[7m")); // Visual selection
        assert!(rendered[0].contains("\x1b[42m")); // Search highlight
        assert!(rendered[1].contains("\x1b[7m")); // Visual selection
        assert!(rendered[1].contains("\x1b[42m")); // Search highlight
    }

    #[test]
    fn test_render_with_current_search_match() {
        let mut buffer = Buffer::new();
        buffer.content = vec![
            "test line".to_string(),
            "test line".to_string(),
        ];

        buffer.search("test", true);
        
        let rendered = buffer.render_lines_with_visual();
        assert!(rendered[0].contains("\x1b[43m")); // Current match highlight
        assert!(rendered[1].contains("\x1b[42m")); // Other match highlight
    }

    #[test]
    fn test_render_with_overlapping_highlights() {
        let mut buffer = Buffer::new();
        buffer.content = vec!["test line test".to_string()];

        // Add search match
        buffer.search("test", true);
        
        // Add visual selection that overlaps with search match
        buffer.cursor_position = (0, 0);
        buffer.start_visual();
        buffer.cursor_position = (0, 6);

        let rendered = buffer.render_lines_with_visual();
        // Both search highlight and visual selection should be visible
        assert!(rendered[0].contains("\x1b[42m")); // Search highlight
        assert!(rendered[0].contains("\x1b[7m")); // Visual selection
    }
}