// src/editor/mod.rs
mod buffer;
mod clipboard::Clipboard;
mod mode;

pub use buffer::Buffer;
pub use clipboard::Clipboard;
pub use mode::Mode;

pub struct Editor {
    buffer: Buffer,
    clipboard: Clipboard,
    mode: Mode,
}