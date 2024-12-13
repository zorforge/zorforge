// src/ui/render.rs
pub trait Render {
    fn render(&self, mode: &Mode) -> Vec<String>;
}