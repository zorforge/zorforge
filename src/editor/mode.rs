// src/editor/mode.rs

/// Represents the current editing mode of the editor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,                 // Default mode for navigation and commands
    Insert(InsertVariant),  // Text insertion modes
    Visual(VisualVariant),  // Selection modes
    Command(CommandType),   // Command and search modes
}

/// Variants of command mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandType {
    Regular,    // For ':' commands
    Search,     // For '/' forward search
    Backward,   // For '?' backward search
}

/// Variants of insert mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InsertVariant {
    Insert,     // 'i' - Insert at cursor
    Append,     // 'a' - Append after cursor
    AppendEnd,  // 'A' - Append at end of line
    LineStart,  // 'I' - Insert at first non-whitespace character
    LineBelow,  // 'o' - Open new line below
    LineAbove,  // 'O' - Open new line above
    Replace,    // 'R' - Replace existing text
}

/// Variants of visual mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualVariant {
    Char,       // 'v' - Character-wise selection
    Line,       // 'V' - Line-wise selection
    Block,      // <Ctrl-v> - Block-wise selection
}

/// Input triggers that can cause mode transitions or actions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModeTrigger {
    // Common Movement Operations
    ArrowLeft,          // <Left>          - Move cursor left
    ArrowRight,         // <Right>         - Move cursor right
    ArrowUp,            // <Up>            - Move cursor up
    ArrowDown,          // <Down>          - Move cursor down
    Home,               // <Home>          - Move to start of line
    End,                // <End>           - Move to end of line

    // Modern System Clipboard Operations
    SystemCopy,         // <Ctrl-Shift-C>  - Copy to system clipboard
    SystemPaste,        // <Ctrl-Shift-V>  - Paste from system clipboard
    SystemCut,          // <Ctrl-Shift-X>  - Cut to system clipboard

    // Mouse Operations
    MouseClick,         // Left click      - Move cursor to click position
    MouseDrag,          // Click and drag  - Start visual selection
    MouseScroll,        // Mouse wheel     - Scroll viewport
    MouseDoubleClick,   // Double click    - Select word under cursor
    MouseTripleClick,   // Triple click    - Select line under cursor

    // Global Mode Changes
    Escape,             // <Esc>           - Return to Normal mode from any mode
    Enter,              // <Enter>         - Execute command or confirm action
    Quit,               // :q, ZZ          - Quit the editor
    QuitForce,          // :q!, ZQ         - Force quit without saving
    
    // Normal Mode -> Other Modes
    InsertNormal,       // i               - Start inserting at cursor
    InsertAppend,       // a               - Start inserting after cursor
    InsertAppendEnd,    // A               - Start inserting at end of line
    InsertLineStart,    // I               - Start inserting at line start
    InsertLineBelow,    // o               - Open line below and insert
    InsertLineAbove,    // O               - Open line above and insert
    InsertReplace,      // R               - Enter replace mode
    VisualChar,         // v               - Enter character-wise visual mode
    VisualLine,         // V               - Enter line-wise visual mode
    VisualBlock,        // <Ctrl-v>        - Enter block-wise visual mode
    CommandMode,        // :               - Enter command mode
    SearchForward,      // /               - Enter forward search mode
    SearchBackward,     // ?               - Enter backward search mode

    // Normal Mode Navigation
    MoveLeft,           // h, <Left>       - Move cursor left
    MoveDown,           // j, <Down>       - Move cursor down
    MoveUp,             // k, <Up>         - Move cursor up
    MoveRight,          // l, <Right>      - Move cursor right
    MoveWordForward,    // w               - Move to next word start
    MoveWordBackward,   // b               - Move to previous word start
    MoveEndWord,        // e               - Move to current word end
    MoveLineStart,      // 0, ^            - Move to line start
    MoveLineEnd,        // $               - Move to line end
    MoveFileStart,      // gg              - Move to start of file
    MoveFileEnd,        // G               - Move to end of file
    
    // Normal Mode Operations
    Undo,               // u               - Undo last change
    Redo,               // <Ctrl-r>        - Redo previously undone change
    DeleteChar,         // x               - Delete character at cursor
    DeleteLine,         // dd              - Delete current line
    YankLine,           // yy              - Copy current line
    PasteLine,          // p               - Paste after cursor
    PasteLineBefore,    // P               - Paste before cursor
    JoinLines,          // J               - Join current line with next
    RepeatLastChange,   // .               - Repeat last change
    Indent,             // >>              - Indent line
    Dedent,             // <<              - Dedent line
    
    // Insert Mode Operations
    InsertTab,          // <Tab>           - Insert tab or spaces
    InsertBackTab,      // <Shift-Tab>     - Remove one level of indentation
    InsertNewLine,      // <Enter>         - Insert newline with indent
    InsertCharacter,    // any character   - Insert the character
    DeleteCharBefore,   // <Backspace>     - Delete character before cursor
    DeleteCharAfter,    // <Delete>        - Delete character after cursor
    DeleteWordBefore,   // <Ctrl-w>        - Delete word before cursor
    DeleteLineBefore,   // <Ctrl-u>        - Delete to start of line
    CompleteWord,       // <Ctrl-n>        - Word completion
    CompleteLineUp,     // <Ctrl-p>        - Line completion (prev)
    CompleteLineDown,   // <Ctrl-n>        - Line completion (next)

    // Visual Mode Operations
    VisualToggle,       // v/V/<Ctrl-v>    - Toggle between visual modes
    VisualIndent,       // >               - Indent selection
    VisualDedent,       // <               - Dedent selection
    VisualYank,         // y               - Copy selection
    VisualDelete,       // d               - Delete selection
    VisualChange,       // c               - Change selection
    VisualJoin,         // J               - Join selected lines
    VisualUpper,        // U               - Convert to uppercase
    VisualLower,        // u               - Convert to lowercase
    
    // Command Mode Operations
    CommandBackspace,   // <Backspace>     - Delete previous character
    CommandComplete,    // <Tab>           - Command completion
    CommandHistoryUp,   // <Up>            - Previous command in history
    CommandHistoryDown, // <Down>          - Next command in history
    
    // Scrolling and Window Operations
    ScrollUp,           // <Ctrl-u>        - Scroll up half screen
    ScrollDown,         // <Ctrl-d>        - Scroll down half screen
    ScrollPageUp,       // <PageUp>        - Scroll up one screen
    ScrollPageDown,     // <PageDown>      - Scroll down one screen
    WindowSplitH,       // :sp, <Ctrl-w>s  - Split window horizontally
    WindowSplitV,       // :vsp, <Ctrl-w>v - Split window vertically
    WindowNext,         // <Ctrl-w>w       - Go to next window
    WindowClose,        // :q, <Ctrl-w>q   - Close current window
}

/// Represents the style of cursor to display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorStyle {
    Block,      // Full block cursor for normal mode
    Line,       // Vertical line for insert mode
    Underline,  // Underscore for replace mode
}

impl Mode {
    /// Creates the default editor mode (Normal)
    pub fn default() -> Self {
        Mode::Normal
    }

    /// Returns true if text input should be processed
    pub fn allows_text_input(&self) -> bool {
        matches!(self, Mode::Insert(_) | Mode::Command(_))
    }

    /// Returns true if cursor movement is allowed
    pub fn allows_cursor_movement(&self) -> bool {
        !matches!(self, Mode::Command(_))
    }

    /// Returns true if modern system clipboard operations are allowed
    pub fn allows_system_clipboard(&self) -> bool {
        true  // Modern clipboard operations allowed in all modes
    }

    /// Returns true if mouse operations are allowed
    pub fn allows_mouse(&self) -> bool {
        !matches!(self, Mode::Command(_))  // Mouse restricted in command mode
    }

    /// Returns true if deletion is allowed
    pub fn allows_deletion(&self) -> bool {
        match self {
            Mode::Normal => true,
            Mode::Insert(_) => true,
            Mode::Visual(_) => false,  // Visual mode has its own deletion handling
            Mode::Command(_) => true,  // Allow backspace in command mode
        }
    }

    /// Returns true if cutting is allowed
    pub fn allows_cut(&self) -> bool {
        match self {
            Mode::Normal => true,
            Mode::Visual(_) => true,
            _ => false,
        }
    }

    /// Returns true if undo/redo operations are allowed
    pub fn allows_undo(&self) -> bool {
        matches!(self, Mode::Normal)
    }

    /// Returns true if in visual mode
    pub fn is_visual(&self) -> bool {
        matches!(self, Mode::Visual(_))
    }

    /// Returns true if selection operations are allowed
    pub fn allows_selection(&self) -> bool {
        match self {
            Mode::Normal | Mode::Visual(_) => true,
            Mode::Insert(_) => true,  // Allow Shift+Arrow selection in insert mode
            Mode::Command(_) => false,
        }
    }

    /// Returns true if scrolling operations are allowed
    pub fn allows_scrolling(&self) -> bool {
        !matches!(self, Mode::Command(_))
    }

    /// Returns the visual variant if in visual mode
    pub fn get_visual_variant(&self) -> Option<VisualVariant> {
        match self {
            Mode::Visual(variant) => Some(*variant),
            _ => None
        }
    }

    /// Returns the prefix for command line (':' or '/')
    pub fn command_prefix(&self) -> &str {
        match self {
            Mode::Command(CommandType::Regular) => ":",
            Mode::Command(CommandType::Search) => "/",
            Mode::Command(CommandType::Backward) => "?",
            _ => "",
        }
    }

    /// Returns a user-friendly name for the current mode
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
            Mode::Visual(variant) => match variant {
                VisualVariant::Char => "VISUAL",
                VisualVariant::Line => "VISUAL LINE",
                VisualVariant::Block => "VISUAL BLOCK",
            },
            Mode::Command(cmd_type) => match cmd_type {
                CommandType::Regular => "COMMAND",
                CommandType::Search => "SEARCH",
                CommandType::Backward => "REVERSE SEARCH",
            },
        }
    }

    /// Returns the appropriate cursor style for the current mode
    pub fn cursor_style(&self) -> CursorStyle {
        match self {
            Mode::Normal => CursorStyle::Block,
            Mode::Insert(InsertVariant::Replace) => CursorStyle::Underline,
            Mode::Insert(_) => CursorStyle::Line,
            Mode::Visual(_) => CursorStyle::Block,
            Mode::Command(_) => CursorStyle::Line,
        }
    }

    /// Handles mode transitions based on triggers
    pub fn transition(&self, trigger: ModeTrigger) -> Mode {
        use ModeTrigger::*;
        
        match (self, trigger) {
            // Global transitions
            (_, Escape) => Mode::Normal,
            (Mode::Command(_), Enter) => Mode::Normal,
            
            // Common Movement Operations (maintain mode if movement is allowed)
            (current, trigger) if self.allows_cursor_movement() && is_movement_trigger(trigger) => *current,
            
            // System clipboard operations (maintain mode)
            (current, trigger) if self.allows_system_clipboard() && is_clipboard_trigger(trigger) => *current,
            
            // Mouse operations
            (current, MouseClick) if self.allows_mouse() => *current,
            (current, MouseDrag) if self.allows_mouse() => Mode::Visual(VisualVariant::Char),
            (Mode::Normal, MouseDoubleClick) => Mode::Visual(VisualVariant::Char),
            (Mode::Normal, MouseTripleClick) => Mode::Visual(VisualVariant::Line),

            // Normal mode transitions
            (Mode::Normal, InsertNormal) => Mode::Insert(InsertVariant::Insert),
            (Mode::Normal, InsertAppend) => Mode::Insert(InsertVariant::Append),
            (Mode::Normal, InsertAppendEnd) => Mode::Insert(InsertVariant::AppendEnd),
            (Mode::Normal, InsertLineStart) => Mode::Insert(InsertVariant::LineStart),
            (Mode::Normal, InsertLineBelow) => Mode::Insert(InsertVariant::LineBelow),
            (Mode::Normal, InsertLineAbove) => Mode::Insert(InsertVariant::LineAbove),
            (Mode::Normal, InsertReplace) => Mode::Insert(InsertVariant::Replace),
            (Mode::Normal, VisualChar) => Mode::Visual(VisualVariant::Char),
            (Mode::Normal, VisualLine) => Mode::Visual(VisualVariant::Line),
            (Mode::Normal, VisualBlock) => Mode::Visual(VisualVariant::Block),
            (Mode::Normal, CommandMode) => Mode::Command(CommandType::Regular),
            (Mode::Normal, SearchForward) => Mode::Command(CommandType::Search),
            (Mode::Normal, SearchBackward) => Mode::Command(CommandType::Backward),

            // Insert mode operations (maintain mode)
            (mode @ Mode::Insert(_), trigger) if is_insert_operation(trigger) => *mode,

            // Visual mode transitions
            (Mode::Visual(_), VisualChar) => Mode::Visual(VisualVariant::Char),
            (Mode::Visual(_), VisualLine) => Mode::Visual(VisualVariant::Line),
            (Mode::Visual(_), VisualBlock) => Mode::Visual(VisualVariant::Block),
            (mode @ Mode::Visual(_), VisualIndent) => *mode,
            (mode @ Mode::Visual(_), VisualDedent) => *mode,
            (Mode::Visual(_), VisualYank) => Mode::Normal,
            (Mode::Visual(_), VisualDelete) => Mode::Normal,
            (Mode::Visual(_), VisualChange) => Mode::Insert(InsertVariant::Insert),

            // Selection operations
            (current, trigger) if self.allows_selection() && is_selection_trigger(trigger) => {
                match current {
                    Mode::Normal => Mode::Visual(VisualVariant::Char),
                    mode => *mode,
                }
            },

            // Scrolling operations (maintain mode)
            (current, trigger) if self.allows_scrolling() && is_scroll_trigger(trigger) => *current,

            // Stay in current mode by default
            (current, _) => *current,
        }
    }
}

impl CursorStyle {
    /// Returns the string representation of the cursor style
    pub fn as_str(&self) -> &str {
        match self {
            CursorStyle::Block => "█",
            CursorStyle::Line => "│",
            CursorStyle::Underline => "_",
        }
    }
}

// Helper functions for categorizing triggers
fn is_movement_trigger(trigger: ModeTrigger) -> bool {
    use ModeTrigger::*;
    matches!(trigger,
        ArrowLeft | ArrowRight | ArrowUp | ArrowDown |
        Home | End | MoveLeft | MoveRight | MoveUp | MoveDown |
        MoveWordForward | MoveWordBackward | MoveEndWord |
        MoveLineStart | MoveLineEnd | MoveFileStart | MoveFileEnd
    )
}

fn is_clipboard_trigger(trigger: ModeTrigger) -> bool {
    use ModeTrigger::*;
    matches!(trigger, SystemCopy | SystemPaste | SystemCut)
}

fn is_insert_operation(trigger: ModeTrigger) -> bool {
    use ModeTrigger::*;
    matches!(trigger,
        InsertCharacter | DeleteCharBefore | DeleteCharAfter |
        DeleteWordBefore | DeleteLineBefore | InsertTab |
        InsertBackTab | InsertNewLine | CompleteWord |
        CompleteLineUp | CompleteLineDown
    )
}

fn is_selection_trigger(trigger: ModeTrigger) -> bool {
    use ModeTrigger::*;
    matches!(trigger,
        MouseDrag | MouseDoubleClick | MouseTripleClick |
        VisualToggle | VisualIndent | VisualDedent |
        VisualYank | VisualDelete | VisualChange
    )
}

fn is_scroll_trigger(trigger: ModeTrigger) -> bool {
    use ModeTrigger::*;
    matches!(trigger,
        ScrollUp | ScrollDown | ScrollPageUp | ScrollPageDown |
        MouseScroll
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_transitions() {
        let normal = Mode::Normal;
        
        // Test insert mode transitions
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
        
        // Test visual mode transitions
        assert_eq!(
            normal.transition(ModeTrigger::VisualChar),
            Mode::Visual(VisualVariant::Char)
        );
        assert_eq!(
            normal.transition(ModeTrigger::VisualLine),
            Mode::Visual(VisualVariant::Line)
        );
        assert_eq!(
            normal.transition(ModeTrigger::VisualBlock),
            Mode::Visual(VisualVariant::Block)
        );
        
        // Test command mode transitions
        assert_eq!(
            normal.transition(ModeTrigger::CommandMode),
            Mode::Command(CommandType::Regular)
        );
        assert_eq!(
            normal.transition(ModeTrigger::SearchForward),
            Mode::Command(CommandType::Search)
        );
        assert_eq!(
            normal.transition(ModeTrigger::SearchBackward),
            Mode::Command(CommandType::Backward)
        );
    }

    #[test]
    fn test_escape_from_all_modes() {
        let modes = vec![
            Mode::Insert(InsertVariant::Insert),
            Mode::Visual(VisualVariant::Char),
            Mode::Command(CommandType::Regular),
        ];

        for mode in modes {
            assert_eq!(mode.transition(ModeTrigger::Escape), Mode::Normal);
        }
    }

    #[test]
    fn test_visual_mode_operations() {
        let visual = Mode::Visual(VisualVariant::Char);
        
        // Test visual mode operations returning to normal
        assert_eq!(
            visual.transition(ModeTrigger::VisualYank),
            Mode::Normal
        );
        assert_eq!(
            visual.transition(ModeTrigger::VisualDelete),
            Mode::Normal
        );
        
        // Test visual mode change entering insert mode
        assert_eq!(
            visual.transition(ModeTrigger::VisualChange),
            Mode::Insert(InsertVariant::Insert)
        );
        
        // Test maintaining visual mode for certain operations
        let visual_line = Mode::Visual(VisualVariant::Line);
        assert_eq!(
            visual_line.transition(ModeTrigger::VisualIndent),
            Mode::Visual(VisualVariant::Line)
        );
    }

    #[test]
    fn test_command_mode_completion() {
        let command = Mode::Command(CommandType::Regular);
        let search = Mode::Command(CommandType::Search);
        
        assert_eq!(command.transition(ModeTrigger::Enter), Mode::Normal);
        assert_eq!(search.transition(ModeTrigger::Enter), Mode::Normal);
    }

    #[test]
    fn test_permission_checks() {
        let normal = Mode::Normal;
        let insert = Mode::Insert(InsertVariant::Insert);
        let visual = Mode::Visual(VisualVariant::Char);
        let command = Mode::Command(CommandType::Regular);

        // Test text input permissions
        assert!(!normal.allows_text_input());
        assert!(insert.allows_text_input());
        assert!(!visual.allows_text_input());
        assert!(command.allows_text_input());

        // Test cursor movement permissions
        assert!(normal.allows_cursor_movement());
        assert!(insert.allows_cursor_movement());
        assert!(visual.allows_cursor_movement());
        assert!(!command.allows_cursor_movement());

        // Test deletion permissions
        assert!(normal.allows_deletion());
        assert!(insert.allows_deletion());
        assert!(!visual.allows_deletion());
        assert!(command.allows_deletion());

        // Test undo permissions
        assert!(normal.allows_undo());
        assert!(!insert.allows_undo());
        assert!(!visual.allows_undo());
        assert!(!command.allows_undo());
    }

    #[test]
    fn test_display_names() {
        assert_eq!(Mode::Normal.display_name(), "NORMAL");
        assert_eq!(
            Mode::Insert(InsertVariant::Insert).display_name(),
            "INSERT"
        );
        assert_eq!(
            Mode::Visual(VisualVariant::Line).display_name(),
            "VISUAL LINE"
        );
        assert_eq!(
            Mode::Command(CommandType::Regular).display_name(),
            "COMMAND"
        );
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
            CursorStyle::Underline
        );
        assert_eq!(
            Mode::Command(CommandType::Regular).cursor_style(),
            CursorStyle::Line
        );
    }

    #[test]
    fn test_command_prefixes() {
        assert_eq!(
            Mode::Command(CommandType::Regular).command_prefix(),
            ":"
        );
        assert_eq!(
            Mode::Command(CommandType::Search).command_prefix(),
            "/"
        );
        assert_eq!(
            Mode::Command(CommandType::Backward).command_prefix(),
            "?"
        );
        assert_eq!(Mode::Normal.command_prefix(), "");
    }

    #[test]
    fn test_visual_variant_access() {
        assert_eq!(
            Mode::Visual(VisualVariant::Char).get_visual_variant(),
            Some(VisualVariant::Char)
        );
        assert_eq!(Mode::Normal.get_visual_variant(), None);
    }
}