use rmx::prelude::*;

/// Terminal emulator configuration.
pub struct Config {
    pub initial_opacity: f32,
    pub font_size: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initial_opacity: 0.8,
            font_size: 16.0,
        }
    }
}
