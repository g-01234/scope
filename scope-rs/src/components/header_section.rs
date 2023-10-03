use crate::backend;

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct HeaderSection {}

impl HeaderSection {
    pub fn show(&self, ui: &mut egui::Ui) {
        // Top horizontal (title and root utility menu)
        ui.horizontal(|ui| {
            // ui.heading(RichText::new("scope 🔭").font(FontId::monospace(14.0)));
            ui.horizontal(|ui| {
                ui.heading("scope🔭");
                ui.label("(version: α)");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                // UtilityMenu::show_for_root(ui);
            });
        });

        // Refresh and compile buttons
        ui.horizontal(|ui| {
            if ui.button("🔄").clicked() {
                backend::query_for_open_files();
            }
            if ui.button("Compile").clicked() {
                backend::send_forge_build();
            }
        });
        ui.separator();
    }
}
