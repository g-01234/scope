use crate::{app::RenderConfigs, backend, shared_state::STATE};

use super::{SelectedTarget, TargetMode};

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ContractSelectorSection {
    target: Option<SelectedTarget>,
}

impl ContractSelectorSection {
    pub fn show(&mut self, ui: &mut egui::Ui, render_configs: &mut RenderConfigs) {
        self.render_contract_selector(ui, render_configs);
        self.render_selected_target(ui, render_configs);
    }

    fn render_contract_selector(&mut self, ui: &mut egui::Ui, render_configs: &mut RenderConfigs) {
        let selected_name = &mut render_configs.selected_name;

        // Initialize a Vec to store tuples of the formatted name and filepath
        let mut contract_list: Vec<(String, String)> = Vec::new();

        for file_path in &(*STATE.open_files.read().unwrap()) {
            if !file_path.is_empty() {
                let split_path: Vec<&str> = file_path.split('/').collect();
                let contract_name = split_path
                    .last()
                    .unwrap()
                    .trim_end_matches(".json")
                    .to_string();
                let file_name = split_path[split_path.len() - 2].to_string();
                let formatted_name = format!("{}:{}", file_name, contract_name);
                contract_list.push((formatted_name, file_path.clone()));
            }
        }
        contract_list.sort();
        let prev_selected = selected_name.clone();

        egui::ComboBox::from_id_source("contract_selector")
            .selected_text(
                selected_name
                    .clone()
                    .unwrap_or_else(|| "Select Contract".to_string()),
            )
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (contract_name, _) in &contract_list {
                    ui.selectable_value(selected_name, Some(contract_name.clone()), contract_name);
                }
                ui.separator();
                ui.selectable_value(
                    selected_name,
                    Some("DeployRawBytecode".to_string()),
                    "Deploy raw bytecode",
                );
                ui.selectable_value(
                    selected_name,
                    Some("LoadWithoutABI".to_string()),
                    "Load address without ABI",
                );
                // backend::query_for_open_files(); // Delete this if slow
            });

        if prev_selected != *selected_name {
            if let Some(ref new_selection) = *selected_name {
                {
                    match new_selection.as_str() {
                        "DeployRawBytecode" => {
                            self.target = Some(SelectedTarget::new_deploy_raw(String::new()));
                        }
                        "LoadWithoutABI" => {
                            self.target = Some(SelectedTarget::new_load_raw());
                        }
                        _ => {
                            if let Some((formatted_name, file_path)) =
                                contract_list.iter().find(|(name, _)| name == new_selection)
                            {
                                if formatted_name.contains(".t.sol") {
                                    self.target = Some(SelectedTarget::new_foundry_test(
                                        file_path.to_string(),
                                    ));
                                } else {
                                    self.target =
                                        Some(SelectedTarget::new_compiled(file_path.to_string()));
                                }
                                backend::query_for_compiled_solidity(file_path.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_selected_target(&mut self, ui: &mut egui::Ui, render_configs: &mut RenderConfigs) {
        if let Some(target) = &mut self.target {
            // If we've received compiled solidity from VSCode, consume it from shared state
            // as the contract for target.contract and set to None in shared state.

            if let Some(received_compiled) = STATE.target_compiled.write().unwrap().take() {
                match &mut target.mode {
                    TargetMode::Compiled { contract, .. } => {
                        *contract = Some(received_compiled.clone());
                    }
                    TargetMode::FoundryTest { contract, .. } => {
                        *contract = Some(received_compiled.clone());
                    }
                    // Other cases
                    _ => {}
                }
            }

            target.show(ui, render_configs);

            // Handle compile occurrence
            let compile_occurred = *STATE.completed_compile.read().unwrap();
            if compile_occurred == Some(true) {
                if let TargetMode::Compiled { file_path, .. } = &target.mode {
                    backend::query_for_compiled_solidity(file_path.to_string());
                }
                backend::query_for_open_files();
                *STATE.completed_compile.write().unwrap() = Some(false);
            }
        }
    }
}
