use egui::epaint::ahash::HashMap;
use ethers::{
    abi::{ParamType, Token},
    types::U256,
};
use eyre::{eyre, Result};
use serde_json::Value;

use num_bigint::{BigInt, BigUint};
use num_traits::Num;

// TODO: switch to alloy?

pub fn encode_fn_call_to_tokens(
    func_name: String,
    abi: Value,
    func_param_input: HashMap<String, HashMap<String, String>>,
) -> Result<Vec<Token>> {
    // create ethers contract object for this contract
    let abi: ethers::abi::Abi = serde_json::from_value(abi.clone()).unwrap();

    // This will probably run into issues if there are multiple fns w/ the same name;
    // should switch to using selectors
    let func = abi
        .functions
        .get(func_name.as_str())
        .expect("Failed to get function args")
        .get(0)
        .expect("Failed to get function args");

    let mut tokens: Vec<Token> = Vec::new();

    for param in func.inputs.iter() {
        let param_name = param.name.clone();
        let param_kind = param.kind.clone();
        // let param_internal_type = param.internal_type.clone();

        if let Some(input_value) = func_param_input
            .get(&func_name)
            .and_then(|param_map| param_map.get(&param_name))
        {
            let token = parse_input_to_token(param_kind, input_value.clone())?;

            tokens.push(token);
        }
    }

    Ok(tokens)
}

// TODO - alloy?
pub fn encode_fn_call_to_calldata(
    func_name: String,
    abi: &Value,
    func_param_input: &HashMap<String, HashMap<String, String>>,
) -> Result<String> {
    // create ethers contract object for this contract
    let abi: ethers::abi::Abi = serde_json::from_value(abi.clone()).unwrap();
    let base = ethers::contract::BaseContract::from(abi);

    // This will probably run into issues if there are multiple fns w/ the same name;
    // should switch to using selectors
    let func = base
        .abi()
        .functions
        .get(func_name.as_str())
        .expect("Failed to get function args")
        .get(0)
        .expect("Failed to get function args");

    let mut tokens: Vec<Token> = Vec::new();

    //
    for param in func.inputs.iter() {
        let param_name = param.name.clone();
        let param_kind = param.kind.clone();
        // let param_internal_type = param.internal_type.clone();

        if let Some(input_value) = func_param_input
            .get(&func_name)
            .and_then(|param_map| param_map.get(&param_name))
        {
            let token = parse_input_to_token(param_kind, input_value.clone())?;

            tokens.push(token);
        }
    }

    Ok(base
        .encode(&func_name, &tokens[..])
        .expect("Failed to encode")
        .to_string())
}

pub fn constructor_args_to_tokens(abi: Value, args: Vec<String>) -> Result<Vec<Token>> {
    let abi: ethers::abi::Abi = serde_json::from_value(abi.clone()).unwrap();
    let mut tokens: Vec<Token> = Vec::new();

    if let Some(constructor) = abi.constructor {
        for (idx, param) in constructor.inputs.iter().enumerate() {
            let param_name = param.name.clone();
            let param_kind = param.kind.clone();
            let token = parse_input_to_token(param_kind, args[idx].clone())?;

            tokens.push(token);
        }
    }
    Ok(tokens)
}

pub fn decode_return_values(
    func_name: String,
    abi: &Value,
    return_data: &ethers::types::Bytes,
) -> Result<Vec<Token>> {
    // create ethers contract object for this contract
    let abi: ethers::abi::Abi = serde_json::from_value(abi.clone()).unwrap();
    let base = ethers::contract::BaseContract::from(abi);

    // This will probably run into issues if there are multiple fns w/ the same name;
    // should switch to using selectors
    let func = base
        .abi()
        .functions
        .get(func_name.as_str())
        .expect("Failed to get function args")
        .get(0)
        .expect("Failed to get function args");

    func.decode_output(return_data).map_err(eyre::Report::from)
}

pub fn parse_input_to_token(param_kind: ParamType, input_value: String) -> Result<Token> {
    match param_kind {
        ParamType::Address => input_value
            .parse()
            .map(Token::Address)
            .map_err(|_| eyre!("Failed to parse Address")),
        ParamType::Bool => input_value
            .parse()
            .map(Token::Bool)
            .map_err(|_| eyre!("Failed to parse Bool")),
        ParamType::String => Ok(Token::String(input_value.clone())),
        ParamType::Uint(_) => {
            let value = if input_value.starts_with("0x") {
                BigUint::from_str_radix(&input_value[2..], 16)?
            } else {
                BigUint::from_str_radix(&input_value, 10)?
            };
            let bytes = value.to_bytes_be();
            let u256_value = U256::from_big_endian(&bytes);
            Ok(Token::Uint(u256_value))
        }

        // this def isn't quite right, needs some work
        ParamType::Int(_) => {
            if input_value.starts_with('-') {
                // Handle negative numbers
                let positive_part = &input_value[1..]; // Strip the `-`
                let value = if positive_part.starts_with("0x") {
                    BigInt::from_str_radix(&positive_part[2..], 16)? // strip the 0x
                } else {
                    BigInt::from_str_radix(positive_part, 10)?
                };
                // Calculate two's complement
                let max_u256 = BigInt::from(2).pow(256);
                let two_complement_value = max_u256 - value;
                let (_, bytes) = two_complement_value.to_bytes_be();
                let u256_value = U256::from_big_endian(&bytes);
                Ok(Token::Int(u256_value))
            } else {
                // Handle positive numbers as before
                let value = if input_value.starts_with("0x") {
                    BigUint::from_str_radix(&input_value[2..], 16)?
                } else {
                    BigUint::from_str_radix(&input_value, 10)?
                };
                let bytes = value.to_bytes_be();
                let u256_value = U256::from_big_endian(&bytes);
                Ok(Token::Int(u256_value))
            }
        }
        ParamType::Bytes => Ok(Token::Bytes(input_value.into_bytes())),
        ParamType::FixedBytes(_) => Ok(Token::FixedBytes(input_value.into_bytes())),

        // TODO:
        // ParamType::Array(_) => Ok(Token::Array(
        //     input_value.clone().into_bytes().into_iter().collect(),
        // )),
        // ParamType::FixedArray(_, _) => {
        //     Some(Token::FixedArray(input_value.clone().into_bytes().into_iter().collect()))
        // }
        // ParamType::Tuple(_) => {
        //     Some(Token::Tuple(input_value.clone().into_bytes().into_iter().collect()))
        // }
        // ... handle other cases similarly ...
        _ => Err(eyre!("Unsupported or unhandled type")),
    }
}
