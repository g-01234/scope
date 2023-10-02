// use eth_toolkit::shared_storage::{DeployedContract, BRIDGE};
use crate::{
    abi, backend,
    components::CompiledContract,
    shared_state::{self, STATE},
    utils,
};
use egui::{
    epaint::ahash::{HashMap, HashMapExt}, // why am i using egui version again?
    Ui,
};
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::sync::{Arc, Mutex};

use ethers::{
    abi::{token, Abi, ParamType, Token},
    prelude::{LocalWallet, MnemonicBuilder, Provider, SignerMiddleware},
    providers::{Http, Middleware, ProviderExt, Ws},
    signers::Signer,
    solc::artifacts::Return,
    types::{
        transaction::eip2718::TypedTransaction, Address, Block, BlockNumber, Bytes,
        Eip1559TransactionRequest, NameOrAddress, TransactionRequest, H160, H256, U256,
    },
    utils::parse_ether,
};

use super::UtilityMenu;

// use ethers::contract::Contract;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DeployedContract {
    pub name: String,
    pub abi: Value,              // move to compiled_contract
    pub bytecode_b: Bytes,       // move to compiled_contract
    pub bytecode_string: String, // move to compiled_contract
    pub address_h160: H160,
    pub address_string: String,
    pub deployed_block: Block<H256>,
    pub storage_slot_input: String,
    pub storage_value_input: String,
    pub func_param_input: HashMap<String, HashMap<String, String>>, //func_name -> param name -> param input;
    pub func_output: HashMap<String, ReturnAndReceipt>,
    pub calldata_input: String,
    pub compiled_contract: Option<CompiledContract>,

    // this is def not how we want to do this but fine for mvp
    pub func_last_tx: HashMap<String, ReturnAndReceipt>, // func name: most recent tx?
}

impl DeployedContract {
    pub fn show(&mut self, ui: &mut Ui) {
        self.show_storage(ui);
        self.show_functions_and_inputs(ui);
        self.show_calldata_area(ui);
    }

    fn show_storage(&mut self, ui: &mut Ui) {
        ui.label("Storage");
        let slot: &mut String = &mut self.storage_slot_input;
        let value: &mut String = &mut self.storage_value_input;
        ui.horizontal_top(|ui| {
            if ui.button("Get").clicked() {
                // cast call
                backend::send_shell_command(format!(
                    "cast storage {:?} {:?}",
                    self.address_h160, slot
                ));
            }

            egui::TextEdit::singleline(slot)
                .hint_text("slot")
                .desired_width(ui.available_width())
                .show(ui);
        });

        ui.horizontal_top(|ui| {
            if ui.button("Set").clicked() {
                // Determine the radix based on string prefix
                if (!value.is_empty() && !slot.is_empty()) {
                    let (radix_slot, clean_slot) = utils::get_radix_and_clean_str(slot);
                    let (radix_value, clean_value) = utils::get_radix_and_clean_str(value);

                    // Attempt to convert `value` and `slot` to U256
                    match (
                        ethers::types::U256::from_str_radix(clean_slot, radix_slot),
                        ethers::types::U256::from_str_radix(clean_value, radix_value),
                    ) {
                        (Ok(u256_slot), Ok(u256_value)) => {
                            // Encode to ABI tokens and send to storage

                            let encoded_slot = ethers::abi::AbiEncode::encode_hex(u256_slot);
                            let encoded_value = ethers::abi::AbiEncode::encode_hex(u256_value);

                            backend::set_storage_at(self.address_h160, encoded_slot, encoded_value);
                        }
                        (Err(e), _) | (_, Err(e)) => {
                            // Send an error popup if conversion fails
                            backend::send_error_popup(format!("Conversion ERROR: {:?}", e));
                        }
                    }
                } else {
                    backend::send_error_popup("Empty slot or value".to_string());
                }
            }
            egui::TextEdit::singleline(value)
                .hint_text("value")
                .desired_width(ui.available_width())
                .show(ui);
        });
    }

    fn show_functions_and_inputs(&mut self, ui: &mut Ui) {
        if let Some(abi_array) = self.abi.as_array() {
            // Collect functions into a Vec

            let mut functions: Vec<&serde_json::Value> = abi_array
                .iter()
                .filter(|abi_item| {
                    if let Some(abi_type) = abi_item.get("type") {
                        abi_type == "function"
                    } else {
                        false
                    }
                })
                .collect();

            // Sort functions based on state mutability
            // Function to determine the priority of state mutability
            let get_priority = |m: &str| -> i32 {
                match m {
                    "pure" => 1,
                    "view" => 2,
                    _ => 3,
                }
            };

            // Sort functions based on state mutability and then by name
            functions.sort_by_key(|a| {
                let mutability = a["stateMutability"].as_str().unwrap_or("");
                let name = a["name"].as_str().unwrap_or("");
                (get_priority(mutability), name)
            });

            // Iterate through the sorted functions to display the buttons
            for abi_item in functions.iter() {
                if let Some(abi_type) = abi_item.get("type") {
                    if abi_type == "function" {
                        ui.separator();
                        let func_name = abi_item["name"].as_str().unwrap_or("Unnamed function");

                        // cool
                        let is_static = matches!(
                            abi_item["stateMutability"].as_str(),
                            Some("view") | Some("pure")
                        );

                        ui.horizontal(|ui| {
                            // button color based on state mutability

                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Min)
                                    .with_cross_align(egui::Align::Min),
                                |ui| {
                                    ui.style_mut().visuals.override_text_color =
                                        Some(egui::Color32::BLACK);
                                    match is_static {
                                        true => {
                                            ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                                egui::Color32::GRAY;
                                        }
                                        _ => {
                                            ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                                egui::Color32::LIGHT_GRAY;
                                        }
                                    }
                                    if ui.button(format!("{}()", func_name)).clicked() {
                                        // Clone only the parts of `self` that you need.
                                        match is_static {
                                            true => {
                                                self.cursed_staticcall_wrapper(
                                                    func_name.to_string(),
                                                );
                                            }
                                            _ => {
                                                self.cursed_send_wrapper(func_name.to_string());
                                            }
                                        }
                                    }
                                },
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                UtilityMenu::show_for_function(
                                    ui,
                                    self,
                                    self.abi.clone(),
                                    abi_item,
                                    func_name.to_string(),
                                );
                            });
                        });

                        // Create input fields for each parameter
                        if let Some(inputs) = abi_item.get("inputs") {
                            if let Some(input_array) = inputs.as_array() {
                                for param in input_array.iter() {
                                    let param_name =
                                        param["name"].as_str().unwrap_or("Unnamed parameter");
                                    let param_type =
                                        param["type"].as_str().unwrap_or("Unknown type");

                                    // Create an input field for each parameter
                                    let placeholder: String =
                                        format!("{}: {}", param_name, param_type);
                                    let func_name = func_name.to_string();
                                    let param_name = param_name.to_string();
                                    let text: &mut String = self
                                        .func_param_input
                                        .entry(func_name.clone())
                                        .or_insert_with(HashMap::new)
                                        .entry(param_name.clone())
                                        .or_default();
                                    // let id = egui::Id::new(param_name.clone());

                                    ui.set_max_width(
                                        ui.available_width() + ui.spacing().item_spacing.x,
                                    );
                                    let text_edit = egui::TextEdit::singleline(text)
                                        .hint_text(&placeholder)
                                        .desired_width(f32::INFINITY)
                                        .show(ui);
                                    ui.set_max_width(
                                        ui.available_width() - ui.spacing().item_spacing.x,
                                    );
                                }
                            }
                        }

                        // Show return data
                        // Shared state wrangling
                        let temp_fn_output_lock = STATE.temp_fn_output.read().unwrap();
                        // Attempt to get the inner HashMap using self.address_h160 as the key
                        if let Some(inner_map) =
                            temp_fn_output_lock.get(&format!("{:#x}", self.address_h160))
                        {
                            // Attempt to get the specific output value using func_name as the key
                            if let Some(ret) = inner_map.get(func_name) {
                                // Insert the retrieved output_value into self.func_output
                                self.func_output.insert(func_name.to_string(), ret.clone());
                            }
                        }

                        // Draw the return data if available
                        if let Some(result) = self.func_output.get(func_name) {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("return:");
                                // CopyButton::new(
                                //     "Copy Raw".to_string(),
                                //     format!("{}", result.return_output),
                                // )
                                // .show(ui);
                            });

                            // Get the return type from the ABI
                            if let Some(outputs) = abi_item.get("outputs") {
                                if let Some(output_array) = outputs.as_array() {
                                    for (index, output) in output_array.iter().enumerate() {
                                        let index_string = index.to_string();

                                        let output_type =
                                            output["type"].as_str().unwrap_or("Unknown type");
                                        let output_name = match output["name"].as_str() {
                                            Some(name) if !name.is_empty() => format!("{} ", name),
                                            _ => format!("val_{}", index_string.clone()),
                                        };

                                        let decode_result = abi::decode_return_values(
                                            func_name.to_string(),
                                            &self.abi,
                                            &result.return_output,
                                        );
                                        let formatted_value = match decode_result {
                                            Ok(decoded_return) => {
                                                let return_value = &decoded_return[index];

                                                let zeroes_to_prepend =
                                                    if format!("{}", return_value).len() % 2 == 1 {
                                                        "0"
                                                    } else {
                                                        ""
                                                    };

                                                match output_type {
                                                    t if t.starts_with("uint")
                                                        || t.starts_with("int") =>
                                                    {
                                                        if let Token::Uint(value)
                                                        | Token::Int(value) = return_value
                                                        {
                                                            format!(
                                                                "0x{}{:x}",
                                                                zeroes_to_prepend, value
                                                            )
                                                        } else {
                                                            format!("{}", return_value)
                                                        }
                                                    }
                                                    t if t.starts_with("bytes")
                                                        || t == "address" =>
                                                    {
                                                        format!(
                                                            "0x{}{}",
                                                            zeroes_to_prepend, return_value
                                                        )
                                                    }
                                                    _ => format!("{}", return_value),
                                                }
                                            }
                                            Err(e) => "0x".to_string(), // error decoding output, just print 0x
                                        };

                                        ui.add(
                                            egui::TextEdit::multiline(
                                                &mut format!(
                                                    "{} ({}): {}  \n",
                                                    output_name, output_type, formatted_value
                                                )
                                                .trim(),
                                            )
                                            .desired_rows(1)
                                            .desired_width(f32::INFINITY),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn show_calldata_area(&mut self, ui: &mut Ui) {
        ui.separator();

        ui.label("Enter Raw Calldata:");
        let calldata: &mut String = &mut self.calldata_input;
        egui::TextEdit::multiline(calldata)
            .desired_width(f32::INFINITY)
            .show(ui);
        // Display buttons for raw verisons of "send" and "staticcall"
        ui.horizontal(|ui| {
            // optionally strip 0x prefix
            let input = self
                .calldata_input
                .strip_prefix("0x")
                .unwrap_or(&self.calldata_input)
                .to_string();

            if ui.button("Send").clicked() {
                self.cursed_send_raw_wrapper(input);
            } else if ui.button("Staticcall").clicked() {
                self.cursed_staticcall_raw_wrapper(input);
            }
        });

        // YUCK!
        let temp_fn_output_lock = STATE.temp_fn_output.read().unwrap();
        // Attempt to get the inner HashMap using self.address_h160 as the key
        if let Some(inner_map) = temp_fn_output_lock.get(&format!("{:#x}", self.address_h160)) {
            // Attempt to get the specific output value using func_name as the key
            if let Some(ret) = inner_map.get("hope nobody else uses this name for a function") {
                // Insert the retrieved output_value into self.func_output
                self.func_output.insert(
                    "hope nobody else uses this name for a function".to_string(),
                    ret.clone(),
                );
            }
        }
        if let Some(result) = self
            .func_output
            .get("hope nobody else uses this name for a function")
        {
            ui.add(
                egui::TextEdit::multiline(
                    &mut format!("Return: \n{}", result.return_output).trim(),
                )
                .desired_rows(1),
            );
        }
    }

    pub fn cursed_send_wrapper(&self, func_name: String) {
        // cloning makes
        let address_h160 = self.address_h160;
        let abi = self.abi.clone();
        let func_param_input = self.func_param_input.clone();

        // Spawn the future
        wasm_bindgen_futures::spawn_local(async move {
            // have to qualify here bc middleware also has a send_transaction?
            let res = DeployedContract::send_transaction(
                address_h160,
                abi,
                func_name.to_string(),
                func_param_input,
            )
            .await;

            let mut temp_fn_output_write_lock = STATE.temp_fn_output.write().unwrap();
            let inner_map = temp_fn_output_write_lock
                .entry(format!("{:#x}", address_h160))
                .or_insert_with(HashMap::new);

            match res {
                Ok(ret) => {
                    inner_map.insert(func_name.to_string(), ret);
                }
                Err(e) => {
                    backend::send_error_popup(format!("ERROR: {}", e));
                }
            }
        });
    }

    async fn send_transaction(
        address: H160,
        json_abi: Value,
        func_name: String,
        func_param_input: HashMap<String, HashMap<String, String>>,
    ) -> Result<ReturnAndReceipt> {
        let client_wrapper = shared_state::read_shared_client()?;
        let client = client_wrapper.client;

        let tx_configs = shared_state::read_tx_configs();

        let ethers_abi: ethers::abi::Abi = serde_json::from_value(json_abi.clone()).unwrap();
        let ethers_contract =
            ethers::contract::Contract::new(address, ethers_abi, Arc::new(client.clone()));

        let tokens =
            abi::encode_fn_call_to_tokens(func_name.to_string(), json_abi, func_param_input)?;
        // log!("{:?}", tokens);

        let mut fn_call = ethers_contract.method::<_, Vec<Token>>(&func_name, &tokens[..])?;
        fn_call.tx.set_from(tx_configs.from_address);

        fn_call
            .tx
            .set_value(utils::eth_str_to_u256_wei(&tx_configs.value)?);

        log!("{:?}", fn_call.tx);
        let static_return = client.provider().call_raw(&fn_call.tx).await?;

        let tx_pending = fn_call.send().await?;

        let tx_receipt = tx_pending.await?.unwrap();
        // log!("{:?}", receipt);

        Ok(ReturnAndReceipt {
            tx_receipt: Some(tx_receipt),
            return_output: static_return,
        })
    }

    pub fn cursed_staticcall_wrapper(&self, func_name: String) {
        let to_address = self.address_h160;
        let abi = self.abi.clone();
        let func_param_input = self.func_param_input.clone();

        // Spawn the future
        wasm_bindgen_futures::spawn_local(async move {
            let res = DeployedContract::staticcall(
                to_address,
                abi,
                func_name.to_string(),
                func_param_input,
            )
            .await;

            let mut temp_fn_output_write_lock = STATE.temp_fn_output.write().unwrap();
            let inner_map = temp_fn_output_write_lock
                .entry(format!("{:#x}", to_address))
                .or_insert_with(HashMap::new);

            match res {
                Ok(ret) => {
                    inner_map.insert(func_name.to_string(), ret);
                }
                Err(e) => backend::send_error_popup(format!("ERROR: {}", e)),
            }
        });
    }

    async fn staticcall(
        address_h160: H160,
        json_abi: Value,
        func_name: String,
        func_param_input: HashMap<String, HashMap<String, String>>,
    ) -> Result<ReturnAndReceipt> {
        let client_wrapper = shared_state::read_shared_client()?;
        let client = client_wrapper.client;

        let tx_configs = shared_state::read_tx_configs();

        let abi: ethers::abi::Abi = serde_json::from_value(json_abi.clone())?;

        let ethers_contract =
            ethers::contract::Contract::new(address_h160, abi, Arc::new(client.clone()));

        // let mut tokens: Vec<Token> = Vec::new();
        // let tokens = deployed_contract.abi_encode_to_tokens(func_name.to_string())?;
        let tokens = abi::encode_fn_call_to_tokens(
            func_name.to_string(),
            json_abi.clone(),
            func_param_input,
        )?;

        // create the function call using the typed/encoded tokens
        let mut call = ethers_contract
            .method::<_, Vec<Token>>(&func_name, &tokens[..])
            .expect("Failed to generate call");

        call.tx.set_from(tx_configs.from_address);

        // annoying, want to abstract this to another fn

        call.tx
            .set_value(utils::eth_str_to_u256_wei(&tx_configs.value)?);

        let res = client
            .provider()
            .call_raw(&call.tx)
            .await
            .map_err(|error| {
                log::error!("ProviderError encountered: {:?}", error);
                Box::new(error)
            })?;
        Ok(ReturnAndReceipt {
            return_output: res,
            tx_receipt: None,
        })
    }

    pub fn cursed_send_raw_wrapper(&self, raw_calldata: String) {
        let address_h160 = self.address_h160;

        // Spawn the future
        wasm_bindgen_futures::spawn_local(async move {
            let res = DeployedContract::send_transaction_raw(address_h160, raw_calldata).await;

            log!("{:?}", res);
            let mut temp_fn_output_write_lock = STATE.temp_fn_output.write().unwrap();
            let inner_map = temp_fn_output_write_lock
                .entry(format!("{:#x}", address_h160))
                .or_insert_with(HashMap::new);

            match res {
                Ok(ret) => {
                    inner_map.insert(
                        "hope nobody else uses this name for a function".to_string(),
                        ret,
                    );
                }
                Err(e) => {
                    backend::send_error_popup(format!("ERROR: {}", e));
                }
            }
        });
    }

    async fn send_transaction_raw(
        address_h160: H160,
        raw_calldata: String,
    ) -> Result<ReturnAndReceipt> {
        // Decode the provided calldata into ethers Bytes
        let calldata_bytes =
            Bytes::from(hex::decode(&raw_calldata).expect("Error decoding calldata"));

        let client_wrapper = shared_state::read_shared_client()?;
        let client = client_wrapper.client;

        let tx_configs = shared_state::read_tx_configs();

        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::default());

        // set the fron before we fill so we get the correct nonce
        tx.set_from(tx_configs.from_address);
        client.fill_transaction(&mut tx, None).await?;

        // double the estimated gas limit
        // hack until we implement gas field
        tx.set_gas(tx.clone().gas().unwrap_or(&U256::zero()) * U256::from(2));

        tx.set_value(utils::eth_str_to_u256_wei(&tx_configs.value)?);

        tx.set_to(address_h160);
        tx.set_data(calldata_bytes);

        log!("{:?}", tx);
        let static_return = client.provider().call_raw(&tx).await?;
        log!("static return: {:?}", static_return);
        // Send the transaction
        let pending_tx = client.send_transaction(tx.clone(), None).await?;
        log!("pending_tx: {:?}", pending_tx);
        // Wait for the transaction to be mined and get the receipt
        match pending_tx.await? {
            Some(receipt) => Ok(ReturnAndReceipt {
                tx_receipt: Some(receipt),
                return_output: static_return,
            }),
            None => Err(eyre!("Transaction receipt is None")),
        }
    }

    pub fn cursed_staticcall_raw_wrapper(&self, raw_calldata: String) {
        let address_h160 = self.address_h160;

        // Spawn the future
        wasm_bindgen_futures::spawn_local(async move {
            let res = DeployedContract::staticcall_raw(address_h160, raw_calldata).await;

            log!("{:?}", res);
            let mut temp_fn_output_write_lock = STATE.temp_fn_output.write().unwrap();
            let inner_map = temp_fn_output_write_lock
                .entry(format!("{:#x}", address_h160))
                .or_insert_with(HashMap::new);

            match res {
                Ok(ret) => {
                    inner_map.insert(
                        "hope nobody else uses this name for a function".to_string(),
                        ret,
                    );
                }
                Err(e) => {
                    backend::send_error_popup(format!("ERROR: {}", e));
                }
            }
        });
    }

    async fn staticcall_raw(address_h160: H160, raw_calldata: String) -> Result<ReturnAndReceipt> {
        // Decode the provided calldata string into ethers Bytes

        let calldata_bytes = Bytes::from(
            hex::decode(raw_calldata.strip_prefix("0x").unwrap_or(&raw_calldata))
                .expect("Error decoding calldata"),
        );

        let client_wrapper = shared_state::read_shared_client()?;
        let client = client_wrapper.client;

        let tx_configs = shared_state::read_tx_configs();

        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::default());

        // set the fron before we fill so we get the correct nonce
        tx.set_from(tx_configs.from_address);
        client.fill_transaction(&mut tx, None).await?;
        tx.set_to(address_h160);
        tx.set_value(utils::eth_str_to_u256_wei(&tx_configs.value)?);
        tx.set_data(calldata_bytes);

        log!("{:?}", tx);
        let result = client.provider().call_raw(&tx).await?;
        log!("staticcall result: {:?}", result);

        Ok(ReturnAndReceipt {
            return_output: result,
            tx_receipt: None,
        })
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ReturnAndReceipt {
    pub tx_receipt: Option<ethers::types::TransactionReceipt>, // Staticalls won't have a receipt
    pub return_output: Bytes,
}
