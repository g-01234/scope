use crate::{
    abi, backend,
    components::{CompiledContract, DeployedContract},
    shared_state,
};
use egui::Ui;
use ethers::etherscan::contract;
use eyre::Result;

// this should be an impl for widget or something
pub struct UtilityMenu {}

impl UtilityMenu {
    // Root level utilities
    pub fn show_for_root(ui: &mut Ui) {
        ui.menu_button("🔨", |ui| {
            if ui.button("Chisel").clicked() {
                let output = format!("chisel");
                backend::send_shell_command(output);
            }
            if ui.button("Slither").clicked() {
                // very annoying rustfmt bug; stops formatting if this whole string is together in a closure
                let mut slither_cmd = "slither . --filter-paths 'lib' ".to_string();
                slither_cmd.push_str("--exclude-informational --exclude-low");

                backend::send_shell_command(slither_cmd);
            }
        });
    }

    pub fn show_for_selected(ui: &mut Ui, contract: &mut CompiledContract) {
        ui.menu_button("🔨", |ui| {
            // idk if the same metadata will always be enabled? better way to handle errors than this?
            if let Some((file_path, contract_name)) = (|| {
                let metadata = contract.compiled_json.get("metadata")?;
                let settings = metadata.get("settings")?;
                let compilation_target = settings.get("compilationTarget")?;
                let map = compilation_target.as_object()?;
                map.iter().next()
            })() {
                if ui.button("Pyrometer").clicked() {
                    let output = format!("pyrometer {}", file_path,);
                    backend::send_shell_command(output);
                    // ui.output_mut(|o| o.copied_text = output);
                    ui.close_menu();
                }

                if ui.button("Storage Layout").clicked() {
                    let output = format!(
                        "forge inspect {}:{} storage --pretty",
                        file_path,
                        contract_name // is there a more concise way to do this...
                            .to_string()
                            .trim_end_matches('"')
                            .trim_start_matches('"')
                    );
                    backend::send_shell_command(output);
                    // ui.output_mut(|o| o.copied_text = output);
                    ui.close_menu();
                }
            }
            // call cast interface
            if ui.button("Cast Interface").clicked() {
                let output = format!(
                    "cast interface {}",
                    contract.file_path.split("//").last().unwrap() // format is file://repo_root/out/File.sol/Contract.json
                );
                backend::send_shell_command(output);
                ui.close_menu();
            }

            ui.menu_button("Copy", |ui| {
                if let Some(creation_bytecode) = contract.compiled_json.get("bytecode") {
                    if ui.button("Copy Creation Code").clicked() {
                        let output = format!("{}", creation_bytecode["object"])
                            .trim_end_matches('"')
                            .trim_start_matches('"')
                            .to_string();
                        ui.output_mut(|o| o.copied_text = output);
                        ui.close_menu();
                    }
                }

                if let Some(run_bytecode) = contract.compiled_json.get("deployedBytecode") {
                    if ui.button("Copy Run Code").clicked() {
                        let output = format!("{}", run_bytecode["object"])
                            .trim_end_matches('"')
                            .trim_start_matches('"')
                            .to_string();
                        ui.output_mut(|o| o.copied_text = output);
                        ui.close_menu();
                    }
                }
                if let Some(ast) = contract.compiled_json.get("ast") {
                    if ui.button("Copy AST").clicked() {
                        ui.output_mut(|o| o.copied_text = ast.to_string());
                        ui.close_menu();
                    }
                }

                if let Some(fns) = contract.compiled_json.get("methodIdentifiers") {
                    if ui.button("Copy fns + selectors").clicked() {
                        ui.output_mut(|o| o.copied_text = fns.to_string());
                        ui.close_menu();
                    }
                }

                if let Some(abi) = contract.compiled_json.get("abi") {
                    if ui.button("Copy ABI").clicked() {
                        ui.output_mut(|o| o.copied_text = abi.to_string());
                        ui.close_menu();
                    }
                }
            });
        });
    }

    pub fn show_for_deployed(ui: &mut Ui, contract: &DeployedContract) -> Result<()> {
        ui.menu_button("🔨", |ui| {
            if ui.button("Get Deployed Bytecode").clicked() {
                backend::send_shell_command(format!("cast code {}", contract.address_string));
            }

            if ui.button("Get Balance").clicked() {
                backend::send_shell_command(format!("cast balance {}", contract.address_string));
            }
        });

        Ok(())
    }

    pub fn show_for_function(
        ui: &mut Ui,
        contract: &DeployedContract,
        abi: serde_json::Value,
        abi_item: &&serde_json::Value,
        func_name: String,
    ) {
        ui.menu_button("🔨", |ui| {
            // idk if the same metadata will always be enabled? better way to handle errors than nested .gets?
            if ui.button("Copy raw calldata").clicked() {
                match abi::encode_fn_call_to_calldata(
                    func_name.to_string(),
                    &abi,
                    &contract.func_param_input,
                ) {
                    Ok(calldata) => ui.output_mut(|o| o.copied_text = calldata),
                    Err(_) => backend::send_error_popup("Failed to encode calldata".to_string()),
                }
                ui.close_menu();
            }
            if let Some(ret) = contract.func_output.get(&func_name) {
                if ui.button("Copy raw return").clicked() {
                    ui.output_mut(|o| o.copied_text = format!("{}", ret.return_output));

                    ui.close_menu();
                }
            }

            if ui.button("Trace").clicked() {
                match abi::encode_fn_call_to_calldata(
                    func_name.to_string(),
                    &abi,
                    &contract.func_param_input.clone(),
                ) {
                    Ok(calldata) => {
                        let tx_configs = shared_state::read_tx_configs();
                        backend::send_shell_command(format!(
                            "cast call --trace {:?} {:?} --value {:?}",
                            contract.address_h160, calldata, tx_configs.value
                        ));
                    }
                    Err(_) => backend::send_error_popup("Failed to encode calldata".to_string()),
                }

                ui.close_menu();
            }

            if ui.button("Debug").clicked() {
                match abi_item["stateMutability"].as_str() {
                    Some("view") | Some("pure") => {
                        if let Ok(calldata) = abi::encode_fn_call_to_calldata(
                            func_name.to_string(),
                            &abi,
                            &contract.func_param_input.clone(),
                        ) {
                            log!("calldata: {:?}", calldata);
                            backend::send_shell_command(format!(
                                "cast call --trace {:?} {:?}",
                                contract.address_h160, calldata
                            ));
                        }
                    }
                    _ => {
                        if let Some(ret) = contract.func_output.get(&func_name) {
                            if let Some(receipt) = ret.tx_receipt.clone() {
                                backend::send_shell_command(format!(
                                    "cast run --debug {:?}",
                                    receipt.transaction_hash
                                ));
                            }
                        }
                    }
                }
                ui.close_menu();
            }
        });
    }
}
