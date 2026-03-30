use rmx::prelude::*;

use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::cell::Flags as CellFlags;
use alacritty_terminal::term::Term;
use alacritty_terminal::vte::ansi::{Color as TermColor, CursorShape, NamedColor, Rgb as TermRgb};
use egui::{
    Color32, FontId, Pos2, Rect, Vec2,
    text::{LayoutJob, TextFormat},
};

use crate::terminal::TermEventProxy;

/// Cell dimensions in pixels.
#[derive(Debug, Clone, Copy)]
pub struct CellMetrics {
    pub width: f32,
    pub height: f32,
}

/// Compute monospace cell dimensions from the egui context.
pub fn compute_cell_metrics(ctx: &egui::Context, font_size: f32) -> CellMetrics {
    let font_id = FontId::monospace(font_size);
    // Layout a reference character to measure cell dimensions.
    let galley = ctx.fonts_mut(|f| {
        f.layout_no_wrap("M".to_string(), font_id.clone(), Color32::WHITE)
    });
    let cell_width = galley.rect.width();
    let cell_height = galley.rect.height();
    CellMetrics {
        width: cell_width,
        height: cell_height,
    }
}

/// Render the terminal grid content using egui.
pub fn render_terminal(
    ctx: &egui::Context,
    term: &Term<TermEventProxy>,
    font_size: f32,
    cell_metrics: CellMetrics,
    opacity: f32,
) {
    let rows = term.screen_lines();
    let content = term.renderable_content();
    let colors = content.colors;
    let cursor = &content.cursor;

    let cell_w = cell_metrics.width;
    let cell_h = cell_metrics.height;
    let font_id = FontId::monospace(font_size);

    let default_bg = TermColor::Named(NamedColor::Background);

    // Use a CentralPanel with a transparent frame to fill the whole window.
    #[expect(deprecated)]
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let painter = ui.painter();

            // Collect cells per line.
            let mut lines: Vec<Vec<CellInfo>> = (0..rows).map(|_| Vec::new()).collect();

            for indexed in content.display_iter {
                let line_idx = indexed.point.line.0 as usize;
                if line_idx >= rows {
                    continue;
                }
                let col_idx = indexed.point.column.0;

                let cell = &indexed.cell;
                let (fg_color, bg_color) = if cell.flags.contains(CellFlags::INVERSE) {
                    (cell.bg, cell.fg)
                } else {
                    (cell.fg, cell.bg)
                };

                // Draw cell background if non-default.
                if bg_color != default_bg {
                    let rgb = resolve_term_rgb(bg_color, colors);
                    let a = (opacity * 255.0) as u8;
                    let bg32 = Color32::from_rgba_unmultiplied(rgb.r, rgb.g, rgb.b, a);
                    let rect = Rect::from_min_size(
                        Pos2::new(col_idx as f32 * cell_w, line_idx as f32 * cell_h),
                        Vec2::new(cell_w, cell_h),
                    );
                    painter.rect_filled(rect, 0.0, bg32);
                }

                let c = if cell.c == '\0' { ' ' } else { cell.c };
                let fg_rgb = resolve_term_rgb(fg_color, colors);
                let fg32 = Color32::from_rgb(fg_rgb.r, fg_rgb.g, fg_rgb.b);

                lines[line_idx].push(CellInfo { c, fg: fg32 });
            }

            // Draw cursor.
            let cursor_line = cursor.point.line.0 as usize;
            let cursor_col = cursor.point.column.0;
            if cursor_line < rows && cursor.shape != CursorShape::Hidden {
                let cx = cursor_col as f32 * cell_w;
                let cy = cursor_line as f32 * cell_h;
                let cursor_color = Color32::from_rgba_unmultiplied(200, 200, 200, 200);

                match cursor.shape {
                    CursorShape::Block => {
                        painter.rect_filled(
                            Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(cell_w, cell_h)),
                            0.0,
                            cursor_color,
                        );
                    }
                    CursorShape::Beam => {
                        painter.rect_filled(
                            Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(2.0, cell_h)),
                            0.0,
                            cursor_color,
                        );
                    }
                    CursorShape::Underline => {
                        painter.rect_filled(
                            Rect::from_min_size(
                                Pos2::new(cx, cy + cell_h - 2.0),
                                Vec2::new(cell_w, 2.0),
                            ),
                            0.0,
                            cursor_color,
                        );
                    }
                    CursorShape::HollowBlock => {
                        painter.rect_stroke(
                            Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(cell_w, cell_h)),
                            0.0,
                            egui::Stroke::new(1.5, cursor_color),
                            egui::StrokeKind::Outside,
                        );
                    }
                    _ => {}
                }
            }

            // Render text row by row using LayoutJob for per-character colors.
            for (line_idx, cells) in lines.iter().enumerate() {
                if cells.is_empty() {
                    continue;
                }

                let mut job = LayoutJob::default();
                job.wrap = egui::text::TextWrapping {
                    max_rows: 1,
                    break_anywhere: false,
                    ..Default::default()
                };

                for cell in cells {
                    let start = job.text.len();
                    job.text.push(cell.c);
                    job.sections.push(egui::text::LayoutSection {
                        leading_space: 0.0,
                        byte_range: start..job.text.len(),
                        format: TextFormat {
                            font_id: font_id.clone(),
                            color: cell.fg,
                            ..Default::default()
                        },
                    });
                }

                let galley = painter.layout_job(job);
                painter.galley(
                    Pos2::new(0.0, line_idx as f32 * cell_h),
                    galley,
                    Color32::WHITE,
                );
            }
        });
}

struct CellInfo {
    c: char,
    fg: Color32,
}

/// Resolve a terminal color to an RGB triplet.
fn resolve_term_rgb(
    color: TermColor,
    colors: &alacritty_terminal::term::color::Colors,
) -> TermRgb {
    match color {
        TermColor::Spec(rgb) => rgb,
        TermColor::Named(name) => {
            let idx = name as usize;
            colors[idx].unwrap_or_else(|| default_named_color(name))
        }
        TermColor::Indexed(idx) => colors[idx as usize].unwrap_or_else(|| indexed_color(idx)),
    }
}

/// Default color for named terminal colors.
fn default_named_color(name: NamedColor) -> TermRgb {
    match name {
        NamedColor::Black => TermRgb { r: 0, g: 0, b: 0 },
        NamedColor::Red => TermRgb { r: 205, g: 49, b: 49 },
        NamedColor::Green => TermRgb { r: 13, g: 188, b: 121 },
        NamedColor::Yellow => TermRgb { r: 229, g: 229, b: 16 },
        NamedColor::Blue => TermRgb { r: 36, g: 114, b: 200 },
        NamedColor::Magenta => TermRgb { r: 188, g: 63, b: 188 },
        NamedColor::Cyan => TermRgb { r: 17, g: 168, b: 205 },
        NamedColor::White => TermRgb { r: 229, g: 229, b: 229 },
        NamedColor::BrightBlack => TermRgb { r: 102, g: 102, b: 102 },
        NamedColor::BrightRed => TermRgb { r: 241, g: 76, b: 76 },
        NamedColor::BrightGreen => TermRgb { r: 35, g: 209, b: 139 },
        NamedColor::BrightYellow => TermRgb { r: 245, g: 245, b: 67 },
        NamedColor::BrightBlue => TermRgb { r: 59, g: 142, b: 234 },
        NamedColor::BrightMagenta => TermRgb { r: 214, g: 112, b: 214 },
        NamedColor::BrightCyan => TermRgb { r: 41, g: 184, b: 219 },
        NamedColor::BrightWhite => TermRgb { r: 255, g: 255, b: 255 },
        NamedColor::Foreground => TermRgb { r: 229, g: 229, b: 229 },
        NamedColor::Background => TermRgb { r: 0, g: 0, b: 0 },
        _ => TermRgb { r: 229, g: 229, b: 229 },
    }
}

/// Standard 256-color palette lookup for indexed colors 16-255.
fn indexed_color(idx: u8) -> TermRgb {
    if idx < 16 {
        return TermRgb { r: 229, g: 229, b: 229 };
    }
    if idx < 232 {
        let idx = idx - 16;
        let r = (idx / 36) % 6;
        let g = (idx / 6) % 6;
        let b = idx % 6;
        let to_val = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
        return TermRgb {
            r: to_val(r),
            g: to_val(g),
            b: to_val(b),
        };
    }
    let v = 8 + 10 * (idx - 232);
    TermRgb { r: v, g: v, b: v }
}
