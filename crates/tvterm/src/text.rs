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
use crate::theme::Palette;

/// Cell dimensions in pixels.
#[derive(Debug, Clone, Copy)]
pub struct CellMetrics {
    pub width: f32,
    pub height: f32,
}

/// Compute monospace cell dimensions from the egui context.
pub fn compute_cell_metrics(ctx: &egui::Context, font_size: f32) -> CellMetrics {
    let font_id = FontId::monospace(font_size);
    let galley = ctx.fonts_mut(|f| {
        f.layout_no_wrap("M".to_string(), font_id.clone(), Color32::WHITE)
    });
    CellMetrics {
        width: galley.rect.width(),
        height: galley.rect.height(),
    }
}

/// Render the terminal grid content using egui.
pub fn render_terminal(
    ctx: &egui::Context,
    term: &Term<TermEventProxy>,
    font_size: f32,
    cell_metrics: CellMetrics,
    opacity: f32,
    palette: Palette,
) {
    let rows = term.screen_lines();
    let content = term.renderable_content();
    let colors = content.colors;
    let cursor = &content.cursor;

    let cell_w = cell_metrics.width;
    let cell_h = cell_metrics.height;
    let font_id = FontId::monospace(font_size);

    let default_bg = TermColor::Named(NamedColor::Background);
    let fg_default = palette.foreground;

    #[expect(deprecated)]
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let painter = ui.painter();

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
                    let rgb = resolve_term_rgb(bg_color, colors, palette);
                    let a = (opacity * 255.0) as u8;
                    let bg32 = Color32::from_rgba_unmultiplied(rgb.r, rgb.g, rgb.b, a);
                    let rect = Rect::from_min_size(
                        Pos2::new(col_idx as f32 * cell_w, line_idx as f32 * cell_h),
                        Vec2::new(cell_w, cell_h),
                    );
                    painter.rect_filled(rect, 0.0, bg32);
                }

                let c = if cell.c == '\0' { ' ' } else { cell.c };
                let fg_rgb = resolve_term_rgb(fg_color, colors, palette);
                let fg32 = Color32::from_rgb(fg_rgb.r, fg_rgb.g, fg_rgb.b);

                lines[line_idx].push(CellInfo { c, fg: fg32 });
            }

            // Draw cursor.
            let cursor_line = cursor.point.line.0 as usize;
            let cursor_col = cursor.point.column.0;
            if cursor_line < rows && cursor.shape != CursorShape::Hidden {
                let cx = cursor_col as f32 * cell_w;
                let cy = cursor_line as f32 * cell_h;
                let cr = palette.cursor;
                let cursor_color = Color32::from_rgba_unmultiplied(cr.r, cr.g, cr.b, 200);

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

            // Render text row by row.
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
                let fg32 = Color32::from_rgb(fg_default.r, fg_default.g, fg_default.b);
                painter.galley(
                    Pos2::new(0.0, line_idx as f32 * cell_h),
                    galley,
                    fg32,
                );
            }
        });
}

struct CellInfo {
    c: char,
    fg: Color32,
}

/// Resolve a terminal color to an RGB triplet using the theme palette.
fn resolve_term_rgb(
    color: TermColor,
    colors: &alacritty_terminal::term::color::Colors,
    palette: Palette,
) -> TermRgb {
    match color {
        TermColor::Spec(rgb) => rgb,
        TermColor::Named(name) => {
            // Use the palette for named colors, but allow alacritty_terminal's
            // color overrides (from OSC sequences) to take priority.
            let idx = name as usize;
            colors[idx].unwrap_or_else(|| palette.resolve_named(name))
        }
        TermColor::Indexed(idx) => {
            colors[idx as usize].unwrap_or_else(|| {
                if idx < 16 {
                    // Map indexed 0-15 to named colors through the palette.
                    let name = match idx {
                        0 => NamedColor::Black,
                        1 => NamedColor::Red,
                        2 => NamedColor::Green,
                        3 => NamedColor::Yellow,
                        4 => NamedColor::Blue,
                        5 => NamedColor::Magenta,
                        6 => NamedColor::Cyan,
                        7 => NamedColor::White,
                        8 => NamedColor::BrightBlack,
                        9 => NamedColor::BrightRed,
                        10 => NamedColor::BrightGreen,
                        11 => NamedColor::BrightYellow,
                        12 => NamedColor::BrightBlue,
                        13 => NamedColor::BrightMagenta,
                        14 => NamedColor::BrightCyan,
                        15 => NamedColor::BrightWhite,
                        _ => unreachable!(),
                    };
                    palette.resolve_named(name)
                } else {
                    indexed_color(idx)
                }
            })
        }
    }
}

/// Standard 256-color palette lookup for indexed colors 16-255.
fn indexed_color(idx: u8) -> TermRgb {
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
