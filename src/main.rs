use editor::Editor;
use config::Config;
use ui::UI;
use input::InputHandler;
use std::io;

fn main() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Initialize components
    let mut editor = Editor::new(config);
    let mut ui = UI::new();
    let mut input_handler = InputHandler::new();

    // Main even loop
    editor.run(ui, input_handler)?;

    Ok(());
}