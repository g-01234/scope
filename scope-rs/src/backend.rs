use crate::{
    components::{CompiledContract, DeployedContract},
    providers::{self},
    shared_state::{self, STATE},
    utils, wasm,
};
use ethers::{
    contract::{ContractFactory, ContractInstance},
    prelude::{LocalWallet, Provider, SignerMiddleware},
    providers::{Http, Middleware},
    types::{BlockNumber, H160, U256},
    utils::parse_ether,
};
use eyre::Result;
use hex;

use serde_json::Value;

use std::sync::Arc;

// Functions for sending messages/queries to the extension
// These will generally call a javascript function exposed in the bridge
pub fn query_for_vscode_style() -> Result<Value> {
    let received_style = wasm::get_vscode_style();
    let json_str = received_style.as_string().unwrap();
    let parsed_style_json: Value = serde_json::from_str(&json_str).unwrap();
    Ok(parsed_style_json)
}

pub fn query_for_open_files() -> Result<()> {
    log!("querying for open files");
    wasm::get_open_files();

    Ok(())
}

pub fn query_for_file_contents(file_path: String) -> Result<()> {
    log!("querying for file contents");
    wasm::get_file_contents(file_path);

    Ok(())
}

pub fn query_for_compiled_solidity(file_path: String) -> Result<()> {
    log!("querying for file contents");
    wasm::get_compiled_solidity(file_path);
    Ok(())
}
pub fn send_shell_command(command: String) -> Result<()> {
    log!("sending shell command");
    wasm::execute_shell_command(command);
    Ok(())
}
pub fn send_forge_build() -> Result<()> {
    log!("requesting forge build");
    wasm::forge_build();
    Ok(())
}

pub fn send_error_popup(error_text: String) {
    wasm::send_error_to_vscode(error_text);
}

pub fn send_ok_popup(ok_text: String) {
    wasm::send_ok_to_vscode(ok_text);
}

pub fn initialize() -> Result<()> {
    query_for_open_files();
    wasm_bindgen_futures::spawn_local(async {
        // Set up the client
        let endpoint = STATE.endpoint.read().unwrap().to_string();
        let client_wrapper = providers::ClientProviderWrapper::new(endpoint)
            .await
            .expect("Could not initialize client");
        *STATE.client.write().unwrap() = Some(client_wrapper.clone());

        // Set up "from" addresses; easier to just set balance to 100e on init?
        let addresses = vec![
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            "0xcccccccccccccccccccccccccccccccccccccccc".to_string(),
            "0xdddddddddddddddddddddddddddddddddddddddd".to_string(),
            "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
            "0xffffffffffffffffffffffffffffffffffffffff".to_string(),
            "0x0000000000000000000000000000000000000000".to_string(),
        ];

        // Set balances to 100e
        let bal: U256 = parse_ether(100u64).expect("Error parsing init amount");
        for address in &addresses {
            // log!("Setting balance of {:?} to {:?}", address, bal);

            let setbal_params: Value = serde_json::json!([address, bal]);

            let _ = client_wrapper
                .client
                .provider()
                .request::<Value, Value>("hardhat_setBalance", setbal_params)
                .await;
        }

        // Write addresses to state.from_addresses
        *STATE.from_addresses.write().unwrap() =
            addresses.iter().map(|a| a.parse().unwrap()).collect();

        // Allows us to impersonate anyone via autoImpersonate
        let impersonate_params: Value = serde_json::json!([true]);

        let _ = client_wrapper
            .client
            .provider()
            .request::<Value, Value>("hardhat_autoImpersonateAccount", impersonate_params)
            .await;
    });
    Ok(())
}

pub fn set_balance(address: H160, balance: U256) {
    wasm_bindgen_futures::spawn_local(async move {
        let client_wrapper = shared_state::read_shared_client().unwrap();
        let set_bal_params: Value = serde_json::json!([address, balance]);

        let res = client_wrapper
            .client
            .provider()
            .request::<Value, Value>("hardhat_setBalance", set_bal_params)
            .await;

        match res {
            Ok(_) => {
                send_ok_popup("Success".to_string());
            }
            Err(e) => {
                send_error_popup(format!("ERROR: {}", e));
            }
        }
    });
}

pub fn set_storage_at(address: H160, slot: String, value: String) {
    wasm_bindgen_futures::spawn_local(async move {
        let client_wrapper = shared_state::read_shared_client().unwrap();
        let set_stor_params: Value = serde_json::json!([address, slot, value]);

        let res = client_wrapper
            .client
            .provider()
            .request::<Value, Value>("hardhat_setStorageAt", set_stor_params)
            .await;
        match res {
            Ok(_) => {
                send_ok_popup("Success".to_string());
            }
            Err(e) => {
                send_error_popup(format!("ERROR: {}", e));
            }
        }
    });
}

pub fn load_at_address_wrapper(compiled: Option<CompiledContract>, address: String) {
    wasm_bindgen_futures::spawn_local(async {
        match load_at_address(compiled, address).await {
            Ok(result) => {
                log!("loaded contract");

                STATE
                    .deployed_contracts
                    .write()
                    .unwrap()
                    .insert(result.address_h160.to_string(), result);
            }
            Err(e) => {
                send_error_popup(format!("ERROR: {}", e));
            }
        }
    });
}

async fn load_at_address(
    compiled: Option<CompiledContract>,
    address: String,
) -> Result<DeployedContract, eyre::Report> {
    log!("in load");

    let client_wrapper = shared_state::read_shared_client()?;

    // 5. Format the address
    let address_stripped = address.strip_prefix("0x").unwrap_or(&address);
    let address_bytes_vec = hex::decode(address_stripped).expect("Decoding failed");

    // Ensure the decoded byte vector has exactly 20 bytes
    if address_bytes_vec.len() != 20 {
        return Err(eyre::eyre!("Address length should be exactly 20 bytes"));
    }

    let mut address_bytes_array = [0u8; 20];
    address_bytes_array.copy_from_slice(&address_bytes_vec);

    let address_h160 = H160::from(address_bytes_array);
    // query for the code at the address via the provider/client
    let bytecode_b = client_wrapper
        .client
        .get_code(address_h160, None)
        .await
        .unwrap_or_else(|_| ethers::types::Bytes::from(vec![]));

    let bytecode_ascii = hex::encode(bytecode_b.as_ref());

    // log!("bytecode str {:?}", bytecode_ascii);

    // Handle the case where we're loading a compiled contract and the case where
    // we don't have an ABI (e.g. so we can easily hit an address w/ raw calldata)
    let solc_json = match compiled.clone() {
        Some(c) => c.compiled_json.clone(),
        None => {
            serde_json::json!({
                "abi": [],
                "bytecode": {
                    "object": bytecode_ascii,
                },
                "deployedBytecode": {
                    "object": bytecode_ascii,
                },
            })
        }
    };

    let abi_string = serde_json::to_string(&solc_json["abi"]).unwrap();
    let ethers_contract =
        ethers::abi::Contract::load(std::io::Cursor::new(abi_string)).expect("Error parsing abi");

    // TODO: yuck, only use this once but should clean up if possible
    let contract_instance: ContractInstance<
        Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
        SignerMiddleware<Provider<Http>, LocalWallet>,
    > = ContractInstance::new(address_h160, ethers_contract, client_wrapper.client.clone());

    log!("contract! {:?}", contract_instance);

    // Get the latest block for sorting
    let latest = client_wrapper
        .client
        .provider()
        .get_block(BlockNumber::Latest)
        .await?
        .unwrap_or_default();

    // 7. get the contract's address; should be same but also if it fails there's an issue
    let addr = contract_instance.address().to_owned();

    let name = match compiled.clone() {
        Some(c) => c.contract_name.clone(),
        None => "Unknown".to_string(),
    };

    // Create deployed contract and return it
    let deployed_contract = DeployedContract {
        name,
        abi: solc_json["abi"].clone(),
        bytecode_b: bytecode_b.clone(),
        bytecode_string: bytecode_ascii.to_string(),
        address_h160: addr,
        address_string: format!("{:#x}", addr),
        compiled_contract: compiled.clone(),
        deployed_block: latest,
        ..Default::default()
    };

    Ok(deployed_contract)
}

pub fn deploy_wrapper(compiled: CompiledContract, constructor_args: Vec<String>) {
    log!("in deploy wrapper");

    wasm_bindgen_futures::spawn_local(async {
        let result = deploy(compiled, constructor_args).await;
        match result {
            Ok(contract) => {
                log!("deployed contract: {:?}", contract.address_string);
                STATE
                    .deployed_contracts
                    .write()
                    .unwrap()
                    .insert(contract.address_h160.to_string(), contract.clone());
            }
            Err(e) => send_error_popup(format!("ERROR: {}", e)),
        }
    });
}

async fn deploy(
    compiled: CompiledContract,
    constructor_args: Vec<String>,
) -> Result<DeployedContract, Box<dyn std::error::Error>> {
    log!("in deploy");

    let client_wrapper = shared_state::read_shared_client()?;
    let client = client_wrapper.client;
    let tx_configs = shared_state::read_tx_configs();

    let solc_json = compiled.compiled_json.clone();
    let abi = serde_json::to_string(&solc_json["abi"]).unwrap();
    let ethers_contract =
        ethers::abi::Contract::load(std::io::Cursor::new(abi)).expect("Error parsing abi");
    let bytecode_ascii = solc_json["bytecode"]["object"].as_str().unwrap()[2..].to_string();

    // log!("bytecode str {:?}", bytecode_ascii);
    let bytecode_b =
        ethers::types::Bytes::from(hex::decode(bytecode_ascii.clone()).expect("error decoding"));
    // log!("bytecode bytes {:?}", bytecode_b);

    let tokens =
        crate::abi::constructor_args_to_tokens(solc_json["abi"].clone(), constructor_args)?;

    // 5. create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(ethers_contract, bytecode_b.clone().into(), client.clone());

    // 6. deploy it with the constructor arguments
    let mut deployer = factory.deploy_tokens(tokens)?;

    deployer.tx.set_from(tx_configs.from_address);

    deployer
        .tx
        .set_value(utils::eth_str_to_u256_wei(&tx_configs.value)?);

    deployer.tx.set_gas_price(tx_configs.gas_price);

    log!("deployer: {:?}", deployer);
    let contract = deployer.send().await?;
    log!("contract! {:?}", contract);
    let latest = client
        .provider()
        .get_block(BlockNumber::Latest)
        .await?
        .unwrap_or_default();
    // 7. get the contract's address
    let addr = contract.address().to_owned();
    log!("deployed! {:?}", addr);

    // Create deployed contract and return it
    let deployed_contract = DeployedContract {
        name: compiled.contract_name.clone(),
        abi: solc_json["abi"].clone(),
        bytecode_b,
        bytecode_string: bytecode_ascii,
        address_h160: addr,
        address_string: format!("{:#x}", addr),
        compiled_contract: Some(compiled),
        deployed_block: latest,
        ..Default::default()
    };

    Ok(deployed_contract)
}

pub fn deploy_raw_bytecode_wrapper(bytecode_string: String) {
    log!("in deploy wrapper");

    wasm_bindgen_futures::spawn_local(async {
        let result = deploy_raw_bytecode(bytecode_string).await;
        match result {
            Ok(contract) => {
                log!("deployed contract: {:?}", contract.address_string);
                STATE
                    .deployed_contracts
                    .write()
                    .unwrap()
                    .insert(contract.address_h160.to_string(), contract.clone());
            }
            Err(e) => send_error_popup(format!("ERROR: {:?}", e)),
        }
    });
}

async fn deploy_raw_bytecode(bytecode_ascii: String) -> Result<DeployedContract> {
    log!("in deploy");
    let client_wrapper = shared_state::read_shared_client()?;
    let client = client_wrapper.client;
    let tx_configs = shared_state::read_tx_configs();

    // Decode the provided bytecode
    let bytecode_b = ethers::types::Bytes::from(hex::decode(bytecode_ascii.clone())?);

    log!("{} {:?}", bytecode_ascii, bytecode_b);
    // Create an empty ABI
    let empty_contract = ethers::abi::Contract::load(std::io::Cursor::new("[]".to_string()))?;

    // Create a factory which will be used to deploy instances of the contract
    let factory = ContractFactory::new(empty_contract, bytecode_b.clone(), client.clone());
    log!("factory: {:?}\n", factory);
    // Deploy the contract

    let mut deployer = factory.deploy(())?;

    deployer.tx.set_from(tx_configs.from_address);
    deployer
        .tx
        .set_value(utils::eth_str_to_u256_wei(&tx_configs.value)?);
    deployer.tx.set_gas_price(tx_configs.gas_price);
    let contract = deployer.send().await?;
    log!("contract: {:?}", contract);
    // 7. get the contract's address
    let addr = contract.address().to_owned();
    log!("deployed! {:?}", addr);

    // Create and return a DeployedContract
    let deployed_contract = DeployedContract {
        name: "Unknown".to_string(),
        bytecode_b: bytecode_b.clone(),
        bytecode_string: bytecode_ascii.to_string(),
        address_h160: addr,
        address_string: format!("{:#x}", addr),
        ..Default::default()
    };

    Ok(deployed_contract)
}
