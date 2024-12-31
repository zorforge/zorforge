// src/editor/mod.rs
mod buffer;
mod buffer_manager;
mod clipboard::Clipboard;
mod mode;

pub use buffer::Buffer;
pub use buffer_manager::BufferManager;
pub use clipboard::Clipboard;
pub use mode::Mode;

pub struct Editor {
    buffer: Buffer,
    clipboard: Clipboard,
    mode: Mode,
}