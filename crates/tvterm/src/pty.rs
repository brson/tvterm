use rmx::prelude::*;
use rmx::std::sync::mpsc;
use rmx::std::thread;

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use winit::event_loop::EventLoopProxy;

use crate::UserEvent;

/// Manages the PTY connection to the shell process.
pub struct Pty {
    writer: Box<dyn std::io::Write + Send>,
    master: Box<dyn MasterPty + Send>,
    pty_rx: mpsc::Receiver<Vec<u8>>,
}

impl Pty {
    /// Spawn a shell in a new PTY and start the reader thread.
    pub fn spawn(
        cols: u16,
        rows: u16,
        event_proxy: EventLoopProxy<UserEvent>,
    ) -> AnyResult<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new_default_prog();
        let _child = pair.slave.spawn_command(cmd)?;
        // Drop slave — we only need the master side.
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let (pty_tx, pty_rx) = mpsc::channel();
        Self::start_reader_thread(reader, pty_tx, event_proxy);

        Ok(Self {
            writer,
            master: pair.master,
            pty_rx,
        })
    }

    fn start_reader_thread(
        mut reader: Box<dyn std::io::Read + Send>,
        tx: mpsc::Sender<Vec<u8>>,
        proxy: EventLoopProxy<UserEvent>,
    ) {
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        let _ = proxy.send_event(UserEvent::PtyExited);
                        break;
                    }
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                        let _ = proxy.send_event(UserEvent::PtyOutput);
                    }
                    Err(e) => {
                        error!("PTY read error: {e}");
                        let _ = proxy.send_event(UserEvent::PtyExited);
                        break;
                    }
                }
            }
        });
    }

    /// Drain all pending output from the reader thread.
    pub fn drain_output(&self) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        while let Ok(chunk) = self.pty_rx.try_recv() {
            chunks.push(chunk);
        }
        chunks
    }

    /// Write bytes to the PTY (keyboard input).
    pub fn write(&mut self, data: &[u8]) -> AnyResult<()> {
        use std::io::Write;
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> AnyResult<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }
}
