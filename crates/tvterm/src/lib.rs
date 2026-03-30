use rmx::prelude::*;

pub mod app;
pub mod config;
pub mod overlay;
pub mod pty;
pub mod renderer;
pub mod terminal;
pub mod text;

use config::Config;

/// Custom events sent to the winit event loop.
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// New PTY output is available.
    PtyOutput,
    /// The shell process has exited.
    PtyExited,
}

/// Run the terminal emulator.
pub fn run(config: Config) -> AnyResult<()> {
    let event_loop = winit::event_loop::EventLoop::<UserEvent>::with_user_event()
        .build()?;
    let proxy = event_loop.create_proxy();
    let mut app = app::App::new(config, proxy);
    event_loop.run_app(&mut app)?;
    Ok(())
}
