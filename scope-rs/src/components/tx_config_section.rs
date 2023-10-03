use crate::{app::RenderConfigs, shared_state::STATE};

use super::AddressSelector;

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct TxConfigSection {
    address_selector: AddressSelector,
}

impl TxConfigSection {
    pub fn show(&mut self, ui: &mut egui::Ui, render_configs: &mut RenderConfigs) {
        ui.label("Transaction Configs");
        self.address_selector.show(ui, render_configs);

        self.render_value_input(ui);
    }

    fn render_value_input(&mut self, ui: &mut egui::Ui) {
        // Get the tx_configs write lock
        let mut tx_configs = STATE.tx_configs.write().unwrap();

        ui.horizontal(|ui| {
            ui.label("Value: ");
            ui.add(
                egui::TextEdit::singleline(&mut tx_configs.value)
                    .hint_text("0 (ether)")
                    .desired_width(ui.available_width() * 0.5),
            );
        });
    }
}
