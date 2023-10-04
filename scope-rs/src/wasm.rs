use crate::{backend, components::CompiledContract, shared_state::STATE};
use js_sys::{Array, Date};
use serde_json::Value;
use wasm_bindgen::prelude::*;

// Receiver functions for messages from extension
#[wasm_bindgen]
pub fn hello_from_rust() -> Result<String, JsValue> {
    Ok("Hello from rust".to_string())
}

#[wasm_bindgen]
pub fn receive_vscode_style(received_style: JsValue) {
    log!("`.receive_vscode_style");
    let json_str = received_style.as_string().unwrap();
    log!("{:?}", json_str);
    let parsed_json: Value = serde_json::from_str(&json_str).unwrap();
    *STATE.vscode_style.write().unwrap() = parsed_json;
    log!("{:?}", STATE.vscode_style.read().unwrap());
}

// Receive the .sol file paths of the files currently open in the editor
#[wasm_bindgen]
pub fn receive_open_file_paths(js_filepaths: &Array) {
    log!("`.receive_open_files");
    // Convert the js Array to a rust String vec
    let length = js_filepaths.length();
    let mut rust_strings = Vec::<String>::new();

    for i in 0..length {
        if let Some(string) = js_filepaths.get(i).as_string() {
            rust_strings.push(string);
        }
    }
    log!("{:?}", rust_strings);
    *STATE.open_files.write().unwrap() = rust_strings;
}

// for receiving raw source code
#[wasm_bindgen]
pub fn receive_file_contents(js_contents: &Array) {
    log!("in backend.receive_file_contents");
    tracing::trace!("in backend.receive_file_contents");
}

#[wasm_bindgen]
pub fn receive_compiled_solidity(sol_json_string: JsValue, file_path: String) {
    log!("in backend.receive_compiled_solidity");
    let json_str = sol_json_string.as_string().unwrap();
    let parsed_json: Value = serde_json::from_str(&json_str).unwrap();

    let compiled = CompiledContract::new(file_path, parsed_json);
    *STATE.target_compiled.write().unwrap() = Some(compiled.clone());
    // log!("{:?}", STATE.target_compiled.read().unwrap());

    // write filepath to shared storage
    // *STATE.compiled_sol_file_path.write().unwrap() = Some(file_path);
}

#[wasm_bindgen]
pub fn handle_completed_forge_build() {
    *STATE.completed_compile.write().unwrap() = Some(true);
    get_open_files();
}

#[wasm_bindgen]
pub fn handle_losing_focus() {
    log!("handle lost focus");
    let now = Date::now();
    *STATE.last_focus_change.write().unwrap() = Some(now);
    *STATE.has_focus.write().unwrap() = false;
}

#[wasm_bindgen]
pub fn handle_gaining_focus() {
    log!("handle gained focus");

    *STATE.has_focus.write().unwrap() = true;
}

#[wasm_bindgen]
pub fn handle_file_opened_or_closed() {
    get_open_files();
}

// Callable javascript functions
#[wasm_bindgen(module = "/src/bridge.js")]
extern "C" {
    pub fn get_open_files();
    pub fn get_file_contents(file_path: String);
    pub fn get_compiled_solidity(file_path: String);
    pub fn execute_shell_command(command: String);
    pub fn forge_build();
    pub fn send_error_to_vscode(error_text: String);
    pub fn send_ok_to_vscode(error_text: String);
    pub fn get_vscode_style() -> JsValue;
}
