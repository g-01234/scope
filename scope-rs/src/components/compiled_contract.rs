use crate::{backend, components::CopyButton, shared_state::STATE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct CompiledContract {
    pub contract_name: String,
    pub file_name: String,
    pub file_path: String,
    pub abi: Value,
    // pub bytecode_b: Vec<u8>,
    pub bytecode_string: String,
    pub compiled_json: Value,
}

impl CompiledContract {
    pub fn new(file_path: String, compiled_json: Value) -> Self {
        let split_path: Vec<&str> = file_path.split('/').collect();
        let contract_name = split_path
            .last()
            .unwrap()
            .trim_end_matches(".json")
            .to_string();
        let file_name = split_path[split_path.len() - 2].to_string();

        // i believe we can rely on forge's solc output to have these fields
        // can we not clone here?
        // TODO: better error handling on these two
        let abi = compiled_json["abi"].clone();
        // let abi = serde_json::to_string(&compiled_json["abi"]).unwrap();
        let bytecode_string =
            compiled_json["bytecode"]["object"].as_str().unwrap()[2..].to_string();

        Self {
            contract_name,
            file_name,
            file_path,
            abi,
            bytecode_string,
            compiled_json,
        }
    }
}
