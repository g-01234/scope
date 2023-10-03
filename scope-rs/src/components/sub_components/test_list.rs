use egui::{epaint::ahash::HashMap, Ui};

use crate::{backend, components::CompiledContract, shared_state};
use regex::Regex;
use std::vec::Vec;

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct TestList {
    pub name: String,
    pub func_output: HashMap<String, String>, // need this? success/failure?
    pub test_configs: TestConfigs,
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct TestConfigs {
    pub gas_report: bool,
    pub fork: bool,
    pub debug: bool,
}

impl TestList {
    // Add a new member to store the verbosity level

    pub fn show(&mut self, ui: &mut Ui, compiled: &CompiledContract, verbosity: &mut i32) {
        // Add a slider at the top to adjust the verbosity level
        let gas_report = &mut self.test_configs.gas_report;
        let fork = &mut self.test_configs.fork;
        let debug = &mut self.test_configs.debug;
        ui.horizontal(|ui| {
            ui.label("Verbosity: ");
            ui.add(egui::Slider::new(verbosity, 1..=4));
            ui.menu_button("Options", |ui| {
                ui.checkbox(gas_report, "gas report");
                ui.checkbox(fork, "fork (anvil -f)");
                // ui.checkbox(debug, "debug"); // CURRENTLY BUGGED
            });
        });

        // Check marks for gas report and fork

        for test in extract_test_functions(compiled.compiled_json.clone()) {
            if ui.button(test.clone()).clicked() {
                // Create the verbosity string based on the slider value
                let verbosity_str = "-".to_string() + &"v".repeat(*verbosity as usize);

                // Initialize an empty string to hold command options
                let mut extra_options = String::new();

                // Append "--gas-report" if gas_report is true
                if *gas_report {
                    extra_options.push_str("--gas-report ");
                }

                // Append the endpoint from the client wrapper if fork is true
                if *fork {
                    if let Ok(client_wrapper) = shared_state::read_shared_client() {
                        let endpoint = client_wrapper.endpoint; // Replace with how you get the endpoint
                        extra_options.push_str(&format!("--fork-url {} ", endpoint));
                    }
                }

                // Construct the final command
                let final_command = format!(
                    "forge test --match-test {:?} {} {}",
                    test, verbosity_str, extra_options
                );

                // Send the final command
                backend::send_shell_command(final_command.trim().to_string()); // Using trim to remove any trailing spaces
            }
        }
    }
}

// fn extract_test_functions(solidity_code: &str) -> Vec<String> {
//     // Initialize an empty vector to store function names
//     let mut function_names: Vec<String> = Vec::new();

//     // Create a regular expression to match function definitions
//     let re = Regex::new(r"function\s+(\w+)\s*\(").unwrap();

//     // Iterate over each capture group and extract function names
//     for cap in re.captures_iter(solidity_code) {
//         let function_name = cap[1].to_string();

//         // Only add function names that start with "test" to the vector
//         if function_name.starts_with("test") {
//             function_names.push(function_name);
//         }
//     }

//     function_names
// }

fn extract_test_functions(compiled: serde_json::Value) -> Vec<String> {
    // Initialize an empty vector to store function names
    let mut function_names: Vec<String> = Vec::new();

    // Check if "methodIdentifiers" object exists
    if let Some(method_identifiers) = compiled.get("methodIdentifiers") {
        // Iterate over each key in the "methodIdentifiers" object
        for key in method_identifiers.as_object().unwrap().keys() {
            // Remove the trailing parentheses from the function name
            let function_name = key.split('(').next().unwrap().to_string();

            // Only add function names that start with "test" to the vector
            if key.starts_with("test") {
                function_names.push(function_name);
            }
        }
    }

    function_names
}
