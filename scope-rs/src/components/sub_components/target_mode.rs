use crate::components::{CompiledContract, TestList};
use serde_json::Value;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub enum TargetMode {
    Compiled {
        file_path: String,

        file_name: String,
        contract: Option<CompiledContract>,
        constructor_args: Vec<String>,
    },
    FoundryTest {
        file_path: String,
        file_name: String,
        contract: Option<CompiledContract>,
        test_list: TestList,
    },
    DeployRaw {
        // contract_name: String, // RawBytecode
        bytecode_to_deploy: String,
    },
    LoadRaw {
        // contract_name: String,
        // abi: Value,
    },
}
