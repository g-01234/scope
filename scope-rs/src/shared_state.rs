use crate::{
    components::{CompiledContract, DeployedContract, ReturnAndReceipt},
    providers::ClientProviderWrapper,
};
use egui::epaint::ahash::HashMap;
use ethers::types::{Address, U256};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

lazy_static! {
    pub static ref STATE: SharedState = SharedState::new();
}
// These locks are generally written to by the backend and read by the frontend
// Typical flow is:
//      1. User interacts with widget on frontend GUI, which calls a fn in (rust) backend
//      2. Backend posts a message to Javascript Land (VSCode) via the bridge
//         (e.g. `send_shell_command` or `get_open_files`)
//      3. JS Land performs its processing and posts a message back to the backend
//         via the exposed rust functions in the bridge
//      4. Backend updates the shared storage with new data
//      5. Frontend renders based on new data on next refresh (60hz)
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SharedState {
    // File data
    pub open_files: RwLock<Vec<String>>,
    pub file_contents: RwLock<Option<Vec<u8>>>,
    pub completed_compile: RwLock<Option<bool>>,
    pub target_compiled: RwLock<Option<CompiledContract>>,

    // VSCode data
    pub vscode_style: RwLock<serde_json::Value>,
    pub has_focus: RwLock<bool>,
    pub last_focus_change: RwLock<Option<f64>>,

    // Blockchain data
    #[serde(skip)]
    pub client: RwLock<Option<ClientProviderWrapper>>,
    pub tx_configs: RwLock<TxConfigs>,
    pub endpoint: RwLock<String>,
    pub deployed_addresses: RwLock<Vec<String>>,
    pub deployed_contracts: RwLock<IndexMap<String, DeployedContract>>,
    pub from_addresses: RwLock<Vec<Address>>,
    pub temp_fn_output: RwLock<HashMap<String, HashMap<String, ReturnAndReceipt>>>,
    pub func_last_tx: RwLock<HashMap<String, ReturnAndReceipt>>,

    // Hacky egui globals
    pub max_width: RwLock<f32>,
    pub max_collapsable_width: RwLock<f32>,
}

impl SharedState {
    pub fn new() -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        Self {
            // endpoint: RwLock::new("https://tinyurl.com/2w7bxmjx".to_string()),
            endpoint: RwLock::new("http://127.0.0.1:8545".to_string()),
            has_focus: RwLock::new(true),
            ..Default::default()
        }
    }
}

// should this derive default or need to set?
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct TxConfigs {
    pub from_address: Address,
    pub gas_price: U256,
    pub gas_limit: String,
    pub value: String,
    pub nonce: String,
}

impl Default for TxConfigs {
    fn default() -> Self {
        Self {
            from_address: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string()
                .parse()
                .unwrap(),
            gas_price: (1e9 as u64).into(),
            gas_limit: "".to_string(),
            value: "".to_string(),
            nonce: "".to_string(),
        }
    }
}

// Helper fns to avoid RwLock syntax
// Gets a clone
pub fn read_tx_configs() -> TxConfigs {
    let tx_configs = STATE.tx_configs.read().unwrap();
    tx_configs.clone()
}

// Gets a clone, can borrow if this gets slow
pub fn read_shared_client() -> Result<ClientProviderWrapper, eyre::Report> {
    let client_guard = STATE.client.read().unwrap();
    if let Some(client_wrapper) = client_guard.as_ref() {
        Ok(client_wrapper.clone()) // Assumes ClientProviderWrapper is Clone
    } else {
        Err(eyre::eyre!("Client is not initialized"))
    }
}
