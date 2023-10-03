use crate::{app::RenderConfigs, shared_state::STATE, utils};

use super::{CopyButton, UtilityMenu};

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct DeployedSection {}

impl DeployedSection {
    pub fn show(&mut self, ui: &mut egui::Ui, render_configs: &mut RenderConfigs) {
        self.render_deployed_contracts(ui);
    }

    fn render_deployed_contracts(&mut self, ui: &mut egui::Ui) {
        let deployed_contracts = &mut (*STATE.deployed_contracts.write().unwrap());

        // sort deployed_contracts by deployed block number; need to do this every time unfortunately
        // as deployed_contracts.remove() will break ordering
        deployed_contracts
            .sort_by(|_, a, _, b| a.deployed_block.number.cmp(&b.deployed_block.number));
        // create collapsable headers for each address
        // can we avoid cloning contracts_map here
        for address in deployed_contracts //.sort_by(|k, v| v.)
            .keys()
            .cloned()
            .collect::<Vec<String>>()
        {
            let name = deployed_contracts.get(&address).unwrap().name.clone();
            ui.horizontal(|ui| {
                let mut should_remove = false;

                if let Some(deployed) = deployed_contracts.get_mut(&address) {
                    let avail_width = ui.available_width();
                    ui.set_max_width(utils::get_max_collapsable_width());
                    let collapsing_resp =
                        ui.collapsing(format!("{:?} - {:?}", name, address.clone()), |ui| {
                            deployed.show(ui);
                        });

                    // let add_space = collapsing_resp.fully_open();
                    let add_space = collapsing_resp.openness;
                    // ui.set_max_width(prev_width);
                    // ui.add_space(prev_width * 0.75);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        // don't love, bounces for a sec
                        // if add_space {
                        //     ui.add_space(avail_width * 0.025);
                        // }
                        ui.add_space(add_space * 7.5);
                        // ui.add_space(ui.available_width() * 0.5);
                        if ui.button("❌").clicked() {
                            should_remove = true;
                        }

                        UtilityMenu::show_for_deployed(ui, deployed);

                        CopyButton::new("📋".to_string(), deployed.address_string.clone()).show(ui);
                    });
                }
                // borrow checker fun
                if should_remove {
                    deployed_contracts.remove(&address);
                }
            });

            ui.separator();
        }
    }
}
