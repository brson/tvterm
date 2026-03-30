/// Render the opacity slider overlay using egui.
///
/// Returns the current opacity value.
pub fn render_overlay(ctx: &egui::Context, opacity: &mut f32, visible: bool) {
    if !visible {
        return;
    }

    egui::Window::new("Opacity")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Opacity");
                ui.add(egui::Slider::new(opacity, 0.0..=1.0).show_value(true));
            });
        });
}
