#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert(InsertVariant),
    Visual,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandType {
    Regular,    // For ':' commands
    Search,     // For '/' search
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InsertVariant {
    Insert,     // 'i'
    Append,     // 'a'
    AppendEnd,  // 'A'
    LineStart,  // 'I'
    LineBelow,  // 'o'
    LineAbove,  // 'O'
    Replace,    // 'R'
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModeTrigger {
    // Mode changes
    Escape,             
    Enter,
    Undo,               // 'u' - Undo last change
    Redo,               // <Ctrl-r> - Redo last undone change
    
    // Insert mode triggers
    InsertNormal,       // 'i'
    InsertAppend,       // 'a'
    InsertAppendEnd,    // 'A'
    InsertLineStart,    // 'I'
    InsertLineBelow,    // 'o'
    InsertLineAbove,    // 'O'
    InsertReplace,      // 'R'
    
    // Other modes
    VisualMode,         // 'v'
    CommandMode,        // ':'
    SearchMode,         // '/'

    // Delete Cut Operations
    DeleteChar,         // for backspace/delete key
    CutChar,            // for 'x' in normal mode
    DeleteForward,      // For delete key specifically

    // Insert Mode special operations
    InsertTab,          // <Tab> - Insert tab or spaces
    InsertBackTab,      // <Shift-Tab> - Remove one level of indentation
    InsertNewLine,      // <Enter> in insert mode - Add new line with auto-indent
    InsertWordForward,  // <Ctrl-Right> - Move cursor forward one word
    InsertWordBackward, // <Ctrl-Left> - Move cursor backward one word
    InsertDeleteWord,   // <Ctrl-w> - Delete word before cursor
    InsertDeleteLine,   // <Ctrl-u> - Delete from cursor to start of line
    InsertPageUp,       // <PageUp> - Move cursor up one screen
    InsertPageDown,     // <PageDown> - Move cursor down one screen
    InsertHome,         // <Home> - Move cursor to start of line
    InsertEnd,          // <End> - Move cursor to end of line
}

impl Mode {
    pub fn default() -> Self {
        Mode::Normal
    }

    // Method to check if delete operations are allowed
    pub fn allows_deletion(&self) -> bool {
        match self {
            Mode::Normal => true,
            Mode::Insert(_) => true,
            Mode::Visual => false, // visual mode has own deletion handling
            Mode::Command(_) => true, // Allow backspace in command mode
        }
    }

    // Method to check if cut operations are allowed
    pub fn allows_cut(&self) -> bool {
        match self {
            Mode::Normal => true,
            Mode::Visual => true, // Visual mode allows cutting selections
            _ => false,
        }
    }

    // Method to check if word-wise movement is allowed
    pub fn allow_word_movement(&self) -> bool {
        match self {
            Mode::Insert(_) => true,
            Mode::Normal => true,
            Mode::Visual => true,
            Mode::Command(_) => false,
        }
    }

    // Method to check if page movement is allowed
    pub fn allows_page_movement(&self) -> bool {
        match self {
            Mode::Insert(_) => true,
            Mode::Normal => true,
            Mode::Visual => true,
            Mode::Command(_) => false,
        }
    }

    pub fn allows_indent(&self) -> bool {
        match self {
            Mode::Insert(_) => true,
            Mode::Normal => true,
            _ => false,
        }
    }

    // Method to check if undo/redo is allowed
    pub fn allows_undo(&self) -> bool {
        matches!(self, Mode::Normal)
    }

    pub fn transition(&self, trigger: ModeTrigger) -> Mode {
        match (self, trigger) {
            // Escape always returns to Normal mode
            (_, ModeTrigger::Escape) => Mode::Normal,
            
            // Normal mode transitions
            (Mode::Normal, ModeTrigger::InsertNormal) => Mode::Insert(InsertVariant::Insert),
            (Mode::Normal, ModeTrigger::InsertAppend) => Mode::Insert(InsertVariant::Append),
            (Mode::Normal, ModeTrigger::InsertAppendEnd) => Mode::Insert(InsertVariant::AppendEnd),
            (Mode::Normal, ModeTrigger::InsertLineStart) => Mode::Insert(InsertVariant::LineStart),
            (Mode::Normal, ModeTrigger::InsertLineBelow) => Mode::Insert(InsertVariant::LineBelow),
            (Mode::Normal, ModeTrigger::InsertLineAbove) => Mode::Insert(InsertVariant::LineAbove),
            (Mode::Normal, ModeTrigger::InsertReplace) => Mode::Insert(InsertVariant::Replace),
            (Mode::Normal, ModeTrigger::VisualMode) => Mode::Visual,
            (Mode::Normal, ModeTrigger::CommandMode) => Mode::Command,
            (Mode::Normal, ModeTrigger::SearchMode) => Mode::Command(CommandType::Search),
            (Mode::Normal, ModeTrigger::CutChar) => Mode::Normal,
            (Mode::Normal, ModeTrigger::Undo) => Mode::Normal,
            (Mode::Normal, ModeTrigger::Redo) => Mode::Normal,
            (Mode::Insert(_), ModeTrigger::DeleteChar) => *self,
            (Mode::Insert(_), ModeTrigger::DeleteForward) => *self,
            (Mode::Insert(_), ModeTrigger::InsertTab) => *self,
            (Mode::Insert(_), ModeTrigger::InsertBackTab) => *self,
            (Mode::Insert(_), ModeTrigger::InsertNewLine) => *self,
            (Mode::Insert(_), ModeTrigger::WordForward) => *self,
            (Mode::Insert(_), ModeTrigger::DeleteWord) => *self,
            (Mode::Insert(_), ModeTrigger::DeleteLine) => *self,
            (Mode::Command(_), ModeTrigger::DeleteChar) => *self,
            
            // Command mode specific
            (Mode::Command, ModeTrigger::Enter) => Mode::Normal,
            
            // Stay in current mode for unhandled transitions
            (current, _) => *current,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert(variant) => match variant {
                InsertVariant::Insert => "INSERT",
                InsertVariant::Append => "INSERT (APPEND)",
                InsertVariant::AppendEnd => "INSERT (APPEND END)",
                InsertVariant::LineStart => "INSERT (LINE START)",
                InsertVariant::LineBelow => "INSERT (BELOW)",
                InsertVariant::LineAbove => "INSERT (ABOVE)",
                InsertVariant::Replace => "REPLACE",
            },
            Mode::Visual => "VISUAL",
            Mode::Command(cmd_type) => match cmd_type {
                Mode::Command => "COMMAND",
                Mode::Command => "SEARCH",
            }
        }
    }

    pub fn allows_text_input(&self) -> bool {
        matches!(self, Mode::Insert(_) | Mode::Command)
    }

    pub fn allows_cursor_movement(&self) -> bool {
        !matches!(self, Mode::Command)
    }

    pub fn cursor_style(&self) -> CursorStyle {
        match self {
            Mode::Insert(InsertVariant::Replace) => CursorStyle::Block,
            Mode::Insert(_) => CursorStyle::Line,
            Mode::Command => CursorStyle::Line,
            _ => CursorStyle::Block,
        }
    }

    pub fn should_prepare_insert(&self) -> Option<InsertVariant> {
        match self {
            Mode::Insert(variant) => Some(*variant),
            _ => None,
        }
    }

    pub fn is_search_mode(&self) -> bool {
        matches!(self, Mode::Command(CommandType::Search))
    }

    pub fn command_prefix(&self) -> &str {
        match self {
            Mode::Command(CommandType::Regular) => ":",
            Mode::Command(CommandType::Search) => "/",
            _ => "",
        }
    }

    /// Get the appropriate cursor style for the current insert variant
    pub fn insert_cursor_style(&self) -> CursorStyle {
        match self {
            Mode::Insert(InsertVariant::Replace) => CursorStyle::Block,
            Mode::Insert(_) => CursorStyle::Line,
            _ => CursorStyle::Block,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorStyle {
    Block,
    Line,
    Underline,
}

impl CursorStyle {
    pub fn as_str(&self) -> &str {
        match self {
            CursorStyle::Block => "█",
            CursorStyle::Line => "│",
            CursorStyle::Underline => "_",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_transitions() {
        let normal = Mode::Normal;
        
        // Test insert mode variations
        assert_eq!(
            normal.transition(ModeTrigger::InsertNormal),
            Mode::Insert(InsertVariant::Insert)
        );
        assert_eq!(
            normal.transition(ModeTrigger::InsertAppend),
            Mode::Insert(InsertVariant::Append)
        );
        assert_eq!(
            normal.transition(ModeTrigger::InsertAppendEnd),
            Mode::Insert(InsertVariant::AppendEnd)
        );
        
        // Test other mode transitions
        assert_eq!(normal.transition(ModeTrigger::VisualMode), Mode::Visual);
        assert_eq!(normal.transition(ModeTrigger::CommandMode), Mode::Command);
    }

    #[test]
    fn test_escape_from_all_modes() {
        let modes = [
            Mode::Insert(InsertVariant::Insert),
            Mode::Visual,
            Mode::Command,
        ];

        for mode in modes.iter() {
            assert_eq!(mode.transition(ModeTrigger::Escape), Mode::Normal);
        }
    }

    #[test]
    fn test_display_names() {
        assert_eq!(Mode::Normal.display_name(), "NORMAL");
        assert_eq!(
            Mode::Insert(InsertVariant::Append).display_name(),
            "INSERT (APPEND)"
        );
        assert_eq!(Mode::Visual.display_name(), "VISUAL");
        assert_eq!(Mode::Command.display_name(), "COMMAND");
    }

    #[test]
    fn test_text_input_permissions() {
        assert!(!Mode::Normal.allows_text_input());
        assert!(Mode::Insert(InsertVariant::Insert).allows_text_input());
        assert!(Mode::Command.allows_text_input());
        assert!(!Mode::Visual.allows_text_input());
    }

    #[test]
    fn test_cursor_styles() {
        assert_eq!(Mode::Normal.cursor_style(), CursorStyle::Block);
        assert_eq!(
            Mode::Insert(InsertVariant::Insert).cursor_style(),
            CursorStyle::Line
        );
        assert_eq!(
            Mode::Insert(InsertVariant::Replace).cursor_style(),
            CursorStyle::Block
        );
    }

    #[test]
    fn test_search_mode_transition() {
        let normal = Mode::Normal;
        assert_eq!(
            normal.transition(ModeTrigger::SearchMode),
            Mode::Command(CommandType::Search)
        );
    }

    #[test]
    fn test_command_prefixes() {
        assert_eq!(Mode::Command(CommandType::Regular).command_prefix(), ":");
        assert_eq!(Mode::Command(CommandType::Search).command_prefix(), "/");
    }

    #[test]
    fn test_is_search_mode() {
        assert!(Mode::Command(CommandType::Search).is_search_mode());
        assert!(!Mode::Command(CommandType::Regular).is_search_mode());
        assert!(!Mode::Normal.is_search_mode());
    }
}