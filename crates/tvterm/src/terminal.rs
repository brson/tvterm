use rmx::prelude::*;
use rmx::std::sync::mpsc;

use alacritty_terminal::event::EventListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::{self, Term};
use alacritty_terminal::vte;
use winit::event_loop::EventLoopProxy;

use crate::UserEvent;

/// Terminal dimensions implementing the `Dimensions` trait.
pub struct TermSize {
    pub cols: usize,
    pub rows: usize,
}

/// Default scrollback history size.
const SCROLLBACK_LINES: usize = 10_000;

impl Dimensions for TermSize {
    fn total_lines(&self) -> usize {
        self.rows + SCROLLBACK_LINES
    }

    fn screen_lines(&self) -> usize {
        self.rows
    }

    fn columns(&self) -> usize {
        self.cols
    }
}

/// Event listener that forwards terminal events to the winit event loop.
pub struct TermEventProxy {
    proxy: EventLoopProxy<UserEvent>,
    pty_write_tx: mpsc::Sender<Vec<u8>>,
}

impl TermEventProxy {
    pub fn new(
        proxy: EventLoopProxy<UserEvent>,
        pty_write_tx: mpsc::Sender<Vec<u8>>,
    ) -> Self {
        Self { proxy, pty_write_tx }
    }
}

impl EventListener for TermEventProxy {
    fn send_event(&self, event: alacritty_terminal::event::Event) {
        use alacritty_terminal::event::Event;
        match event {
            Event::Wakeup => {
                let _ = self.proxy.send_event(UserEvent::PtyOutput);
            }
            Event::PtyWrite(text) => {
                let _ = self.pty_write_tx.send(text.into_bytes());
            }
            Event::Title(title) => {
                info!("Terminal title: {title}");
            }
            Event::Exit => {
                let _ = self.proxy.send_event(UserEvent::PtyExited);
            }
            Event::Bell => {
                debug!("Bell");
            }
            _ => {}
        }
    }
}

/// Wraps the alacritty terminal state and VTE parser.
pub struct Terminal {
    pub term: Term<TermEventProxy>,
    parser: vte::ansi::Processor,
    pty_write_rx: mpsc::Receiver<Vec<u8>>,
}

impl Terminal {
    /// Create a new terminal with the given dimensions.
    pub fn new(cols: usize, rows: usize, proxy: EventLoopProxy<UserEvent>) -> Self {
        let (pty_write_tx, pty_write_rx) = mpsc::channel();
        let event_proxy = TermEventProxy::new(proxy, pty_write_tx);
        let size = TermSize { cols, rows };
        let config = term::Config::default();
        let term = Term::new(config, &size, event_proxy);
        let parser = vte::ansi::Processor::new();

        Self {
            term,
            parser,
            pty_write_rx,
        }
    }

    /// Feed raw bytes from the PTY into the terminal parser.
    pub fn process_bytes(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    /// Drain any pending PTY write requests from the terminal.
    pub fn drain_pty_writes(&self) -> Vec<Vec<u8>> {
        let mut writes = Vec::new();
        while let Ok(data) = self.pty_write_rx.try_recv() {
            writes.push(data);
        }
        writes
    }

    /// Resize the terminal grid.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let size = TermSize { cols, rows };
        self.term.resize(size);
    }
}
