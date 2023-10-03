use egui::{epaint::ahash::HashMap, Button, Layout, Ui};

use crate::{
    abi,
    app::RenderConfigs,
    backend,
    components::{CompiledContract, TargetMode, TestList, UtilityMenu},
    shared_state::STATE,
};
use eyre::Result;
use regex::Regex;
use std::{default, vec::Vec};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct SelectedTarget {
    pub name: String,
    pub mode: TargetMode,
    // pub file_path: String,
    // pub contract_name: String,
    // pub file_name: String,

    // // can probably make a new struct based on these two fields
    // pub is_raw_bytecode: bool,
    // pub is_empty_load: bool,
    // pub bytecode_to_deploy: String,

    // pub contract: Option<CompiledContract>,
    // // pub func_output: HashMap<String, String>, // need this? success/failure?
    // pub constructor_args: Vec<String>, // Added this line to handle constructor arguments
}

impl SelectedTarget {
    pub fn new_compiled(file_path: String) -> Self {
        let split_path: Vec<&str> = file_path.split('/').collect();
        let contract_name = split_path
            .last()
            .unwrap()
            .trim_end_matches(".json")
            .to_string();
        let file_name = split_path[split_path.len() - 2].to_string();

        Self {
            name: contract_name,
            mode: TargetMode::Compiled {
                file_path,

                file_name,
                contract: None,
                constructor_args: Vec::new(),
            },
        }
    }

    pub fn new_foundry_test(file_path: String) -> Self {
        let split_path: Vec<&str> = file_path.split('/').collect();
        let contract_name = split_path
            .last()
            .unwrap()
            .trim_end_matches(".json")
            .to_string();
        let file_name = split_path[split_path.len() - 2].to_string();

        Self {
            name: contract_name,
            mode: TargetMode::FoundryTest {
                file_path,
                file_name,
                contract: None,
                test_list: TestList::default(),
            },
        }
    }

    pub fn new_deploy_raw(bytecode_to_deploy: String) -> Self {
        Self {
            name: "Deploy Raw".to_string(),
            mode: TargetMode::DeployRaw { bytecode_to_deploy },
        }
    }

    pub fn new_load_raw() -> Self {
        Self {
            name: "Load Raw".to_string(),
            mode: TargetMode::LoadRaw {},
        }
    }

    pub fn show(&mut self, ui: &mut Ui, render_configs: &mut RenderConfigs) {
        match &mut self.mode {
            TargetMode::Compiled {
                // contract_name,
                contract,
                constructor_args,
                ..
            } => {
                ui.horizontal(|ui| {
                    ui.label(format!("Target: {:?}", self.name));
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button("Deploy").clicked() {
                            if let Some(c) = contract {
                                backend::deploy_wrapper(c.clone(), constructor_args.clone());
                            }
                        }
                        if let Some(c) = contract {
                            UtilityMenu::show_for_selected(ui, c)
                        }
                    });
                });

                // If we have a compiled contract, render the constructor args
                if let Some(contract) = &contract {
                    if let Some(abi) = contract.compiled_json["abi"].as_array() {
                        if let Some(constructor) = abi
                            .iter()
                            .find(|&item| item["type"].as_str().unwrap_or("") == "constructor")
                        {
                            if let Some(input_array) = constructor
                                .get("inputs")
                                .and_then(|inputs| inputs.as_array())
                                .filter(|a| !a.is_empty())
                            {
                                ui.label("Constructor args:");
                                for (index, param) in input_array.iter().enumerate() {
                                    let param_name = param["name"].as_str().unwrap_or("unnamed");
                                    let param_type =
                                        param["type"].as_str().unwrap_or("Unknown type");

                                    // Create an input field for each parameter
                                    let placeholder: String =
                                        format!("{}: {}", param_name, param_type);
                                    // Ensure there is enough space in the Vec
                                    while constructor_args.len() <= index {
                                        constructor_args.push(String::new());
                                    }
                                    let text_edit =
                                        egui::TextEdit::singleline(&mut constructor_args[index])
                                            .hint_text(&placeholder)
                                            .desired_width(f32::INFINITY);
                                    ui.add(text_edit);
                                }
                            }
                        }
                    }
                    // ui.separator();

                    ui.horizontal_top(|ui| {
                        // Load at address section for compiled
                        ui.label("Load target at: ");
                        egui::TextEdit::singleline(&mut render_configs.load_address)
                            .hint_text("Address".to_string())
                            .desired_width(ui.available_width() - 52.5) // sorry
                            .show(ui);

                        if ui.button("Load").clicked() {
                            backend::load_at_address_wrapper(
                                Some(contract.clone()),
                                render_configs.load_address.to_string(),
                            );
                            render_configs.load_address = String::new();
                        }
                    });
                }
            }

            TargetMode::FoundryTest {
                contract,
                test_list,
                ..
            } => {
                if let Some(compiled_test) = contract.clone() {
                    ui.label(format!("Target: {:?}", self.name));

                    // let mut test_list = TestList::default();
                    test_list.name = compiled_test.contract_name.clone();

                    test_list.show(ui, &compiled_test, &mut render_configs.verbosity);
                }
            }
            TargetMode::DeployRaw { bytecode_to_deploy } => {
                ui.horizontal(|ui| {
                    ui.label(format!("Target: {:?}", self.name));
                    if ui.button("Deploy").clicked() {
                        backend::deploy_raw_bytecode_wrapper(
                            bytecode_to_deploy.trim_start_matches("0x").to_string(),
                        );
                    }
                });
                let text_edit = egui::TextEdit::singleline(bytecode_to_deploy)
                    .hint_text("Enter init+runtime bytecode")
                    .desired_width(f32::INFINITY);
                ui.add(text_edit);
            }
            TargetMode::LoadRaw { .. } => {
                ui.label(format!("Target: {:?}", self.name));
                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        let address: &mut String = &mut render_configs.load_address;
                        egui::TextEdit::singleline(address)
                            .hint_text("Load address".to_string())
                            .show(ui);
                        if ui.button("Load").clicked() {
                            backend::load_at_address_wrapper(None, address.to_string());
                            render_configs.load_address = String::new();
                        }
                    });
                });
            }
        }
    }
}

// pub fn show(&mut self, ui: &mut Ui) {
//     // Show contract name, deploy button, and option to run pyrometer on it (move pyrometer out to dropdown?)
//     ui.horizontal(|ui| {
//         ui.label(format!("Target: {:?}", &self.contract_name));
//         // right align deploy and util buttons
//         ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
//             if !self.is_empty_load && ui.button("Deploy").clicked() {
//                 match self.is_raw_bytecode {
//                     true => backend::deploy_raw_bytecode_wrapper(
//                         self.bytecode_to_deploy.trim_start_matches("0x").to_string(),
//                     ),
//                     false => {
//                         if let Some(contract) = self.contract.clone() {
//                             backend::deploy_wrapper(contract, self.constructor_args.clone());
//                         }
//                     }
//                 }
//             }
//             if let Some(contract) = &mut self.contract {
//                 UtilityMenu::show_for_selected(ui, contract)
//             }
//         });
//     });

//     if self.is_raw_bytecode {
//         let text_edit = egui::TextEdit::singleline(&mut self.bytecode_to_deploy)
//             .hint_text("Enter init+runtime bytecode")
//             .desired_width(f32::INFINITY);

//         ui.add(text_edit);
//     } else if let Some(contract) = &self.contract {
//         // If we have a compiled contract, render the constructor args
//         if let Some(abi) = contract.compiled_json["abi"].as_array() {
//             if let Some(constructor) = abi
//                 .iter()
//                 .find(|&item| item["type"].as_str().unwrap_or("") == "constructor")
//             {
//                 if let Some(inputs) = constructor.get("inputs") {
//                     if let Some(input_array) = inputs.as_array() {
//                         ui.label("Constructor args:");
//                         for (index, param) in input_array.iter().enumerate() {
//                             let param_name = param["name"].as_str().unwrap_or("unnamed");
//                             let param_type = param["type"].as_str().unwrap_or("Unknown type");

//                             // Create an input field for each parameter
//                             let placeholder: String = format!("{}: {}", param_name, param_type);
//                             // Ensure there is enough space in the Vec
//                             while self.constructor_args.len() <= index {
//                                 self.constructor_args.push(String::new());
//                             }
//                             let text_edit =
//                                 egui::TextEdit::singleline(&mut self.constructor_args[index])
//                                     .hint_text(&placeholder)
//                                     .desired_width(f32::INFINITY);
//                             let resp = ui.add(text_edit);
//                             // if ui.ctx().wants_keyboard_input()
//                             //     && ui.ctx().input(|i| i.pointer.interact_pos().is_none())
//                             // {
//                             //     resp.surrender_focus();
//                             // };
//                         }
//                     }
//                 }
//             }
//         }
//     }
// Display input fields for constructor arguments
//     for (arg_name, arg_value) in &self.constructor_args {
//         let text: &mut String = arg_value.clone();
//         ui.horizontal(|ui| {
//             ui.label(format!("{}:", arg_name));
//             ui.add(egui::TextEdit::singleline(text));
//         });
//     }
// }

// fn extract_test_functions(compiled: serde_json::Value) -> Vec<String> {
//     // Initialize an empty vector to store function names
//     let mut function_names: Vec<String> = Vec::new();

//     // Check if "methodIdentifiers" object exists
//     if let Some(method_identifiers) = compiled.get("methodIdentifiers") {
//         // Iterate over each key in the "methodIdentifiers" object
//         for key in method_identifiers.as_object().unwrap().keys() {
//             // Remove the trailing parentheses from the function name
//             // let function_name = key.split("(").next().unwrap().to_string();

//             // Only add function names that start with "test" to the vector
//             if key.starts_with("test") {
//                 function_names.push(key.to_string());
//             }
//         }
//     }

//     function_names
// }
