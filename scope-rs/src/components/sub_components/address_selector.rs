use egui::Ui;
use ethers::types::{Address, U256};

use crate::{app::RenderConfigs, backend, shared_state::STATE};
use eyre::Result;

use super::CopyButton;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct AddressSelector {
    pub new_address_input: String,
    pub new_balance_input: String,
}

impl AddressSelector {
    pub fn show(&mut self, ui: &mut Ui, render_configs: &mut RenderConfigs) {
        let from_addresses = &mut (*STATE.from_addresses.write().unwrap()); // Use write lock for modification
        let mut tx_configs = STATE.tx_configs.write().unwrap(); // Read the tx_configs from the shared state
        let from_address = &mut tx_configs.from_address; // Modify the reference to directly point to the shared state's field

        // Flag to control the visibility of the text input for a new address
        let show_new_address_input = &mut render_configs.show_new_address_input;

        // Flag to control the visibility of the balance input
        let show_balance_input = &mut render_configs.show_balance_input;
        let new_balance_input = &mut self.new_balance_input;

        // Existing dropdown
        ui.horizontal(|ui| {
            ui.label("From:  ");
            egui::ComboBox::from_id_source("from_address")
                .selected_text(from_address.to_string())
                .width(ui.available_width() * 0.5)
                .wrap(false)
                .show_ui(ui, |ui| {
                    for address in from_addresses.iter() {
                        ui.selectable_value(from_address, *address, format!("{}", address));
                    }
                    ui.selectable_value(
                        show_new_address_input,
                        !(*show_new_address_input),
                        "Add New Address",
                    );
                });

            let full_address = format!("{:#x}", from_address);
            // Button to toggle the visibility of the text input for a new address
            CopyButton::new("📋".to_string(), full_address.clone()).show(ui);
            if ui.button("Get Bal.").clicked() {
                backend::send_shell_command(format!("cast balance {}", full_address));
            }

            if ui.button("Set").clicked() {
                *show_balance_input = !(*show_balance_input);
            }
        });
        if *show_balance_input {
            ui.horizontal(|ui| {
                ui.label("New Balance:");

                ui.add(
                    egui::TextEdit::singleline(new_balance_input)
                        .desired_width(ui.available_width() * 0.5)
                        .hint_text("(ether)"),
                );
                if ui.button("Confirm").clicked() {
                    // Convert the input to U256 and set the balance
                    if let Ok(eth_value) = new_balance_input.parse::<f64>() {
                        let bal: U256 = ethers::utils::parse_ether(eth_value)
                            .expect("Error parsing new amount");
                        backend::set_balance(*from_address, bal); // set_balance is your backend function to set balance
                        *show_balance_input = false;
                        new_balance_input.clear();
                    } else {
                        backend::send_error_popup("Failed to parse balance".to_string());
                    }
                }
            });
        }

        if *show_new_address_input {
            let new_address_input = &mut self.new_address_input;
            // Text input for a new address
            ui.horizontal(|ui| {
                ui.label("New address: ");
                ui.add(
                    egui::TextEdit::singleline(new_address_input)
                        .desired_width(ui.available_width() * 0.5),
                );
                if ui.button("Add").clicked() {
                    // Perform validation if necessary
                    let ethers_address: Result<Address, _> = new_address_input.parse();

                    match ethers_address {
                        Ok(address) => {
                            from_addresses.push(address);
                            new_address_input.clear();
                            *show_new_address_input = false;
                            *from_address = address;
                        }
                        Err(_) => backend::send_error_popup("Failed to parse address".to_string()),
                    }
                }
                if ui.button("❌").clicked() {
                    *show_new_address_input = false;
                    new_address_input.clear();
                }
            });
        }
    }
}
