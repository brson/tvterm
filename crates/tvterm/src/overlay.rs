use crate::theme::Theme;

/// Render the settings overlay using egui.
pub fn render_overlay(
    ctx: &egui::Context,
    opacity: &mut f32,
    bg_dim: &mut f32,
    theme: &mut Theme,
    visible: bool,
) {
    if !visible {
        return;
    }

    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Opacity");
                ui.add(egui::Slider::new(opacity, 0.0..=1.0).show_value(true));
            });

            ui.horizontal(|ui| {
                ui.label("Background");
                ui.add(egui::Slider::new(bg_dim, 0.0..=1.0).show_value(true));
            });

            ui.separator();

            ui.label("Theme");
            for &t in Theme::ALL {
                let p = t.palette();
                let selected = *theme == t;

                ui.horizontal(|ui| {
                    if ui.selectable_label(selected, t.name()).clicked() {
                        *theme = t;
                    }

                    let swatch_size = egui::vec2(12.0, 12.0);
                    let colors = [
                        p.red, p.green, p.yellow, p.blue, p.magenta, p.cyan,
                    ];
                    for c in colors {
                        let (rect, _) = ui.allocate_exact_size(swatch_size, egui::Sense::hover());
                        ui.painter().rect_filled(
                            rect,
                            2.0,
                            egui::Color32::from_rgb(c.r, c.g, c.b),
                        );
                    }
                });
            }
        });
}
