use rmx::prelude::*;
use rmx::std::sync::Arc;
use alacritty_terminal::grid::Scroll;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use crate::config::Config;
use crate::overlay;
use crate::pty::Pty;
use crate::renderer::Renderer;
use crate::terminal::Terminal;
use crate::text::{self, CellMetrics};
use crate::UserEvent;

const INITIAL_COLS: u16 = 80;
const INITIAL_ROWS: u16 = 24;

/// Application state, created before the event loop starts.
pub struct App {
    config: Config,
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    state: Option<RunningState>,
}

/// State that exists only after the window is created (after `resumed`).
struct RunningState {
    window: Arc<Window>,
    renderer: Renderer,
    terminal: Terminal,
    pty: Pty,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    opacity: f32,
    overlay_visible: bool,
    cell_metrics: Option<CellMetrics>,
    font_size: f32,
    modifiers: ModifiersState,
}

impl App {
    pub fn new(config: Config, proxy: winit::event_loop::EventLoopProxy<UserEvent>) -> Self {
        Self {
            config,
            proxy,
            state: None,
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_transparent(true)
            .with_title("tvterm");

        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                error!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let renderer = match Renderer::new(window.clone()) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to create renderer: {e}");
                event_loop.exit();
                return;
            }
        };

        let egui_ctx = egui::Context::default();

        // Configure egui visuals for a transparent terminal.
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::TRANSPARENT;
        visuals.window_fill = egui::Color32::from_rgba_unmultiplied(30, 30, 30, 220);
        egui_ctx.set_visuals(visuals);

        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            event_loop,
            Some(window.scale_factor() as f32),
            None,
            Some(renderer.max_texture_side()),
        );

        let terminal = Terminal::new(
            INITIAL_COLS as usize,
            INITIAL_ROWS as usize,
            self.proxy.clone(),
        );

        let pty = match Pty::spawn(INITIAL_COLS, INITIAL_ROWS, self.proxy.clone()) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to spawn PTY: {e}");
                event_loop.exit();
                return;
            }
        };

        self.state = Some(RunningState {
            window,
            renderer,
            terminal,
            pty,
            egui_ctx,
            egui_state,
            opacity: self.config.initial_opacity,
            overlay_visible: false,
            cell_metrics: None,
            font_size: self.config.font_size,
            modifiers: ModifiersState::empty(),
        });
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            UserEvent::PtyOutput => {
                let chunks = state.pty.drain_output();
                for chunk in &chunks {
                    state.terminal.process_bytes(chunk);
                }

                let writes = state.terminal.drain_pty_writes();
                for data in &writes {
                    if let Err(e) = state.pty.write(data) {
                        error!("PTY write error: {e}");
                    }
                }

                if !chunks.is_empty() {
                    state.window.request_redraw();
                }
            }
            UserEvent::PtyExited => {
                info!("Shell exited");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        // Let egui handle the event first.
        let egui_response = state.egui_state.on_window_event(&state.window, &event);
        if egui_response.repaint {
            state.window.request_redraw();
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::ModifiersChanged(mods) => {
                state.modifiers = mods.state();
            }
            WindowEvent::Resized(size) => {
                state.renderer.resize(size.width, size.height);
                if let Some(cell) = state.cell_metrics {
                    let cols = (size.width as f32 / cell.width).floor() as usize;
                    let rows = (size.height as f32 / cell.height).floor() as usize;
                    if cols > 0 && rows > 0 {
                        state.terminal.resize(cols, rows);
                        if let Err(e) = state.pty.resize(cols as u16, rows as u16) {
                            error!("PTY resize error: {e}");
                        }
                    }
                }
                state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                state.render_frame();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Don't scroll terminal if egui wants the event.
                if !egui_response.consumed {
                    let cell_h = state.cell_metrics.map_or(20.0, |m| m.height);
                    let lines = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y as i32,
                        MouseScrollDelta::PixelDelta(pos) => {
                            (pos.y as f32 / cell_h).round() as i32
                        }
                    };
                    if lines != 0 {
                        state.terminal.term.scroll_display(Scroll::Delta(lines));
                        state.window.request_redraw();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }

                // Ctrl+Shift+O toggles the overlay.
                if state.modifiers.contains(ModifiersState::CONTROL)
                    && state.modifiers.contains(ModifiersState::SHIFT)
                {
                    if let Key::Character(c) = &event.logical_key {
                        if c.eq_ignore_ascii_case("o") {
                            state.overlay_visible = !state.overlay_visible;
                            state.window.request_redraw();
                            return;
                        }
                    }
                }

                // Don't send keys to PTY if egui consumed the event.
                if egui_response.consumed {
                    return;
                }

                if let Some(bytes) = translate_key(&event.logical_key, state.modifiers) {
                    if let Err(e) = state.pty.write(&bytes) {
                        error!("PTY write error: {e}");
                    }
                }
            }
            _ => {}
        }
    }
}

impl RunningState {
    fn render_frame(&mut self) {
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let opacity = self.opacity;
        let font_size = self.font_size;

        #[expect(deprecated)]
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Compute cell metrics on first frame (fonts not available until run()).
            if self.cell_metrics.is_none() {
                let metrics = text::compute_cell_metrics(ctx, font_size);
                info!("Cell metrics: {:.1}x{:.1}", metrics.width, metrics.height);
                self.cell_metrics = Some(metrics);
            }
            let cell_metrics = self.cell_metrics.unwrap();

            // Render terminal content.
            text::render_terminal(
                ctx,
                &self.terminal.term,
                font_size,
                cell_metrics,
                opacity,
            );

            // Render overlay.
            overlay::render_overlay(ctx, &mut self.opacity, self.overlay_visible);
        });

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        if let Err(e) = self.renderer.render(
            &full_output.textures_delta,
            &clipped_primitives,
            full_output.pixels_per_point,
            self.opacity,
        ) {
            error!("Render error: {e}");
        }
    }
}

/// Translate a key event into bytes to send to the PTY.
fn translate_key(key: &Key, modifiers: ModifiersState) -> Option<Vec<u8>> {
    let ctrl = modifiers.contains(ModifiersState::CONTROL);
    let alt = modifiers.contains(ModifiersState::ALT);
    let shift = modifiers.contains(ModifiersState::SHIFT);

    match key {
        Key::Character(c) => {
            if ctrl && !shift {
                let ch = c.chars().next()?;
                if ch.is_ascii_alphabetic() {
                    let ctrl_code = (ch.to_ascii_lowercase() as u8) - b'a' + 1;
                    return Some(maybe_alt(alt, &[ctrl_code]));
                }
                match ch {
                    '[' | '3' => return Some(maybe_alt(alt, &[0x1b])),
                    '\\' | '4' => return Some(maybe_alt(alt, &[0x1c])),
                    ']' | '5' => return Some(maybe_alt(alt, &[0x1d])),
                    '6' => return Some(maybe_alt(alt, &[0x1e])),
                    '/' | '7' => return Some(maybe_alt(alt, &[0x1f])),
                    '2' | ' ' | '@' => return Some(maybe_alt(alt, &[0x00])),
                    _ => {}
                }
            }
            let s = c.as_str();
            Some(maybe_alt(alt, s.as_bytes()))
        }
        Key::Named(named) => named_key_sequence(*named, ctrl, shift, alt),
        _ => None,
    }
}

fn maybe_alt(alt: bool, bytes: &[u8]) -> Vec<u8> {
    if alt {
        let mut v = vec![0x1b];
        v.extend_from_slice(bytes);
        v
    } else {
        bytes.to_vec()
    }
}

fn named_key_sequence(key: NamedKey, ctrl: bool, shift: bool, alt: bool) -> Option<Vec<u8>> {
    let modifier_param = if ctrl || shift || alt {
        Some(1 + (shift as u8) + (alt as u8) * 2 + (ctrl as u8) * 4)
    } else {
        None
    };

    let simple = |s: &str| Some(s.as_bytes().to_vec());

    match key {
        NamedKey::Enter => simple("\r"),
        NamedKey::Backspace => {
            if ctrl {
                simple("\x08")
            } else if alt {
                Some(b"\x1b\x7f".to_vec())
            } else {
                simple("\x7f")
            }
        }
        NamedKey::Tab => {
            if shift {
                simple("\x1b[Z")
            } else {
                simple("\t")
            }
        }
        NamedKey::Escape => simple("\x1b"),
        NamedKey::ArrowUp => Some(csi_mod(b"A", modifier_param)),
        NamedKey::ArrowDown => Some(csi_mod(b"B", modifier_param)),
        NamedKey::ArrowRight => Some(csi_mod(b"C", modifier_param)),
        NamedKey::ArrowLeft => Some(csi_mod(b"D", modifier_param)),
        NamedKey::Home => Some(csi_mod(b"H", modifier_param)),
        NamedKey::End => Some(csi_mod(b"F", modifier_param)),
        NamedKey::PageUp => Some(tilde_mod(5, modifier_param)),
        NamedKey::PageDown => Some(tilde_mod(6, modifier_param)),
        NamedKey::Insert => Some(tilde_mod(2, modifier_param)),
        NamedKey::Delete => Some(tilde_mod(3, modifier_param)),
        NamedKey::F1 => Some(ss3_or_csi(b"P", 11, modifier_param)),
        NamedKey::F2 => Some(ss3_or_csi(b"Q", 12, modifier_param)),
        NamedKey::F3 => Some(ss3_or_csi(b"R", 13, modifier_param)),
        NamedKey::F4 => Some(ss3_or_csi(b"S", 14, modifier_param)),
        NamedKey::F5 => Some(tilde_mod(15, modifier_param)),
        NamedKey::F6 => Some(tilde_mod(17, modifier_param)),
        NamedKey::F7 => Some(tilde_mod(18, modifier_param)),
        NamedKey::F8 => Some(tilde_mod(19, modifier_param)),
        NamedKey::F9 => Some(tilde_mod(20, modifier_param)),
        NamedKey::F10 => Some(tilde_mod(21, modifier_param)),
        NamedKey::F11 => Some(tilde_mod(23, modifier_param)),
        NamedKey::F12 => Some(tilde_mod(24, modifier_param)),
        _ => None,
    }
}

fn csi_mod(suffix: &[u8], modifier: Option<u8>) -> Vec<u8> {
    let mut seq = vec![0x1b, b'['];
    if let Some(m) = modifier {
        seq.extend_from_slice(b"1;");
        seq.extend_from_slice(m.to_string().as_bytes());
    }
    seq.extend_from_slice(suffix);
    seq
}

fn tilde_mod(num: u8, modifier: Option<u8>) -> Vec<u8> {
    let mut seq = vec![0x1b, b'['];
    seq.extend_from_slice(num.to_string().as_bytes());
    if let Some(m) = modifier {
        seq.push(b';');
        seq.extend_from_slice(m.to_string().as_bytes());
    }
    seq.push(b'~');
    seq
}

fn ss3_or_csi(ss3_suffix: &[u8], csi_num: u8, modifier: Option<u8>) -> Vec<u8> {
    if modifier.is_some() {
        tilde_mod(csi_num, modifier)
    } else {
        let mut seq = vec![0x1b, b'O'];
        seq.extend_from_slice(ss3_suffix);
        seq
    }
}
